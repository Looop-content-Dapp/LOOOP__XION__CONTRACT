
#[cfg(not(feature = "library"))]

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw721_base_soulbound::state::TokenInfo;
use cosmwasm_std::{Coin, Uint128, BankMsg};

// use cw721_base_soulbound::ExecuteMsg::Mint;

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
 
    // Get and increment token ID
    let current_token_id = TOKEN_ID_COUNTER.load(deps.storage)?;

    let next_id = current_token_id + 1;
    TOKEN_ID_COUNTER.save(deps.storage, &next_id)?;
  

    let token_id = format!("{}-{}", config.symbol.to_lowercase(), next_id);
  
    // Create new pass extension
    let extension = PassExtension::new(
        env.block.time,
        config.pass_duration,
        config.grace_period,
    );

    let contract = Contract::default();
    
    // First check if token exists
    if contract.tokens.may_load(deps.storage, &token_id)?.is_some() {
        deps.api.debug("Token ID already exists");
        return Err(ContractError::Custom("Token ID already exists".to_string()));
    }

    // Create token directly
    let token = TokenInfo {
        owner: deps.api.addr_validate(&owner_address)?,
        approvals: vec![],
        token_uri: Some(config.collection_info),
        extension,
    };

    // Save token directly
    contract.tokens.save(deps.storage, &token_id, &token)?;
  
    // Increment token count
    contract.increment_tokens(deps.storage)?;
   
    // v.0.1.0 create the royalty split code here. 70% for artist and 30% for loop.

    let payment = info.funds[0].amount.u128();

    let house_amount = (payment * config.house_percentage as u128) / 100u128;

    let artist_amount = payment - house_amount;

    // Create royalty split payment message
    let house_payment_msg =  BankMsg::Send {
        to_address: config.payment_address.to_string(),
        amount: vec![Coin {
            denom: info.funds[0].denom.clone(),
            amount: Uint128::from(house_amount),
        }],
    };

    let artist_payment_msg =  BankMsg::Send { 
        to_address: config.artist.to_string(),
        amount: vec![Coin {
            denom: info.funds[0].denom.clone(),
            amount: Uint128::from(artist_amount),
        }]
    };

    deps.api.debug("Returning successful response");
    Ok(Response::new()
        .add_message(house_payment_msg)
        .add_message(artist_payment_msg)
        .add_attribute("action", "mint_pass")
        .add_attribute("collection", config.name)
        .add_attribute("artist", config.artist)
        .add_attribute("minter", info.sender)
        .add_attribute("token_id", token_id))
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

    // royalty split after renewal 

    let payment = info.funds[0].amount.u128();

    let house_amount = (payment * config.house_percentage as u128) / 100u128;

    let artist_amount = payment - house_amount;

    // Create royalty split payment message
    let house_payment_msg =  BankMsg::Send {
        to_address: config.payment_address.to_string(),
        amount: vec![Coin {
            denom: info.funds[0].denom.clone(),
            amount: Uint128::from(house_amount),
        }],
    };

    let artist_payment_msg =  BankMsg::Send { 
        to_address: config.artist.to_string(),
        amount: vec![Coin {
            denom: info.funds[0].denom.clone(),
            amount: Uint128::from(artist_amount),
        }]
    };

    Ok(Response::new()
        .add_message(house_payment_msg)
        .add_message(artist_payment_msg)
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