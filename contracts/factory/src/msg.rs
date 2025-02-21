use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use crate::state::Collection;

#[cw_serde]
pub struct InstantiateMsg {
    pub nft_code_id: u64,
    pub price: u128,
    pub duration: u64,
    pub grace_period: u64,  
    pub payment_address: Addr,

    pub house_percentage: u32,
    pub artist_percentage: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateCollection {
        name: String,
        symbol: String,
        artist: Addr,
        minter: Addr,
        collection_info: String,
    },
    UpdateNftCodeId {
        code_id: u64,
    },

    UpdateRoyalties {
        house_percentage: u32,
        artist_percentage: u32,
    },
}


#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    
    #[returns(CollectionResponse)]
    Collection { 
        artist: String 
    },

    #[returns(CollectionsResponse)]
    ArtistCollections { 
        artist: String,
        limit: Option<u32>,
    },

    #[returns(CollectionsResponse)]
    AllCollections {
        limit: Option<u32>,
    },

    #[returns(bool)]
    IsSymbolAvailable { 
        symbol: String 
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub nft_code_id: u64,
    pub admin: String,
    pub total_collections: u64,

    pub house_percentage: u32,
    pub artist_percentage: u32,
}

#[cw_serde]
pub struct CollectionResponse {
    pub collection: Option<Collection>,
}

#[cw_serde]
pub struct CollectionsResponse {
    pub collections: Vec<Collection>,
}

// Events remain the same
#[cw_serde]
pub struct CollectionCreatedEvent {
    pub name: String,
    pub symbol: String,
    pub artist: Addr,
    pub minter: Addr,
    pub contract_address: Addr,
    pub house_percentage: u32,
    pub artist_percentage: u32,
}

#[cw_serde]
pub struct CollectionUpdatedEvent {
    pub symbol: String, 
    pub is_active: bool,
}

