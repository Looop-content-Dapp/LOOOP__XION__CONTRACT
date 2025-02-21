#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError, to_json_binary};
use cw2::set_contract_version;



use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, PassMsg};
use crate::state::{CONFIG, Config, TOKEN_ID_COUNTER};
use crate::execute::{mint_pass, renew_pass, burn_expired_pass};
use crate::query::{query_config, query_validity, query_artist_info, get_user_pass};
use crate::msg::PassQuery;
use crate::state::Contract;
use crate::helpers::convert_query_msg;

// Version info for migration info
const CONTRACT_NAME: &str = "crates.io:loop_music";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let payment_address = deps.api.addr_validate(&msg.payment_address.to_string())?;
    let artist = deps.api.addr_validate(&msg.artist.to_string())?;

    let collection_name = msg.name;
    let collection_symbol = msg.symbol;

    if msg.house_percentage + msg.artist_percentage != 100 {
        return Err(ContractError::InvalidRoyalties {});
    }

    let config = Config {
        name: collection_name.clone(),
        symbol: collection_symbol.clone(),
        pass_price: msg.pass_price,
        collection_info: msg.collection_info,
        minter: msg.minter,
        pass_duration: msg.pass_duration,
        grace_period: msg.grace_period,
        payment_address: payment_address.clone(),
        artist: artist.clone(),
        house_percentage: msg.house_percentage,
        artist_percentage: msg.artist_percentage,
        
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

     // Initialize token ID counter
     TOKEN_ID_COUNTER.save(deps.storage, &0u64)?;

    // Initialize the base CW721 contract
    let contract = Contract::default();
    let cw721_msg = cw721_base_soulbound::InstantiateMsg {
        name: config.name,
        symbol: config.symbol,
        minter: config.minter.to_string(),
    };
    contract.instantiate(deps, env, info, cw721_msg)?;

      

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("collection_name", collection_name)
        .add_attribute("collection_symbol", collection_symbol)
        .add_attribute("artist", artist)
        .add_attribute("payment_address", payment_address)
        .add_attribute("house_percentage", msg.house_percentage.to_string())
        .add_attribute("artist_percentage", msg.artist_percentage.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Extension { msg } => match msg {
            PassMsg::MintPass { owner_address }
             => {
                deps.api.debug("Executing mint_pass");
                mint_pass(deps, env, info, owner_address)},
            PassMsg::RenewPass { token_id } => renew_pass(deps, env, info, token_id),
            PassMsg::BurnExpiredPass { token_id } => burn_expired_pass(deps, env, info, token_id),
        },
        _ => Err(ContractError::Custom("Unsupported operation".to_string())),
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: cw721_base_soulbound::QueryMsg<PassQuery>,
) -> StdResult<Binary> {
    let contract = Contract::default();
    
    match msg {
        QueryMsg::Extension { msg } => match msg {
            PassQuery::CheckValidity { token_id } => to_json_binary(&query_validity(deps, env, token_id)?),
            PassQuery::GetConfig {} => to_json_binary(&query_config(deps)?),
            PassQuery::GetArtistInfo {} => to_json_binary(&query_artist_info(deps)?),
            PassQuery::GetUserPass { symbol, owner } => { 
                to_json_binary(&get_user_pass(deps, env, symbol, owner)?)
            }
        },
        base_query => {
            let converted_msg = convert_query_msg(base_query)
                .map_err(|e| StdError::generic_err(format!("{}", e)))?;
            contract.query(deps, env, converted_msg)
        }
    }
}

