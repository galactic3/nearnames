mod api_lot;
mod api_profile;
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
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, Balance, Duration, PanicOnDefault, Promise,
    PromiseResult, PublicKey, Timestamp,
};
use uint::construct_uint;

pub use crate::api_lot::*;
pub use crate::api_profile::*;
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
}

#[ext_contract]
pub trait ExtSelfContract {
    fn lot_after_claim_clean_up(&mut self, lot_id: LotId);
    fn profile_after_rewards_claim(&mut self, profile_id: ProfileId, rewards: Balance);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub profiles: UnorderedMap<ProfileId, Profile>,
    pub lots: UnorderedMap<LotId, Lot>,
    pub seller_rewards_commission: Fraction,
    pub bid_step: Fraction,
    pub prev_bidder_commission_share: Fraction,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractConfigView {
    pub seller_rewards_commission: FractionView,
    pub bid_step: FractionView,
    pub prev_bidder_commission_share: FractionView,
}

impl From<&Contract> for ContractConfigView {
    fn from(contract: &Contract) -> ContractConfigView {
        ContractConfigView {
            seller_rewards_commission: (&contract.seller_rewards_commission).into(),
            bid_step: (&contract.bid_step).into(),
            prev_bidder_commission_share: (&contract.prev_bidder_commission_share).into(),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn config_get(&self) -> ContractConfigView {
        self.into()
    }

    #[init(ignore_state)]
    pub fn new(
        seller_rewards_commission: FractionView,
        bid_step: FractionView,
        prev_bidder_commission_share: FractionView,
    ) -> Self {
        Self {
            profiles: UnorderedMap::new(PREFIX_PROFILES.as_bytes().to_vec()),
            lots: UnorderedMap::new(PREFIX_LOTS.as_bytes().to_vec()),
            seller_rewards_commission: Fraction::new(
                seller_rewards_commission.num,
                seller_rewards_commission.denom,
            ),
            bid_step: Fraction::new(bid_step.num, bid_step.denom),
            prev_bidder_commission_share: Fraction::new(
                prev_bidder_commission_share.num,
                prev_bidder_commission_share.denom,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    use near_sdk_sim::to_ts;

    fn get_context_simple(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("alice_near".parse().unwrap())
            .is_view(is_view)
            .block_timestamp(to_ts(13))
            .build()
    }

    fn get_context_pred_x(profile_id: &ProfileId, is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(profile_id.clone())
            .is_view(is_view)
            .block_timestamp(to_ts(10))
            .build()
    }

    fn get_context_pred_alice(is_view: bool) -> VMContext {
        get_context_pred_x(&"alice".parse().unwrap(), is_view)
    }

    pub fn build_contract() -> Contract {
        Contract::new(
            FractionView { num: 1, denom: 10 },
            FractionView { num: 1, denom: 5 },
            FractionView { num: 4, denom: 5 },
        )
    }

    #[test]
    fn contract_config_get() {
        let context = get_context_simple(false);
        testing_env!(context);
        let contract = build_contract();

        let config = contract.config_get();
        assert_eq!(
            config.seller_rewards_commission,
            FractionView { num: 1, denom: 10 },
            "wrong seller rewards commission",
        );
        assert_eq!(
            config.bid_step,
            FractionView { num: 1, denom: 5 },
            "wrong bid_step",
        );
        assert_eq!(
            config.prev_bidder_commission_share,
            FractionView { num: 4, denom: 5 },
            "wrong seller rewards commission",
        );
    }

    #[test]
    #[should_panic(expected = "Not enough rewards for transfer")]
    pub fn profile_rewards_claim_fail_below_threshold() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        let profile_id: ProfileId = "alice".parse().unwrap();
        let amount = MIN_PROFILE_REWARDS_CLAIM_AMOUNT - 1;

        contract.internal_profile_rewards_transfer(&profile_id, amount);

        let context = get_context_pred_alice(false);
        testing_env!(context);

        contract.profile_rewards_claim();
    }
}
