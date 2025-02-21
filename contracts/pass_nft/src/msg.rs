use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Addr};
use crate::state::PassExtension;
use cw721_base_soulbound::CustomMsg;

// Custom Instantiate message for contract
#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub artist: Addr,
    pub minter: Addr,
    pub collection_info: String,
    pub pass_price: u128,
    pub pass_duration: u64,
    pub grace_period: u64,
    pub payment_address: Addr,

    pub house_percentage: u32,
    pub artist_percentage: u32,
}

// Custom Pass messages extending the base contract
#[cw_serde]
pub enum PassMsg {
    MintPass {owner_address: String},
    RenewPass { token_id: String },
    BurnExpiredPass { token_id: String },
}

impl CustomMsg for PassMsg {}

pub type ExecuteMsg = cw721_base_soulbound::ExecuteMsg<PassExtension, PassMsg>;

// Custom Pass Queries
#[cw_serde]
#[derive(QueryResponses)]
pub enum PassQuery {

#[returns(PassResponse)]
 GetUserPass {
symbol: String, 
owner: String
 },

#[returns(ValidityResponse)]
CheckValidity { token_id: String },

#[returns(ConfigResponse)]
GetConfig {},

#[returns(ArtistInfoResponse)] 
GetArtistInfo {},
}

pub type QueryMsg = cw721_base_soulbound::QueryMsg<PassQuery>;

// Custom query responses
#[cw_serde]
pub struct ValidityResponse {
    pub token_id: String,
    pub is_valid: bool,
    pub expires_at: Timestamp,
    pub in_grace_period: bool,
    pub grace_period_end: Option<Timestamp>,
}

#[cw_serde]
pub struct PassResponse {
    pub collection_name: String,
    pub contract_address: Addr,
    pub token_id: String,
    pub owner: String,
    pub is_valid: bool,
    pub expires_at: Timestamp,
}

#[cw_serde]
pub struct ConfigResponse {
    pub name: String, 
    pub symbol: String, 
    pub artist: Addr, 
    pub minter: Addr,
    pub collection_info: String, 
    pub pass_price: u128, 
    pub pass_duration: u64,
    pub grace_period: u64, 
    pub payment_address: Addr,
    pub house_percentage: u32,
    pub artist_percentage: u32, 
}

#[cw_serde]
pub struct ArtistInfoResponse {
    pub artist: String,
    pub total_passes: u64,
    pub active_passes: u64,
}
