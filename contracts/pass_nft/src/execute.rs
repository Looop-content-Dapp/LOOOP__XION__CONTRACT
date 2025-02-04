use cosmwasm_std::Addr;
#[cfg(not(feature = "library"))]

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw721_base_soulbound::state::TokenInfo;

use crate::error::ContractError;
use crate::state::{Contract, PassExtension, CONFIG, TOKEN_ID_COUNTER};
use crate::state::PassStatus;
use crate::helpers::validate_payment;
// use crate::msg::{ExecuteMsg, PassMsg};

pub fn mint_pass(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner_address: String,
) -> Result<Response, ContractError> {

    let config = CONFIG.load(deps.storage)?;

    // Validate payment
    validate_payment(&info, config.pass_price)?;

    // new fix: Do the token_id via contract
    let current_token_id = TOKEN_ID_COUNTER.load(deps.storage)?;

    // increment token_id to assign to user 
    let next_id = current_token_id + 1;

    TOKEN_ID_COUNTER.save(deps.storage, &next_id)?;

    let token_id = format!("{}-{}", config.symbol.to_lowercase(), next_id);

    // Create new pass extension with timestamps
    let extension = PassExtension::new(
        env.block.time,
        config.pass_duration,
        config.grace_period,
    );

    // Create token using base contract's functionality
    let contract = Contract::default();

    contract.tokens.update(deps.storage, &token_id, |old| match old {
    Some(_) => Err(ContractError::Custom("Token ID already exists".to_string())),
    None => Ok(TokenInfo {
            owner: Addr::unchecked(owner_address),
            approvals: vec![],
            token_uri: Some(config.collection_info),
            extension,
        }),
    })?;

    // Send payment artist's payment address
    let payment_msg = cosmwasm_std::BankMsg::Send {
        to_address: config.payment_address.to_string(),
        amount: info.funds,
    };

    // Increment token count
    contract.increment_tokens(deps.storage)?;

    Ok(Response::new()
    .add_message(payment_msg)
    .add_attribute("action", "mint_pass")
    .add_attribute("collection", config.name)
    .add_attribute("artist", config.artist)
    .add_attribute("minter", info.sender)
    .add_attribute("token_id", token_id)
)
}



pub fn renew_pass(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // Validate payment
    validate_payment(&info, config.pass_price)?;

    let contract = Contract::default();

    let mut token = contract.tokens.load(deps.storage, &token_id)?;

    // Allow both token owner and admin to renew
    if info.sender != token.owner && info.sender != config.minter {
        return Err(ContractError::Unauthorized {});
    }

    // Renew the pass
    token.extension.renew(
        env.block.time,
        config.pass_duration,
        config.grace_period,
    );

    // Save updated token
    contract.tokens.save(deps.storage, &token_id, &token)?;

    // Send renewal payment to artist
    let payment_msg = cosmwasm_std::BankMsg::Send {
        to_address: config.payment_address.to_string(),
        amount: info.funds,
    };

    Ok(Response::new()
        .add_message(payment_msg)
        .add_attribute("action", "renew_pass")
        .add_attribute("collection", config.name)
        .add_attribute("artist", config.artist)
        .add_attribute("token_id", token_id) 
        .add_attribute("owner", token.owner.to_string()) 
        .add_attribute("new_expiry", token.extension.expires_at.to_string()))
}




pub fn burn_expired_pass(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {

    let config = CONFIG.load(deps.storage)?;
    let contract = Contract::default();
    let token = contract.tokens.load(deps.storage, &token_id)?;

     // Allow both token owner and admin to burn
     if info.sender != token.owner && info.sender != config.minter {
        return Err(ContractError::Unauthorized {});
    }

    if token.extension.status(env.block.time) != PassStatus::Expired {
        return Err(ContractError::Custom("Pass is not expired".to_string()));
    }
  
    contract.tokens.remove(deps.storage, &token_id)?;
    contract.decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "burn_expired_pass")
        .add_attribute("collection", config.name)
        .add_attribute("artist", config.artist)
        .add_attribute("token_id", token_id)
        .add_attribute("owner", info.sender))
}