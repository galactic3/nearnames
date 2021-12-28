mod api_lot;
mod fraction;
mod lot;
mod profile;
mod utils;
mod economics;

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
pub use crate::fraction::*;
pub use crate::lot::*;
pub use crate::profile::*;
pub use crate::utils::*;
pub use crate::economics::*;

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
    use near_sdk_sim::{to_ts, to_yocto};

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

    fn get_context_with_payer(
        profile_id: &ProfileId,
        attached_deposit: Balance,
        timestamp: Timestamp,
    ) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(profile_id.clone())
            .is_view(false)
            .attached_deposit(attached_deposit)
            .block_timestamp(timestamp)
            .build()
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
    fn profile_get_missing() {
        let context = get_context_simple(false);
        testing_env!(context);
        let contract = build_contract();

        let profile = contract.profile_get("alice".parse().unwrap());

        assert_eq!(
            profile.profile_id,
            "alice".parse().unwrap(),
            "Expected profile_id alice"
        );
        assert_eq!(
            profile.rewards_available.0, 0,
            "Expected zero rewards_available"
        );
        assert_eq!(
            profile.rewards_claimed.0, 0,
            "Expected zero rewards_claimed"
        );
    }

    #[test]
    fn profile_get_present() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();

        let rewards_available: u128 = to_yocto("2");
        let rewards_claimed: u128 = to_yocto("3");
        // TODO: make params constructor private
        let mut profile: Profile = contract.internal_profile_extract(&"alice".parse().unwrap());
        profile.rewards_available = rewards_available;
        profile.rewards_claimed = rewards_claimed;
        contract.internal_profile_save(&profile);

        let response: ProfileView = contract.profile_get("alice".parse().unwrap());
        assert_eq!(
            response.rewards_available,
            rewards_available.into(),
            "rewards_available mismatch"
        );
        assert_eq!(
            response.rewards_claimed,
            rewards_claimed.into(),
            "rewards_claimed mismatch"
        );
    }

    pub fn create_lot_x_sells_y_api(
        contract: &mut Contract,
        seller_id: &ProfileId,
        lot_id: &LotId,
    ) -> Lot {
        let context = get_context_pred_x(&lot_id, false);
        testing_env!(context);

        let reserve_price = to_yocto("2");
        let buy_now_price = to_yocto("10");
        let finish_timestamp = to_ts(17);

        contract.lot_offer(
            seller_id.clone(),
            reserve_price.into(),
            buy_now_price.into(),
            Some(WrappedTimestamp::from(finish_timestamp)),
            None,
        );

        contract.lots.get(&lot_id).unwrap()
    }

    fn create_lot_bob_sells_alice_api(contract: &mut Contract) -> Lot {
        let lot_id: ProfileId = "alice".parse().unwrap();
        let seller_id: ProfileId = "bob".parse().unwrap();

        create_lot_x_sells_y_api(contract, &seller_id, &lot_id)
    }

    pub fn api_lot_bid(contract: &mut Contract, lot_id: &LotId, bid: &Bid) {
        let context = get_context_with_payer(&bid.bidder_id, bid.amount, bid.timestamp);
        testing_env!(context);
        contract.lot_bid(lot_id.clone());
    }

    #[test]
    fn internal_transfer() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        let profile_id: ProfileId = "alice".parse().unwrap();
        contract.internal_profile_rewards_transfer(&profile_id, to_yocto("3"));
        let profile = contract.profiles.get(&profile_id);
        assert!(profile.is_some());
        let profile = profile.unwrap();

        assert_eq!(profile.rewards_available, to_yocto("3"));

        contract.internal_profile_rewards_transfer(&profile_id, to_yocto("2"));
        assert_eq!(contract.profiles.len(), 1);
        let profile = contract.profiles.get(&profile_id);
        assert!(profile.is_some());
        let profile = profile.unwrap();

        assert_eq!(
            profile.rewards_available,
            to_yocto("5"),
            "expected balance 5 near after two transfers"
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

    #[test]
    pub fn profile_rewards_claim_success() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        let profile_id: ProfileId = "alice".parse().unwrap();
        let amount = MIN_PROFILE_REWARDS_CLAIM_AMOUNT;
        contract.internal_profile_rewards_transfer(&profile_id, amount);

        let context = get_context_pred_alice(false);
        testing_env!(context);
        contract.profile_rewards_claim();
        let result = contract.profile_get(profile_id.clone());
        assert_eq!(
            result.rewards_available.0,
            to_yocto("0"),
            "Expected rewards_available 0 after claim"
        );
        assert_eq!(
            result.rewards_claimed.0, amount,
            "Expected rewards_claimed amount after claim"
        );
    }

    #[test]
    pub fn test_profile_lots_offering_bidding() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();

        create_lot_bob_sells_alice_api(&mut contract);

        {
            let profile = contract.profiles.get(&"bob".parse().unwrap()).unwrap();
            let expected_lots_offering: Vec<LotId> = vec!["alice".parse().unwrap()];
            assert_eq!(
                profile.lots_offering.to_vec(),
                expected_lots_offering,
                "must be present after offer",
            );
            assert!(profile.lots_bidding.is_empty(), "must be empty for seller");
        }

        api_lot_bid(
            &mut contract,
            &"alice".parse().unwrap(),
            &Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("7"),
                timestamp: to_ts(10),
            },
        );

        {
            let profile = contract.profiles.get(&"carol".parse().unwrap()).unwrap();
            let expected_lots_bidding: Vec<LotId> = vec!["alice".parse().unwrap()];
            assert_eq!(
                profile.lots_bidding.to_vec(),
                expected_lots_bidding,
                "alice must be present after lot bid",
            );
            assert_eq!(
                profile.lots_offering.to_vec(),
                vec![],
                "must be empty for bidder"
            );
        }
    }

    #[test]
    pub fn test_profile_lots_bidding_api() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();

        create_lot_bob_sells_alice_api(&mut contract);
        // create another lot to be filtered out
        create_lot_x_sells_y_api(
            &mut contract,
            &"seller_1".parse().unwrap(),
            &"lot_1".parse().unwrap(),
        );

        api_lot_bid(
            &mut contract,
            &"alice".parse().unwrap(),
            &Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("7"),
                timestamp: to_ts(10),
            },
        );

        {
            let profile = contract.profiles.get(&"bob".parse().unwrap()).unwrap();
            let expected_lots_offering: Vec<LotId> = vec!["alice".parse().unwrap()];
            assert_eq!(
                profile.lots_offering.to_vec(),
                expected_lots_offering,
                "must be present after offer",
            );
            assert!(profile.lots_bidding.is_empty(), "must be empty for seller");
        }

        {
            let profile = contract.profiles.get(&"carol".parse().unwrap()).unwrap();
            let expected_lots_bidding: Vec<LotId> = vec!["alice".parse().unwrap()];
            assert_eq!(
                profile.lots_bidding.to_vec(),
                expected_lots_bidding,
                "alice must be present after lot bid",
            );
            assert_eq!(
                profile.lots_offering.to_vec(),
                vec![],
                "must be empty for bidder"
            );
        }

        {
            let result = contract.lot_list_offering_by("bob".parse().unwrap(), None, None);
            assert_eq!(result.len(), 1, "lot_offering must contain 1 lot");
            let result = result.get(0).unwrap();
            assert_eq!(
                &result.lot_id,
                &"alice".parse().unwrap(),
                "expected alice in offering lot list"
            );
            assert_eq!(
                result.last_bidder_id,
                Some("carol".parse().unwrap()),
                "expected carol last_bidder",
            );
            assert_eq!(
                result.status,
                "OnSale".to_string(),
                "expected status on sale",
            );

            let result = contract.lot_list_offering_by("carol".parse().unwrap(), None, None);
            assert!(result.is_empty(), "lot_offering for carol must be empty");
        }

        {
            let result = contract.lot_list_bidding_by("carol".parse().unwrap(), None, None);
            assert_eq!(result.len(), 1, "lot_bidding must contain 1 lot");
            let result = result.get(0).unwrap();
            assert_eq!(
                &result.lot_id,
                &"alice".parse().unwrap(),
                "expected alice in bidding lot list"
            );
            assert_eq!(
                result.last_bidder_id,
                Some("carol".parse().unwrap()),
                "expected bob as profile_role"
            );
            assert_eq!(
                result.status,
                "OnSale".to_string(),
                "expected status on sale",
            );

            let result = contract.lot_list_bidding_by("bob".parse().unwrap(), None, None);
            assert!(result.is_empty(), "lot_bidding for bob must be empty");
        }
    }
}
