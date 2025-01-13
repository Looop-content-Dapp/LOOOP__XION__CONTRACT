use cosmwasm_std::{
    entry_point, to_json_binary, Reply, SubMsgResponse, SubMsgResult, Binary, Deps, DepsMut, Env, MessageInfo, 
    Response, StdResult, Addr,
    Order,
};

use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, COLLECTIONS, COLLECTION_COUNT, COLLECTION_BY_SYMBOL};
use crate::execute::create_collection;
use crate::query::{query_config, query_collection, query_collection_by_symbol};

// Constants
const CONTRACT_NAME: &str = "crates.io:factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Set contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Initialize config
    let config = Config {
        nft_code_id: msg.nft_code_id,
        owner: info.sender.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    // Initialize collection count
    COLLECTION_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
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
            pass_price,
            pass_duration,
            grace_period,
            payment_address,
        } => create_collection(
            deps,
            env,
            info,
            name,
            symbol,
            pass_price,
            pass_duration,
            grace_period,
            payment_address.to_string(),
        ),
        ExecuteMsg::UpdateNftCodeId { code_id } => {
            let config = CONFIG.load(deps.storage)?;
            if info.sender != config.owner {
                return Err(ContractError::Unauthorized {});
            }
            CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
                config.nft_code_id = code_id;
                Ok(config)
            })?;
            Ok(Response::new()
                .add_attribute("action", "update_nft_code_id")
                .add_attribute("new_code_id", code_id.to_string()))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Collection { artist } => to_json_binary(&query_collection(deps, artist)?),
        QueryMsg::CollectionBySymbol { symbol } => to_json_binary(&query_collection_by_symbol(deps, symbol)?),
       
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => handle_instantiate_reply(deps, msg),
        id => Err(ContractError::UnknownReplyId { id }),
    }
}

fn handle_instantiate_reply(deps: DepsMut, msg: Reply) -> Result<Response, ContractError> {
    let SubMsgResult::Ok(SubMsgResponse { data: Some(data), .. }) = msg.result 
        else { return Err(ContractError::InvalidInstantiation {}) };
    
    // Parse the data to get contract address
    let contract_addr = String::from_utf8(data.to_vec())
        .map_err(|_| ContractError::InvalidInstantiation {})?;
    let contract_addr = Addr::unchecked(contract_addr);

    // Find the collection without an address and update it
    let collections: Vec<_> = COLLECTIONS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    if let Some((artist_addr, mut collection)) = collections
        .into_iter()
        .find(|(_, col)| col.contract_address == Addr::unchecked(""))
    {
        collection.contract_address = contract_addr.clone();
        COLLECTIONS.save(deps.storage, &artist_addr, &collection)?;
        
        COLLECTION_BY_SYMBOL.save(deps.storage, collection.symbol, &contract_addr)?;

        Ok(Response::new()
            .add_attribute("contract_addr", contract_addr)
            .add_attribute("collection", collection.name))
    } else {
        Err(ContractError::CollectionNotFound {})
    }
}