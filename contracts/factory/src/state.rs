use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub nft_code_id: u64, // Code ID of the NFT contract to instantiate
    pub owner: Addr,     // Owner of the factory contract
}

#[cw_serde]
pub struct Collection {
    pub name: String,            // Collection name
    pub symbol: String,          // Collection symbol
    pub artist: Addr,            // Artist's address
    pub contract_address: Addr,  // Address of the NFT contract
    pub created_at: u64,         // Block timestamp when created
}

// Store the factory configuration
pub const CONFIG: Item<Config> = Item::new("config");

// Map artist address to their collection(s)
// artist address -> Collection
pub const COLLECTIONS: Map<&Addr, Collection> = Map::new("collections");

// Store total number of collections
pub const COLLECTION_COUNT: Item<u64> = Item::new("collection_count");

pub const COLLECTION_BY_SYMBOL: Map<String, Addr> = Map::new("collection_by_symbol");