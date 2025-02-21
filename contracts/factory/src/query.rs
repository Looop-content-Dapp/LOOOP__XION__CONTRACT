
use cosmwasm_std::{Deps, StdResult};
use crate::msg::{ConfigResponse, CollectionResponse, CollectionsResponse};
use crate::state::{CONFIG, COLLECTIONS, COLLECTION_COUNT, ARTIST_COLLECTIONS, SYMBOL_TAKEN, Collection};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        nft_code_id: config.nft_code_id,
        admin: config.admin.to_string(),
        total_collections: COLLECTION_COUNT.load(deps.storage)?,
        house_percentage: config.house_percentage,
        artist_percentage: config.artist_percentage
    })
}

pub fn query_collection(deps: Deps, artist: String) -> StdResult<CollectionResponse> {
    let artist_addr = deps.api.addr_validate(&artist)?;
    let collections = ARTIST_COLLECTIONS.may_load(deps.storage, &artist_addr)?.unwrap_or_default();
    
    let collection = collections.first()
        .and_then(|symbol| COLLECTIONS.may_load(deps.storage, symbol.clone()).ok().flatten());
    
    Ok(CollectionResponse { collection })
}

pub fn query_artist_collections(
    deps: Deps,
    artist: String,
    limit: Option<u32>,
) -> StdResult<CollectionsResponse> {
    let artist_addr = deps.api.addr_validate(&artist)?;
    let collections = ARTIST_COLLECTIONS.may_load(deps.storage, &artist_addr)?.unwrap_or_default();
    
    let limit = limit.unwrap_or(10) as usize;
    let collections = collections
        .into_iter()
        .filter_map(|symbol| COLLECTIONS.may_load(deps.storage, symbol).ok().flatten())
        .take(limit)
        .collect();
    
    Ok(CollectionsResponse { collections })
}

pub fn query_all_collections(
    deps: Deps,
    limit: Option<u32>,
) -> StdResult<CollectionsResponse> {
    let limit = limit.unwrap_or(10) as usize;
    let collections: Vec<Collection> = COLLECTIONS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .take(limit)
        .filter_map(|item| item.ok().map(|(_, collection)| collection))
        .collect();

    Ok(CollectionsResponse { collections })
}


pub fn query_is_symbol_available(deps: Deps, symbol: String) -> StdResult<bool> {
    let is_taken = SYMBOL_TAKEN.may_load(deps.storage, symbol)?.unwrap_or(false);
    Ok(!is_taken)
}