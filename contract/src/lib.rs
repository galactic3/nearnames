mod api_lot;
mod api_profile;
mod contract;
mod economics;
mod fraction;
mod lot;
mod profile;
mod utils;

use std::collections::HashSet;
use std::fmt;
use std::ops;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json;
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, Duration, PanicOnDefault, Promise,
    PromiseResult, PublicKey, Timestamp,
};
use uint::construct_uint;

pub use crate::api_lot::*;
pub use crate::api_profile::*;
pub use crate::contract::*;
pub use crate::economics::*;
pub use crate::fraction::*;
pub use crate::lot::*;
pub use crate::profile::*;
pub use crate::utils::*;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

pub type LotId = AccountId;
pub type ProfileId = AccountId;
pub type WrappedBalance = U128;
pub type WrappedTimestamp = U64;
pub type WrappedDuration = U64;

pub const PREFIX_PROFILES: &str = "u";
pub const PREFIX_LOTS: &str = "a";
pub const PREFIX_LOTS_BIDS: &str = "y";
pub const PREFIX_PROFILE_LOTS_BIDDING: &str = "b";
pub const PREFIX_PROFILE_LOTS_OFFERING: &str = "f";

#[ext_contract]
pub trait ExtLockContract {
    fn unlock(&mut self, public_key: PublicKey);
    fn get_owner(&self) -> AccountId;
}

#[ext_contract]
pub trait ExtSelfContract {
    fn lot_after_claim_clean_up(&mut self, lot_id: LotId);
    fn lot_after_remove_unsafe_remove(&mut self, lot_id: LotId);
    fn profile_after_rewards_claim(&mut self, profile_id: ProfileId, rewards: Balance);
}

#[cfg(test)]
mod tests;
