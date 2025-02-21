use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, 
    Response, StdResult, Reply,
 };
 use cw2::set_contract_version;
 
 use crate::error::ContractError;
 use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
 use crate::state::{Config, CONFIG, COLLECTION_COUNT};
 use crate::execute::{create_collection, update_nft_code_id, reply_collection_created, update_royalties};
 use crate::query::{query_config, query_collection, query_all_collections, query_artist_collections,query_is_symbol_available};
 
 const CONTRACT_NAME: &str = "crates.io:loop-factory";
 const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
 
 #[cfg_attr(not(feature = "library"), entry_point)]
 pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
 ) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.house_percentage + msg.artist_percentage != 100 {
        return Err(ContractError::InvalidRoyalties {});
    }
 
    let config = Config {
        admin: info.sender,
        duration: msg.duration,
        grace_period: msg.grace_period,
        payment_address: msg.payment_address,
        price: msg.price,
        nft_code_id: msg.nft_code_id,
        house_percentage: msg.house_percentage,
        artist_percentage: msg.artist_percentage
        
    };
    CONFIG.save(deps.storage, &config)?;
    COLLECTION_COUNT.save(deps.storage, &0u64)?;
 
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", config.admin.to_string())
        .add_attribute("nft_code_id", msg.nft_code_id.to_string()))
 }
 
 #[cfg_attr(not(feature = "library"), entry_point)]
 pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
 ) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateCollection { 
            name, 
            symbol,
            artist,
            minter, 
            collection_info
        } => create_collection(deps, env, info, name, symbol, artist, minter, collection_info),
        
        ExecuteMsg::UpdateNftCodeId { code_id } => 
            update_nft_code_id(deps, info, code_id),

        ExecuteMsg::UpdateRoyalties { house_percentage, artist_percentage } => 
            update_royalties(deps, info, house_percentage, artist_percentage)
    }
 }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => 
            to_json_binary(&query_config(deps)?),
        QueryMsg::Collection { artist } => 
            to_json_binary(&query_collection(deps, artist)?),
        QueryMsg::ArtistCollections { artist, limit } =>
            to_json_binary(&query_artist_collections(deps, artist, limit)?),
        QueryMsg::AllCollections { limit } =>
            to_json_binary(&query_all_collections(deps, limit)?),
        QueryMsg::IsSymbolAvailable { symbol } => 
            to_json_binary(&query_is_symbol_available(deps, symbol)?),
    }
}

 #[cfg_attr(not(feature = "library"), entry_point)]
 pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => reply_collection_created(deps, msg),
        id => Err(ContractError::UnknownReplyId { id }),
    }
 }