use cosmwasm_std::{
  to_json_binary, Addr, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg};
use pass_nft::msg::InstantiateMsg as NftInstantiateMsg;

use crate::error::ContractError;
use crate::state::{Collection, COLLECTIONS, COLLECTION_BY_SYMBOL, COLLECTION_COUNT, CONFIG};

pub fn create_collection(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  name: String,
  symbol: String,
  pass_price: u128,
  pass_duration: u64,
  grace_period: u64,
  payment_address: String,
) -> Result<Response, ContractError> {

  if !symbol.chars().all(char::is_uppercase) || symbol.contains(' ') {
      return Err(ContractError::InvalidSymbol {});
  }

  if COLLECTION_BY_SYMBOL.has(deps.storage, symbol.clone()) {
      return Err(ContractError::SymbolAlreadyTaken {});
  }

  let payment_addr = deps.api.addr_validate(&payment_address)?;

  let config = CONFIG.load(deps.storage)?;

  // Create instantiate message for NFT contract
  let instantiate_msg = to_json_binary(&NftInstantiateMsg {
      name: name.clone(),
      symbol: symbol.clone(),
      artist: info.sender.to_string(),
      pass_price,
      pass_duration,
      grace_period,
      payment_address: payment_addr,
  })?;

  // Create SubMsg to instantiate NFT contract
  let sub_msg = SubMsg::reply_on_success(
      WasmMsg::Instantiate {
          admin: Some(info.sender.to_string()),
          code_id: config.nft_code_id,
          msg: instantiate_msg,
          funds: vec![],
          label: format!("{} Collection", name),
      },
      1,
  );

  let collection = Collection {
      name: name.clone(),
      symbol: symbol.clone(),
      artist: info.sender.clone(),
      contract_address: Addr::unchecked(""),
      created_at: env.block.time.seconds(),
  };

  COLLECTIONS.save(deps.storage, &info.sender, &collection)?;

  COLLECTION_COUNT.update(deps.storage, |count| -> Result<u64, ContractError> {
      Ok(count + 1)
  })?;

  Ok(Response::new()
      .add_submessage(sub_msg)
      .add_attribute("action", "create_collection")
      .add_attribute("artist", info.sender)
      .add_attribute("name", name)
      .add_attribute("symbol", symbol))
}

pub fn update_nft_code_id(
  deps: DepsMut,
  info: MessageInfo,
  code_id: u64,
) -> Result<Response, ContractError> {
  // Load and verify owner
  let mut config = CONFIG.load(deps.storage)?;
  if info.sender != config.owner {
      return Err(ContractError::Unauthorized {});
  }

  // Update code ID
  config.nft_code_id = code_id;
  CONFIG.save(deps.storage, &config)?;

  Ok(Response::new()
      .add_attribute("action", "update_nft_code_id")
      .add_attribute("new_code_id", code_id.to_string()))
}