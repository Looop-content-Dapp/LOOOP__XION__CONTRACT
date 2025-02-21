use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use crate::error::ContractError;

#[cw_serde]
pub struct Config {
    pub nft_code_id: u64,
    pub admin: Addr,
    pub price: u128,
    pub duration: u64,
    pub grace_period: u64,  
    pub payment_address: Addr,
    //royalty 
    pub house_percentage: u32,
    pub artist_percentage: u32,
}

#[cw_serde]
pub struct Collection {
    pub name: String,
    pub symbol: String,
    pub artist: Addr,
    pub contract_address: Addr,
    pub created_at: u64,
    pub minter: Addr,
    pub collection_info: String,
     // collection-specific royalty settings
     pub house_percentage: u32,
     pub artist_percentage: u32,
}


impl Collection {
    pub fn new(
        name: String,
        symbol: String,
        artist: Addr,
        minter: Addr,
        contract_address: Addr,
        created_at: u64,
        collection_info: String,
        house_percentage: u32,
        artist_percentage: u32,
    ) -> Self {
        Self {
            name,
            symbol,
            artist,
            minter,
            contract_address,
            created_at,
            collection_info,
            house_percentage,
            artist_percentage
        }
    }

    pub fn is_authorized(&self, addr: &Addr) -> bool {
        *addr == self.artist || *addr == self.minter
    }

    pub fn validate_royalties(&self) -> Result<(), ContractError> {
        if self.house_percentage + self.artist_percentage != 100 {
            return Err(ContractError::InvalidRoyalties { });
        }
        Ok(())
    }
}

pub fn save_new_collection(
    storage: &mut dyn cosmwasm_std::Storage,
    collection: &Collection,
) -> Result<(), ContractError> {

    
       // Validate royalties
      let _ = collection.validate_royalties();
 
    // Ensure the symbol is not already taken
    if SYMBOL_TAKEN.may_load(storage, collection.symbol.clone())?.unwrap_or(false) {
        return Err(ContractError::SymbolAlreadyTaken {});
    }

    // Increment collection count, ensuring no overflow
    COLLECTION_COUNT.update(storage, |count| {
        count
            .checked_add(1)
            .ok_or_else(|| ContractError::MaxSupplyReached { })
    })?;

    // Save collection data
    COLLECTIONS.save(storage, collection.symbol.clone(), collection)?;

    // Mark the symbol as taken
    SYMBOL_TAKEN.save(storage, collection.symbol.clone(), &true)?;

    // Update the artist's collections
    let mut artist_collections = ARTIST_COLLECTIONS
        .may_load(storage, &collection.artist)?
        .unwrap_or_default();
    if !artist_collections.contains(&collection.symbol) {
        artist_collections.push(collection.symbol.clone());
    }
    ARTIST_COLLECTIONS.save(storage, &collection.artist, &artist_collections)?;

    Ok(())
}





pub const CONFIG: Item<Config> = Item::new("config");

pub const COLLECTIONS: Map<String, Collection> = Map::new("collections");

pub const SYMBOL_TAKEN: Map<String, bool> = Map::new("symbol_taken");

pub const ARTIST_COLLECTIONS: Map<&Addr, Vec<String>> = Map::new("artist_collections");

pub const COLLECTION_COUNT: Item<u64> = Item::new("collection_count");