use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

// Import Custom Message
use crate::msg::PassMsg;


// Define the main configuration item
pub const CONFIG: Item<Config> = Item::new("config");

// Define metadata for NFTs
#[cw_serde]
pub struct PassExtension {
    pub expires_at: Timestamp,
    pub is_active: bool,
    pub grace_period_end: Timestamp,
    pub times_renewed: u32,
}

pub type Contract<'a> = cw721_base_soulbound::Cw721Contract<'a, PassExtension, PassMsg, PassMsg, PassMsg>;


/// Contract configuration
#[cw_serde]
pub struct Config {
    pub name: String, // Collection name
    pub symbol: String, // Collection symbol
    pub artist: Addr,   // Artist's address
    pub pass_price: u128, // Cost to mint/renew a pass in uxion
    pub pass_duration: u64,// Duration of the pass in seconds
    pub grace_period: u64, // Grace period after expiry
    pub payment_address: Addr, // Address receiving payments
}

/// Additional helpers for managing PassExtension logic
impl PassExtension {
    /// Create a new pass with proper timestamps
    pub fn new(current_time: Timestamp, pass_duration: u64, grace_period: u64) -> Self {
        let expires_at = current_time.plus_seconds(pass_duration);
        let grace_period_end = expires_at.plus_seconds(grace_period);

        Self {
            expires_at,
            is_active: true,
            grace_period_end,
            times_renewed: 0,
        }
    }

    /// Check pass status
    pub fn status(&self, current_time: Timestamp) -> PassStatus {
        if current_time < self.expires_at {
            PassStatus::Active
        } else if current_time <= self.grace_period_end {
            PassStatus::InGracePeriod
        } else {
            PassStatus::Expired
        }
    }

    /// Handle renewal of a pass
    pub fn renew(&mut self, current_time: Timestamp, pass_duration: u64, grace_period: u64) {
        self.expires_at = current_time.plus_seconds(pass_duration);
        self.grace_period_end = self.expires_at.plus_seconds(grace_period);
        self.is_active = true;
        self.times_renewed += 1;
    }
}

/// Enum to represent pass status
#[cw_serde]
pub enum PassStatus {
    Active,
    InGracePeriod,
    Expired,
}


pub const TOKENS_BY_OWNER: Map<&Addr, Vec<String>> = Map::new("tokens_by_owner");
