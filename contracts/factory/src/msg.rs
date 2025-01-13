use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use crate::state::Collection;

#[cw_serde]
pub struct InstantiateMsg {
    pub nft_code_id: u64, 
}

#[cw_serde]
pub enum ExecuteMsg {
   // Create a new collection for an artist
   CreateCollection {
    name: String,
    symbol: String,
    pass_price: u128,
    pass_duration: u64,
    grace_period: u64,
    payment_address: Addr,
},

UpdateNftCodeId {
    code_id: u64,
},
}


#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Get factory configuration
    #[returns(ConfigResponse)]
    Config {},
    
    // Get collection by artist address
    #[returns(CollectionResponse)]
    Collection { 
        artist: String 
    },

    // Get collection by symbol
    #[returns(CollectionResponse)]
    CollectionBySymbol { 
        symbol: String 
    },
}

// Response types
#[cw_serde]
pub struct ConfigResponse {
    pub nft_code_id: u64,
    pub owner: String,
    pub total_collections: u64,
}

#[cw_serde]
pub struct CollectionResponse {
    pub collection: Option<Collection>,
}
