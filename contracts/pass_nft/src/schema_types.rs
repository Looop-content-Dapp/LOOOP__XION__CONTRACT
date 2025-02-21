use cosmwasm_schema::{cw_serde, QueryResponses};


// Create schema-specific types
#[cw_serde]
#[derive(QueryResponses)]
pub enum SchemaQueryMsg {
  #[returns(crate::msg::ValidityResponse)] 
    CheckValidity { token_id: String },

    #[returns(crate::msg::ConfigResponse)] 
    GetConfig {},

    #[returns(crate::msg::ArtistInfoResponse)]
    GetArtistInfo {},

    #[returns(crate::msg::PassResponse)]
    GetUserPass { symbol: String, owner: String },
}