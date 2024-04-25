use crate::builder::*;

/// A struct to hold all tushare calls
pub struct Tushare {
    /// Internal string holds tushare webapi access token.
    /// Used in every call as a hidden parameter.
    pub token: String,
    /// This is actually a constant of "http://api.tushare.pro"
    pub api_endpoint: String,
}

/// Tushare struct methods implementation
impl Tushare{
    /// Only entry to create a tushare object
    /// # token
    /// The token is necessary for every call
    /// Apply it before you do any access 
    pub fn new(token: String) -> Self {
        Tushare{ token : token,
                 api_endpoint: "http://api.tushare.pro".to_string()}
    }

    /// Create a QueryBuilder to actually build and process the query
    /// # api_name: 
    pub fn querybuilder(self: &Self, api_name: String) -> QueryBuilder{
        QueryBuilder::new(self, api_name)
    }

}
