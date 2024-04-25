 # Tushare
 A rust tushare library to read the data from http://api.tushare.pro and tranform it into polars dataframe object.
 ## Example
 Typical approach to get 1 row stock info of 000001.SZ(Pingan Bank) with a builder pattern:
 ```rust
 use tushare::*;
 let tushare = Tushare::new("<your token>".to_string());
 let df = tushare.querybuilder("daily".to_string())
             .addparam("trade_date".to_string(), "20240424".to_string()) //opiontal step
             .addparam("ts_code".to_string(),"000001.SZ".to_string()) //optional step
             .fields("ts_code,trade_date,open,high,low,close,pre_close,change,pct_chg,vol".to_string()) //optional step
             .query()?;
 print!("here are the results\n");
 print!("{df:?}");
 ```
 ## Note
 1. Get a token from tushare.pro site before you can started.
 2. Param api_name for function tushare.querybuilder() is predefined by Tushare webapi, refer to <https://tushare.pro/document/1?doc_id=130>.
 If you are still confusing what string should be used here (like I do), refer to the "api" field of <https://github.com/ProV1denCEX/Tushare.jl/blob/master/src/Tushare.yaml>
 from ProV1denCEX. I personally found it very useful, together with other optional fields.
 3. Param k/v, fields are defined clearly on Tushare website, see example <https://tushare.pro/document/2?doc_id=25>
 4. Be aware of date string. They must be in *YYYYMMDD* format. Otherwise empty data will be returned.
 5. The date column of return dataframe are *String* type by tushare. You could tranform the whole column to datetime using Polars by yourself. 
 
 ## Recommended error handling flow
 See [TushareError] for error definition details.
 The flow is:
 1. NetworkError occurs during network request. You may want to retry if the network is not stable.
 2. RequestError occurs if Tushare explicity return a nonzero code in its body. See error message for more details. Possible reason: wrong token
 3. JsonError/DataError occur if Tushare returned body is not as expected in document. Normally this won't happen. 
 You could set the log level to "Info". This library will log the request and response body in info.
 4. EmptyError occurs if Tushare return zero rows of data. Because this makes it impossible to infer the data type of each columns, it was marked as error.
 Usually you can check if wrong date format is used. The correct format is "20240404".
 5. PolarsError occurs during the json -> dataframe transforming. Again, it should not happen. Check the info for more details.
