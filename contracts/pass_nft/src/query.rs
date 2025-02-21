use cosmwasm_std::{Deps, Env, StdResult};
use crate::msg::{ ValidityResponse, ConfigResponse,PassResponse };
use crate::state::{Contract, CONFIG, PassStatus};
use crate::msg::ArtistInfoResponse;
use cosmwasm_std::StdError;
use cosmwasm_std::Order;


pub fn get_user_pass(
    deps: Deps,
    env: Env,
    symbol: String,
    owner: String,
) -> StdResult<PassResponse> {
    let contract = Contract::default();
    let config = CONFIG.load(deps.storage)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    // Format the expected token_id pattern
    let token_id_prefix = format!("{}-", symbol.to_lowercase());
    
    // Query tokens by owner
    let tokens: Vec<String> = contract.tokens
        .idx
        .owner
        .prefix(owner_addr.clone())
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    // Find the token that matches our symbol
    let matching_token = tokens
        .into_iter()
        .find(|token_id| token_id.starts_with(&token_id_prefix))
        .ok_or_else(|| StdError::not_found("No pass found for this symbol and owner"))?;

    // Get token details
    let token = contract.tokens.load(deps.storage, &matching_token)?;
    let status = token.extension.status(env.block.time);
    let is_valid = matches!(status, PassStatus::Active | PassStatus::InGracePeriod);

    Ok(PassResponse {
        collection_name: config.name,
        contract_address: env.contract.address,
        owner: owner_addr.to_string(),
        token_id: matching_token,
        is_valid,
        expires_at: token.extension.expires_at,
    })
}


//check if token is still valid, not in grace period 
pub fn query_validity(deps: Deps, env: Env, token_id: String) -> StdResult<ValidityResponse> {
    let contract = Contract::default();
    let token = contract.tokens.load(deps.storage, &token_id)?;
    
    let status = token.extension.status(env.block.time);
    let is_valid = matches!(status, PassStatus::Active | PassStatus::InGracePeriod);
    
    Ok(ValidityResponse {
        token_id,
        is_valid,
        expires_at: token.extension.expires_at,
        in_grace_period: matches!(status, PassStatus::InGracePeriod),
        grace_period_end: if is_valid { Some(token.extension.grace_period_end) } else { None },
    })
}

// New query handler for artist info
pub fn query_artist_info(deps: Deps) -> StdResult<ArtistInfoResponse> {
    let config = CONFIG.load(deps.storage)?;
    let contract = Contract::default();
    let total_tokens = contract.token_count(deps.storage)?;
    
    // Count active passes
    let active_passes = contract.tokens
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|item| {
            if let Ok((_, token)) = item {
                token.extension.is_active
            } else {
                false
            }
        })
        .count() as u64;

    Ok(ArtistInfoResponse {
        artist: config.artist.to_string(),
        total_passes: total_tokens,
        active_passes,
    })
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        name: config.name,
        symbol: config.symbol,
        collection_info: config.collection_info,
        artist: config.artist,
        minter: config.minter,
        pass_price: config.pass_price,
        pass_duration: config.pass_duration,
        grace_period: config.grace_period,
        payment_address: config.payment_address,
        house_percentage: config.house_percentage,
        artist_percentage: config.artist_percentage
        
    })
}
