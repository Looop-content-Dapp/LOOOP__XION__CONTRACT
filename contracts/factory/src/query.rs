use cosmwasm_std::{Deps, StdResult, Order};
use crate::msg::{ConfigResponse, CollectionResponse};
use crate::state::{CONFIG, COLLECTIONS, COLLECTION_COUNT, COLLECTION_BY_SYMBOL};

// Query factory configuration
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let total_collections = COLLECTION_COUNT.load(deps.storage)?;

    Ok(ConfigResponse {
        nft_code_id: config.nft_code_id,
        owner: config.owner.to_string(),
        total_collections,
    })
}

// Query collection by artist address
pub fn query_collection(deps: Deps, artist: String) -> StdResult<CollectionResponse> {
    let artist_addr = deps.api.addr_validate(&artist)?;
    let collection = COLLECTIONS.may_load(deps.storage, &artist_addr)?;
    
    Ok(CollectionResponse { 
        collection 
    })
}

// Query collection by symbol
pub fn query_collection_by_symbol(deps: Deps, symbol: String) -> StdResult<CollectionResponse> {
    let contract_addr = COLLECTION_BY_SYMBOL.may_load(deps.storage, symbol)?;
    
    match contract_addr {
        Some(addr) => {
            // Find collection with matching contract address
            let collections: Vec<_> = COLLECTIONS
                .range(deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<_>>>()?;

            let collection = collections
                .into_iter()
                .find(|(_, col)| col.contract_address == addr)
                .map(|(_, col)| col);

            Ok(CollectionResponse { collection })
        },
        None => Ok(CollectionResponse { collection: None }),
    }
}