use cosmwasm_schema::{cw_serde, QueryResponses};


// Create schema-specific types
#[cw_serde]
#[derive(QueryResponses)]
pub enum SchemaQueryMsg {
  #[returns(crate::msg::ValidityResponse)] 
    CheckValidity { token_id: String },

    #[returns(crate::msg::ConfigResponse)] 
    GetConfig {},
}