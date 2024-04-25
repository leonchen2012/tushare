use crate::tushare::Tushare;
use log::{error, info};
use polars::prelude::*;
use reqwest;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Cursor;
use thiserror::Error;

/// TushareError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum TushareError {
    /// Tushare returns empty rows.
    /// It might have returned dataframe column names but it's impossible to infer column type without row data
    /// If this is the intended behavior, the caller should handle this error  
    #[error("Tushare returned empty data")]
    EmptyError,
    /// Tushare returns non-zero error code in response body
    #[error("Tushare request return error:{code}, msg:{msg}")]
    RequestError { code: String, msg: String },
    /// Transform Tushare returned json to polars json error
    #[error("Expected json node {0} not exist")]
    DataError(String),

    /// Represents a network failure to read tushare web api.
    #[error("Request network error, not accessable or possible 500")]
    NetworkError(#[from] reqwest::Error),

    /// Represents a failure to decode tushare result json
    #[error("Parse tushare response json error")]
    JsonError(#[from] serde_json::Error),

    /// Represents a failure to converting json to polars dataframe
    #[error("Convert json to polars dataframe error")]
    PolarsError(#[from] polars::error::PolarsError)
}

/// Used to specify API parameter pairs
pub type Dict = HashMap<String, String>;

fn mergedict(map_pre:Dict, map_post:Dict) -> Dict{
    map_pre.into_iter().chain(map_post).collect()
}


/// A tushare query that satistfies rust builder pattern
/// The QueryBuilder is immutable, which means a new instance
/// of QueryBuilder will be created during params()/addparam()/fields() calling
/// So it is safe for multi-threading
pub struct QueryBuilder<'a> {
    tushare: &'a Tushare,
    api_name: String,
    params: Option<Dict>,
    fields: Option<String>,
}

impl<'a> QueryBuilder<'a> {
    pub(crate) fn new(tushare: &'a Tushare, api_name: String) -> Self {
        QueryBuilder {
            tushare,
            api_name,
            params: None,
            fields: None,
        }
    }

    /// Set parameters to the query. Parameters are e.g. trade_date, start_date, end_date, market, exchange.
    /// For detailed param explanation, see the tushare api website <https://tushare.pro/document/2?doc_id=25> .
    /// Note this step is optional, you can safely ignore this during ramp up, and the return will be up to 6,000 rows.
    /// The main purpose of parameters is to define your requirements clearly
    /// # param
    /// The predefined request parameters according to each api_name, e.g. 'start_date', 'end_date'
    pub fn params(self: &Self, params: Dict) -> Self {
        QueryBuilder {
            tushare: self.tushare,
            api_name: self.api_name.clone(),
            params: Some(params),
            fields: self.fields.clone(),
        }
    }

    /// Add a parameter to the query, e.g. trade_date, start_date, end_date, market, exchange.
    /// This is a helper function for params() since constructing a hashmap is a little bit boring.
    /// Parameter pairs with the same key will be overwritten.
    /// For detailed param explanation, see the tushare api website <https://tushare.pro/document/2?doc_id=25> .
    /// Note this is optional, you can ignore this during ramp up, and the return will be up to 6,000 rows.
    /// The main purpose of parameters is to define your requirements clearly.
    /// # k/v
    /// The predefined request key/value pair according to each api_name, e.g. 'start_date', 'end_date'
    pub fn addparam(self: &Self, k:String, v:String) -> Self{
        let new_paramdict = Dict::from([(k, v)]);
        let paramdict = match &self.params {
            Some(dict) => mergedict(dict.clone(),new_paramdict),
            None => new_paramdict
        };
        QueryBuilder{
            tushare: self.tushare,
            api_name: self.api_name.clone(),
            params: Some(paramdict),
            fields: self.fields.clone(),            
        }
    }
    /// Set the return fields to the query.
    /// For detailed return field explanation, see the tushare api website https://tushare.pro/document/2?doc_id=25 .
    /// Note this is optional, you can ignore this during ramp up, and the return will be up to 10~20 columns.
    /// You may want to use it to reduce network IO and clarify your requirement clearly.
    /// # fields
    /// The predefined fields string separated with commas, e.g. "ts_code,trade_date,open,high,low,close,pre_close"
    pub fn fields(self: &Self, fields: String) -> Self {
        QueryBuilder {
            tushare: self.tushare,
            api_name: self.api_name.clone(),
            params: self.params.clone(),
            fields: Some(fields),
        }
    }

    fn build(self: &Self) -> Value {
        match (&self.params, &self.fields) {
            (Some(p), Some(f)) => json!({
                "api_name":self.api_name,
                "token":self.tushare.token,
                "params": p,
                "fields": f
            }),
            (Some(p), None) => json!({
                "api_name":self.api_name,
                "token":self.tushare.token,
                "params": p,
                "fields": null
            }),
            (None, Some(f)) => json!({
                "api_name":self.api_name,
                "token":self.tushare.token,
                "params": null,
                "fields": f
            }),
            (None, None) => json!({
                "api_name":self.api_name,
                "token":self.tushare.token,
                "params": null,
                "fields": null
            }),
        }
    }

    fn json_reformat(resp_json:Value) -> Result<Vec<Value>, TushareError>{
        let mut data_json: Vec<Value> = vec![];
        let fields_json = resp_json["data"]["fields"]
            .as_array()
            .ok_or(TushareError::DataError("data/fields".to_string()))?;
        let mut fields: Vec<&str> = vec![];
        for (i, field) in fields_json.iter().enumerate() {
            let _field = field
                .as_str()
                .ok_or(TushareError::DataError(format!("data/fields at {i}")))?;
            fields.push(_field);
        }
        let data: &Vec<Value> = resp_json["data"]["items"]
            .as_array()
            .ok_or(TushareError::DataError("data/items".to_string()))?;
        for (i, item) in data.iter().enumerate() {
            let item_data = item.as_array().ok_or(TushareError::DataError(format!(
                "data/items/{i} is expected to be an array"
            )))?;
            let mut item_json: serde_json::Map<String, Value> = serde_json::Map::new();
            for (k, v) in fields.iter().zip(item_data.iter()) {
                item_json.insert(k.to_string(), v.clone());
            }
            data_json.push(Value::Object(item_json))
        }
        Ok(data_json)

    }


    /// Query API predefined request type & parameters and return a Data Frame as output
    /// Fundamental entry for every tushare data access.
    pub fn query(self: &Self) -> Result<DataFrame, TushareError> {
        let tushare_request = self.build();
        info!(
            "Request text:\n {}\n",
            serde_json::to_string(&tushare_request).unwrap_or("to str error".to_string())
        );
        let client = Client::new();
        let resp_text = client
            .post(self.tushare.api_endpoint.clone())
            .body(tushare_request.to_string())
            .send()? // sending network error
            .error_for_status()? // 400 or other http error
            .text()?;
        info!("Network return:\n {}\n", resp_text);
        let resp_json: Value = serde_json::from_str(&resp_text)?;
        if let Some(ret_code) = resp_json["code"].as_i64() {
            info!("resp code: {:?}", ret_code);    
            if ret_code != 0 {
                let code = resp_json["code"].as_str().unwrap_or("unknown");
                let msg = resp_json["msg"].as_str().unwrap_or("unknown");
                return Err(TushareError::RequestError {
                    code: code.to_string(),
                    msg: msg.to_string(),
                });
            }
        }
        let data_json = Self::json_reformat(resp_json)?;
        let data_str = serde_json::to_string(&data_json)?;
        info!("data_str: {}", data_str);
        if data_str == "" || data_str == "[]"{
            return Err(TushareError::EmptyError)
        }
        let cursor = Cursor::new(data_str);
        let df = JsonReader::new(cursor).finish()?;
        Ok(df)
    }
}
