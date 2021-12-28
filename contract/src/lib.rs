mod api_lot;
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
    use std::convert::TryInto;

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

    #[test]
    #[should_panic]
    fn internal_lot_extract_missing() {
        let mut contract = build_contract();
        contract.internal_lot_extract(&"alice".parse().unwrap());
    }

    fn create_lot_bob_sells_alice() -> Lot {
        let reserve_price = to_yocto("5");
        let buy_now_price = to_yocto("10");

        let start_timestamp = to_ts(10);
        let finish_timestamp = to_ts(11);

        Lot::new(
            "alice".parse().unwrap(),
            "bob".parse().unwrap(),
            reserve_price,
            buy_now_price,
            start_timestamp,
            finish_timestamp,
        )
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

    #[test]
    fn internal_lot_create_save_extract() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        assert_eq!(
            contract.lots.len(),
            0,
            "{}",
            "expected contract.lots.len() == 0 after extract"
        );

        let lot_bob_sells_alice = create_lot_bob_sells_alice();
        contract.internal_lot_save(&lot_bob_sells_alice);
        assert_eq!(
            contract.lots.len(),
            1,
            "{}",
            "expected contract.lots.len() == 1"
        );

        let lot_found: Lot = contract.internal_lot_extract(&lot_bob_sells_alice.lot_id);
        assert_eq!(
            contract.lots.len(),
            0,
            "{}",
            "expected contract.lots.len() == 0 after extract"
        );

        assert_eq!(
            lot_found.lot_id,
            "alice".parse().unwrap(),
            "{}",
            "expected lot_id == alice"
        );
    }

    pub fn api_lot_bid(contract: &mut Contract, lot_id: &LotId, bid: &Bid) {
        let context = get_context_with_payer(&bid.bidder_id, bid.amount, bid.timestamp);
        testing_env!(context);
        contract.lot_bid(lot_id.clone());
    }

    fn internal_lot_bid(contract: &mut Contract, lot_id: &LotId, bid: &Bid) {
        let mut lot = contract.internal_lot_extract(lot_id);
        lot.place_bid(bid, contract.bid_step);
        contract.internal_lot_save(&lot);
    }

    #[test]
    fn api_lot_list_present_active() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        let lot_bob_sells_alice = create_lot_bob_sells_alice();
        contract.internal_lot_save(&lot_bob_sells_alice);

        let first_bid_amount = to_yocto("6");
        let next_bid_amount = first_bid_amount + contract.bid_step * first_bid_amount;

        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: first_bid_amount,
            timestamp: to_ts(10),
        };
        internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);

        let context = get_context_pred_alice(true);
        testing_env!(context);

        let response: Vec<LotView> = contract.lot_list(None, None);
        assert!(!response.is_empty());
        let response: &LotView = &response[0];

        assert_eq!(response.lot_id, "alice".parse().unwrap());
        assert_eq!(response.seller_id, "bob".parse().unwrap());
        assert_eq!(response.start_timestamp, (to_ts(10)).into());
        assert_eq!(response.finish_timestamp, (to_ts(11)).into());
        assert_eq!(response.reserve_price, to_yocto("5").into());
        assert_eq!(response.buy_now_price, to_yocto("10").into());
        assert_eq!(
            response.last_bid_amount,
            Some(first_bid_amount.into()),
            "wrong last_bid for active lot"
        );
        assert_eq!(
            response.next_bid_amount,
            Some(next_bid_amount.into()),
            "wrong next bid for active lot"
        );
        assert_eq!(response.is_active, true);
        assert_eq!(response.is_withdrawn, false);
    }

    #[test]
    fn lot_list_present_withdrawn() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        let mut lot = create_lot_bob_sells_alice();
        lot.is_withdrawn = true;
        contract.internal_lot_save(&lot);

        let context = get_context_pred_alice(true);
        testing_env!(context);

        let response: Vec<LotView> = contract.lot_list(None, None);
        assert!(!response.is_empty());
        let response: &LotView = &response[0];

        assert_eq!(response.is_withdrawn, true);
    }

    #[test]
    fn lot_is_active_buy_now() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }
        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert!(lot.is_active(to_ts(10)), "must be active with no bids",);

        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto("10"),
            timestamp: to_ts(10),
        };
        internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);
        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();

        assert!(
            !lot.is_active(to_ts(10) + 1),
            "must be inactive with buy now bid",
        );
    }

    #[test]
    fn lot_is_active_withdrawn() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }
        let mut lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert!(lot.is_active(to_ts(10)), "must be active with no bids",);

        lot.is_withdrawn = true;

        assert_eq!(
            lot.is_active(to_ts(10) + 1),
            false,
            "must be inactive with buy now bid",
        );
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

    // checks:
    //   - lot cannot bid
    //   - seller cannot bid
    //   - lot is active
    //   - bid amount is enough
    #[test]
    pub fn internal_lot_bid_double() {
        // in this method we don't care bout predecessor, it's internal method
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        let first_bid_amount = to_yocto("6");
        let second_bid_amount = to_yocto("8");

        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: first_bid_amount,
            timestamp: to_ts(10),
        };
        internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);
        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert_eq!(lot.bids().len(), 1, "expected one bid for lot");

        let last_bid = lot.last_bid().unwrap();
        assert_eq!(last_bid.amount, first_bid_amount, "wrong first bid amount");
        assert_eq!(
            last_bid.bidder_id,
            "carol".parse().unwrap(),
            "expected carol as last bidder"
        );
        assert_eq!(
            last_bid.timestamp,
            to_ts(10),
            "expected carol as last bidder"
        );

        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: second_bid_amount,
            timestamp: to_ts(10) + 1,
        };
        internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);
        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert_eq!(lot.bids().len(), 2, "wrong bids length");

        let last_bid = lot.last_bid().unwrap();
        assert_eq!(
            last_bid.amount, second_bid_amount,
            "expected last bid to be 6 near"
        );
        assert_eq!(
            last_bid.bidder_id,
            "carol".parse().unwrap(),
            "expected carol as last bidder"
        );
        assert_eq!(
            last_bid.timestamp,
            to_ts(10) + 1,
            "expected carol as last bidder"
        );
    }

    #[test]
    #[should_panic(expected = "bid: expected status active")]
    pub fn internal_lot_bid_fail_after_finish() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }
        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto("6"),
            timestamp: to_ts(13),
        };
        internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);
    }

    #[test]
    #[should_panic(expected = "bid: expected bigger bid")]
    pub fn internal_lot_bid_fail_bid_below_reserve() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }
        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto("4"),
            timestamp: to_ts(10),
        };
        internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);
    }

    #[test]
    #[should_panic(expected = "bid: expected bigger bid")]
    pub fn internal_lot_bid_fail_bid_below_prev_bid() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let bid: Bid = Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("7"),
                timestamp: to_ts(10),
            };
            internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);
        }

        {
            let bid: Bid = Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("6"),
                timestamp: to_ts(10) + 1,
            };
            internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);
        }
    }

    #[test]
    #[should_panic(expected = "bid: expected status active")]
    pub fn internal_lot_bid_fail_bid_above_buy_now_price() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let bid: Bid = Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("10"),
                timestamp: to_ts(10),
            };
            internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);
        }

        {
            let bid: Bid = Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("11"),
                timestamp: to_ts(10) + 1,
            };
            internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);
        }
    }

    fn subtract_seller_reward_commission(reward: Balance, commission: Fraction) -> Balance {
        reward - commission * reward
    }

    #[test]
    pub fn test_api_lot_bid() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        let first_bid_amount = to_yocto("6");
        let second_bid_amount = to_yocto("8");
        api_lot_bid(
            &mut contract,
            &"alice".parse().unwrap(),
            &Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: first_bid_amount,
                timestamp: to_ts(10),
            },
        );

        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert_eq!(lot.bids().len(), 1, "expected one bid for lot");

        let last_bid = lot.last_bid().unwrap();
        assert_eq!(last_bid.amount, first_bid_amount, "wrong_first_bid");
        assert_eq!(
            last_bid.bidder_id,
            "carol".parse().unwrap(),
            "expected carol as last bidder"
        );
        assert_eq!(last_bid.timestamp, to_ts(10), "expected start as timestamp");
        {
            let profile_alice = contract.internal_profile_extract(&"alice".parse().unwrap());
            assert_eq!(
                profile_alice.rewards_available, 0,
                "lot profile should have zero balance"
            );
            contract.internal_profile_save(&profile_alice);
        }
        {
            let profile_bob = contract.internal_profile_extract(&"bob".parse().unwrap());
            let expected = subtract_seller_reward_commission(
                first_bid_amount,
                contract.seller_rewards_commission,
            );
            assert_eq!(
                profile_bob.rewards_available, expected,
                "seller profile should have bid balance minus comission"
            );
            contract.internal_profile_save(&profile_bob);
        }
        {
            let profile_carol = contract.internal_profile_extract(&"carol".parse().unwrap());
            assert_eq!(
                profile_carol.rewards_available, 0,
                "first bidder profile should have zero balance"
            );
            contract.internal_profile_save(&profile_carol);
        }

        api_lot_bid(
            &mut contract,
            &"alice".parse().unwrap(),
            &Bid {
                bidder_id: "dan".parse().unwrap(),
                amount: second_bid_amount,
                timestamp: to_ts(10) + 1,
            },
        );

        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        let last_bid = lot.last_bid().unwrap();
        assert_eq!(
            last_bid.amount, second_bid_amount,
            "wrong second bid amount"
        );
        assert_eq!(
            last_bid.bidder_id,
            "dan".parse().unwrap(),
            "expected dan as last bidder"
        );
        assert_eq!(
            last_bid.timestamp,
            to_ts(10) + 1,
            "expected start plus one timestamp"
        );
        {
            let profile_alice = contract.internal_profile_extract(&"alice".parse().unwrap());
            assert_eq!(
                profile_alice.rewards_available, 0,
                "lot profile should have zero balance"
            );
            contract.internal_profile_save(&profile_alice);
        }
        {
            let profile_bob = contract.internal_profile_extract(&"bob".parse().unwrap());
            let expected = subtract_seller_reward_commission(
                second_bid_amount,
                contract.seller_rewards_commission,
            );
            assert_eq!(
                profile_bob.rewards_available, expected,
                "wrong seller rewards after second bid"
            );
            contract.internal_profile_save(&profile_bob);
        }
        {
            let first_bidder_rewards = first_bid_amount
                + contract.prev_bidder_commission_share
                    * (contract.seller_rewards_commission * (second_bid_amount - first_bid_amount));
            let profile_carol = contract.internal_profile_extract(&"carol".parse().unwrap());
            assert_eq!(
                profile_carol.rewards_available, first_bidder_rewards,
                "first bidder profile should have prev bid balance plus commission"
            );
            contract.internal_profile_save(&profile_carol);
        }
        {
            let profile_dan = contract.internal_profile_extract(&"dan".parse().unwrap());
            assert_eq!(
                profile_dan.rewards_available, 0,
                "last bidder profile should have zero balance"
            );
            contract.internal_profile_save(&profile_dan);
        }
    }

    #[test]
    #[should_panic(expected = "bid: expected bigger bid")]
    pub fn api_lot_bid_fail_small_bid() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        api_lot_bid(
            &mut contract,
            &"alice".parse().unwrap(),
            &Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("4"),
                timestamp: to_ts(10),
            },
        );
    }

    #[test]
    #[should_panic(expected = "bid: expected status active")]
    pub fn api_lot_bid_fail_inactive() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        api_lot_bid(
            &mut contract,
            &"alice".parse().unwrap(),
            &Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("6"),
                timestamp: to_ts(11),
            },
        );
    }

    // derived from empty string
    const DEFAULT_PUBLIC_KEY: &str = "ed25519:Ga6C8S7jVG2inG88cos8UsdtGVWRFQasSdTdtHL7kBqL";

    #[test]
    pub fn api_lot_claim_success() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let bid: Bid = Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto("7"),
                timestamp: to_ts(10),
            };
            internal_lot_bid(&mut contract, &"alice".parse().unwrap(), &bid);
        }

        {
            let context =
                get_context_with_payer(&"carol".parse().unwrap(), to_yocto("0"), to_ts(11));
            testing_env!(context);
            let public_key: PublicKey = DEFAULT_PUBLIC_KEY.parse().unwrap();

            contract.lot_claim("alice".parse().unwrap(), public_key);
        }
    }

    #[test]
    pub fn api_lot_claim_by_seller_success_withdraw_after_finish() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        let context = get_context_with_payer(&"bob".parse().unwrap(), to_yocto("0"), to_ts(13));
        testing_env!(context);
        contract.lot_withdraw("alice".to_string().try_into().unwrap());
        let public_key: PublicKey = DEFAULT_PUBLIC_KEY.parse().unwrap();
        contract.lot_claim("alice".parse().unwrap(), public_key);
    }

    #[test]
    #[should_panic(expected = "claim by bidder: expected status sale success")]
    pub fn api_lot_claim_fail_still_active() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = build_contract();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let context =
                get_context_with_payer(&"carol".parse().unwrap(), to_yocto("0"), to_ts(10));
            testing_env!(context);
            let public_key: PublicKey = DEFAULT_PUBLIC_KEY.parse().unwrap();

            contract.lot_claim("alice".parse().unwrap(), public_key);
        }
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
