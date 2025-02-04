use cosmwasm_std::{
    to_json_binary, Addr, DepsMut, Env, MessageInfo, Reply, Response, SubMsg, WasmMsg
 };
use pass_nft::msg::InstantiateMsg as NftInstantiateMsg;
 use cw_utils::parse_reply_instantiate_data;
 
 use crate::error::ContractError;
 use crate::state::{Collection, CONFIG, COLLECTIONS, SYMBOL_TAKEN, save_new_collection};
 use crate::msg::CollectionCreatedEvent;
 
 pub fn create_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    symbol: String,
    artist: Addr,
    minter: Addr,
    collection_info: String,
 ) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

if !symbol.chars().all(char::is_uppercase) {
    return Err(ContractError::InvalidSymbol {});
}
 
    if SYMBOL_TAKEN.may_load(deps.storage, symbol.clone())?.unwrap_or(false) {
        return Err(ContractError::SymbolAlreadyTaken {});
    }
 
    let instantiate_msg = to_json_binary(&NftInstantiateMsg {
        name: name.clone(),
        pass_duration: config.duration,
        symbol: symbol.clone(),
        collection_info: collection_info.clone(),
        minter: minter.clone(),
        pass_price: config.price,
        grace_period: config.grace_period,
        payment_address: config.payment_address,
        artist: artist.clone(),
    })?;
 
    let sub_msg = SubMsg::reply_on_success(
        WasmMsg::Instantiate {
            admin: Some(config.admin.to_string()),
            code_id: config.nft_code_id,
            msg: instantiate_msg,
            funds: vec![],
            label: format!("{} Collection", name),
        },
        1,
    );

    let created_at = env.block.time.seconds();
 
    let collection = Collection::new(
        name.clone(),
        symbol.clone(),
        artist.clone(),
        minter.clone(),
        Addr::unchecked(""),
        created_at,
        collection_info,
        
    );
 
    save_new_collection(deps.storage, &collection)?;
 
    let event = CollectionCreatedEvent {
        name,
        symbol,
        artist,
        minter,
        contract_address: Addr::unchecked(""),
    };
 
    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "create_collection")
        .add_attribute("event", to_json_binary(&event)?.to_string()))
 }
 
 pub fn update_nft_code_id(
    deps: DepsMut,
    info: MessageInfo,
    code_id: u64,
 ) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
 
    config.nft_code_id = code_id;
    CONFIG.save(deps.storage, &config)?;
 
    Ok(Response::new()
        .add_attribute("action", "update_nft_code_id")
        .add_attribute("new_code_id", code_id.to_string()))
 }
 
 pub fn reply_collection_created(
    deps: DepsMut,
    reply: Reply,
) -> Result<Response, ContractError> {
    let res = parse_reply_instantiate_data(reply).unwrap();
    let contract_addr = deps.api.addr_validate(&res.contract_address)?;

    let mut found_symbol = String::new();
    COLLECTIONS.range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .find(|item| {
            if let Ok((symbol, collection)) = item {
                if collection.contract_address.as_str().is_empty() {
                    found_symbol = symbol.to_string();
                    true
                } else {
                    false
                }
            } else {
                false
            }
        });

    if found_symbol.is_empty() {
        return Err(ContractError::CollectionNotFound {});
    }

    COLLECTIONS.update(deps.storage, found_symbol.clone(), |col| match col {
        Some(mut collection) => {
            collection.contract_address = contract_addr.clone();
            Ok(collection)
        }
        None => Err(ContractError::CollectionNotFound {}),
    })?;

    Ok(Response::new()
        .add_attribute("action", "collection_created")
        .add_attribute("symbol", found_symbol)
        .add_attribute("contract_address", contract_addr.to_string()))
}