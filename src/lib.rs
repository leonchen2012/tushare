//! # Tushare
//! A rust library to read the data from http://api.tushare.pro and tranform it into polars dataframe object.
//! ## Example
//! Typical approach to get 1 row stock info of 000001.SZ(Pingan Bank) with a builder pattern:
//! ```rust
//! use tushare::*;
//! let tushare = Tushare::new("<your token>");
//! let df = tushare.querybuilder("daily")
//!             .addparam("trade_date", "20240424") //opiontal step
//!             .addparam("ts_code","000001.SZ") //optional step
//!             .fields("ts_code,trade_date,open,high,low,close,pre_close,change,pct_chg,vol") //optional step
//!             .query()?;
//! print!("here are the results\n");
//! print!("{df:?}");
//! ```
//! ## Note
//! 1. Get a token from tushare.pro site before you start.
//! 2. Param api_name for function tushare.querybuilder() is predefined by Tushare webapi, refer to <https://tushare.pro/document/1?doc_id=130>.
//! If you are still confusing what string should be used here (like I do), refer to the "api" field of [this doc](https://github.com/ProV1denCEX/Tushare.jl/blob/master/src/Tushare.yaml)
//! from ProV1denCEX. I personally found it very useful, together with other optional fields.
//! 3. Param k/v, fields are defined clearly on Tushare website, see example <https://tushare.pro/document/2?doc_id=25>
//! 4. Be aware of date string. They must be in *YYYYMMDD* format. Otherwise empty data will be returned.
//! 5. The date column of dataframe are *String* type by tushare server. You could tranform the whole column to datetime using Polars by yourself. 
//! 
//! ## Recommended error handling flow
//! See [TushareError] for error definition details.
//! The only place that will produce an error is the query() method of QueryBuilder. The recommended error handling flow is:
//! 1. NetworkError occurs during http request. You may want to retry if your network is not stable.
//! 2. RequestError occurs if Tushare server explicity return a nonzero code in its body. See error message for more details. Possible reason: wrong token
//! 3. JsonError/DataError occur if body returned by Tushare server is not the same as document. Normally this won't happen. 
//! You could set the log level to "Info" and check the log for the request and response body.
//! 4. EmptyError occurs if Tushare return zero rows of data. Because this makes it impossible to infer the data type of each columns, it was marked as error.
//! Usually you can check if wrong date format is used. The correct format is "20240404".
//! 5. PolarsError occurs during the json -> dataframe transforming. Again, it should not happen. Check the info log for more details.

pub mod builder;
pub mod tushare;
pub use tushare::Tushare;
pub use builder::{Dict, QueryBuilder, TushareError};



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_query() {
        let tushare = Tushare::new("<token here>");
        let input: Dict= Dict::from([("start_date".into(), "2023-01-01".into()), ("end_date".into(), "2024-04-23".into())]);
        let df = tushare.querybuilder("daily")
                .params(input)
                .fields("ts_code,trade_date,open,high,low,close,pre_close,change,pct_chg,vol")
                .query();
        print!("here are the results");
        print!("{df:?}");
    }
}
