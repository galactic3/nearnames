mod lot;
mod profile;
mod utils;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, Balance, Duration, Promise, PromiseResult,
    PublicKey, Timestamp,
};

pub use crate::lot::*;
pub use crate::profile::*;
pub use crate::utils::*;

pub type LotId = AccountId;
pub type ProfileId = AccountId;
pub type WrappedBalance = U128;
pub type WrappedTimestamp = U64;

pub const PREFIX_PROFILES: &str = "u";
pub const PREFIX_LOTS: &str = "a";
pub const PREFIX_LOTS_BIDS: &str = "y";

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
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub profiles: UnorderedMap<ProfileId, Profile>,
    pub lots: UnorderedMap<LotId, Lot>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            profiles: UnorderedMap::new(PREFIX_PROFILES.as_bytes().to_vec()),
            lots: UnorderedMap::new(PREFIX_LOTS.as_bytes().to_vec()),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn hello(&self) -> String {
        format!("hello, {}", env::predecessor_account_id())
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    fn to_yocto(near_value: Balance) -> Balance {
        near_value * 10u128.pow(24)
    }

    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context_simple(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("alice_near".parse().unwrap())
            .is_view(is_view)
            .block_timestamp(DAY_NANOSECONDS * 13)
            .build()
    }

    fn get_context_pred_alice(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id("alice".parse().unwrap())
            .is_view(is_view)
            .block_timestamp(DAY_NANOSECONDS * 10)
            .build()
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

    #[test]
    fn profile_get_missing() {
        let context = get_context_simple(false);
        testing_env!(context);
        let contract = Contract::default();

        assert!(
            contract.profile_get("alice".parse().unwrap()).is_none(),
            "Expected get_profile to return None",
        );
    }

    #[test]
    fn profile_get_present() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();

        let rewards_available: u128 = to_yocto(2);
        let rewards_claimed: u128 = to_yocto(3);
        let profile: Profile = Profile {
            profile_id: "alice".parse().unwrap(),
            rewards_available,
            rewards_claimed,
        };
        contract.internal_profile_save(&profile);

        let response: Option<ProfileView> = contract.profile_get("alice".parse().unwrap());
        assert!(response.is_some());
        let response = response.unwrap();

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
        let mut contract = Contract::default();
        contract.internal_lot_extract(&"alice".parse().unwrap());
    }

    const DAY_NANOSECONDS: u64 = 10u64.pow(9) * 60 * 60 * 24;

    fn create_lot_bob_sells_alice() -> Lot {
        let reserve_price = to_yocto(5);
        let buy_now_price = to_yocto(10);

        let time_now = DAY_NANOSECONDS * 10;
        let duration = DAY_NANOSECONDS * 1;

        Contract::internal_lot_create(
            "alice".parse().unwrap(),
            "bob".parse().unwrap(),
            reserve_price,
            buy_now_price,
            time_now,
            duration,
        )
    }

    #[test]
    fn internal_lot_create_fields() {
        let context = get_context_simple(false);
        testing_env!(context);

        let lot = create_lot_bob_sells_alice();
        assert_eq!(
            lot.lot_id,
            "alice".parse().unwrap(),
            "expected lot.lot_id = alice"
        );
        assert_eq!(
            lot.seller_id,
            "bob".parse().unwrap(),
            "expected lot.seller_id = bob"
        );
        assert_eq!(
            lot.reserve_price,
            to_yocto(5),
            "expected reserve price 5 yocto"
        );
        assert_eq!(
            lot.buy_now_price,
            to_yocto(10),
            "expected buy now price 10 yocto"
        );
        assert_eq!(
            lot.start_timestamp,
            DAY_NANOSECONDS * 10,
            "expected start day ten"
        );
        assert_eq!(
            lot.finish_timestamp,
            DAY_NANOSECONDS * 11,
            "expected finish day eleven"
        );
    }

    #[test]
    fn internal_lot_create_save_extract() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
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

    #[test]
    fn lot_get_missing() {
        let context = get_context_simple(false);
        testing_env!(context);
        let contract = Contract::default();

        assert!(
            contract.lot_list().is_empty(),
            "Expected lot_list to be empty",
        );
    }

    #[test]
    fn lot_list_present() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        let lot_bob_sells_alice = create_lot_bob_sells_alice();
        contract.internal_lot_save(&lot_bob_sells_alice);

        let response: Vec<LotView> = contract.lot_list();
        assert!(!response.is_empty());
        let response: &LotView = &response[0];

        assert_eq!(response.lot_id, "alice".parse().unwrap());
        assert_eq!(response.seller_id, "bob".parse().unwrap());
        assert_eq!(response.start_timestamp, (DAY_NANOSECONDS * 10).into());
        assert_eq!(response.finish_timestamp, (DAY_NANOSECONDS * 11).into());
        assert_eq!(response.reserve_price, to_yocto(5).into());
        assert_eq!(response.buy_now_price, to_yocto(10).into());
        assert_eq!(response.last_bid_amount, None);
        assert_eq!(
            response.next_bid_amount, None,
            "expected none next price for inactive lot"
        );
        assert_eq!(response.is_active, false);
        assert_eq!(response.is_withdrawn, false);
    }

    #[test]
    fn lot_bid_list() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        let lot_bob_sells_alice = create_lot_bob_sells_alice();
        contract.internal_lot_save(&lot_bob_sells_alice);

        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto(7),
            timestamp: DAY_NANOSECONDS * 10,
        };
        contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);

        let bid: Bid = Bid {
            bidder_id: "dan".parse().unwrap(),
            amount: to_yocto(9),
            timestamp: DAY_NANOSECONDS * 10 + 1,
        };
        contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);

        let response: Vec<BidView> = contract.lot_bid_list("alice".parse().unwrap());
        let expected: Vec<BidView> = vec![
            BidView {
                bidder_id: "carol".parse().unwrap(),
                amount: WrappedBalance::from(to_yocto(7)),
                timestamp: WrappedTimestamp::from(DAY_NANOSECONDS * 10),
            },
            BidView {
                bidder_id: "dan".parse().unwrap(),
                amount: WrappedBalance::from(to_yocto(9)),
                timestamp: WrappedTimestamp::from(DAY_NANOSECONDS * 10 + 1),
            },
        ];

        assert_eq!(response, expected, "wrong bids list");
    }

    #[test]
    fn lot_list_present_active() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        let lot_bob_sells_alice = create_lot_bob_sells_alice();
        contract.internal_lot_save(&lot_bob_sells_alice);

        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto(7),
            timestamp: DAY_NANOSECONDS * 10,
        };
        contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);

        let context = get_context_pred_alice(true);
        testing_env!(context);

        let response: Vec<LotView> = contract.lot_list();
        assert!(!response.is_empty());
        let response: &LotView = &response[0];

        assert_eq!(response.lot_id, "alice".parse().unwrap());
        assert_eq!(response.seller_id, "bob".parse().unwrap());
        assert_eq!(response.start_timestamp, (DAY_NANOSECONDS * 10).into());
        assert_eq!(response.finish_timestamp, (DAY_NANOSECONDS * 11).into());
        assert_eq!(response.reserve_price, to_yocto(5).into());
        assert_eq!(response.buy_now_price, to_yocto(10).into());
        assert_eq!(
            response.last_bid_amount,
            Some((to_yocto(7)).into()),
            "expected present last price 7 near"
        );
        assert_eq!(
            response.next_bid_amount,
            Some((to_yocto(7) + 1).into()),
            "expected none next price for inactive lot"
        );
        assert_eq!(response.is_active, true);
        assert_eq!(response.is_withdrawn, false);
    }

    #[test]
    fn lot_list_present_withdrawn() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        let mut lot = create_lot_bob_sells_alice();
        lot.is_withdrawn = true;
        contract.internal_lot_save(&lot);

        let context = get_context_pred_alice(true);
        testing_env!(context);

        let response: Vec<LotView> = contract.lot_list();
        assert!(!response.is_empty());
        let response: &LotView = &response[0];

        assert_eq!(response.is_withdrawn, true);
    }

    #[test]
    fn lot_is_active_by_tm() {
        let context = get_context_simple(false);
        testing_env!(context);

        let lot_bob_sells_alice = create_lot_bob_sells_alice();
        // we don't care about starting time, it's just for the record
        assert_eq!(lot_bob_sells_alice.is_active(DAY_NANOSECONDS * 9), true);
        assert_eq!(lot_bob_sells_alice.is_active(DAY_NANOSECONDS * 10), true);
        assert_eq!(
            lot_bob_sells_alice.is_active(DAY_NANOSECONDS * 11 - 1),
            true
        );
        assert_eq!(lot_bob_sells_alice.is_active(DAY_NANOSECONDS * 11), false);
        assert_eq!(lot_bob_sells_alice.is_active(DAY_NANOSECONDS * 12), false);
    }

    #[test]
    fn lot_is_active_buy_now() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }
        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert!(
            lot.is_active(DAY_NANOSECONDS * 10),
            "must be active with no bids",
        );

        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto(10),
            timestamp: DAY_NANOSECONDS * 10,
        };
        contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);
        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();

        assert!(
            !lot.is_active(DAY_NANOSECONDS * 10 + 1),
            "must be inactive with buy now bid",
        );
    }

    #[test]
    fn lot_is_active_withdrawn() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }
        let mut lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert!(
            lot.is_active(DAY_NANOSECONDS * 10),
            "must be active with no bids",
        );

        lot.is_withdrawn = true;

        assert_eq!(
            lot.is_active(DAY_NANOSECONDS * 10 + 1),
            false,
            "must be inactive with buy now bid",
        );
    }

    #[test]
    fn lot_create_api() {
        let context = get_context_pred_alice(false);
        testing_env!(context);

        let mut contract = Contract::default();

        let lot_id: ProfileId = "alice".parse().unwrap();
        let seller_id: ProfileId = "bob".parse().unwrap();
        let reserve_price = to_yocto(5);
        let buy_now_price = to_yocto(10);
        let duration = DAY_NANOSECONDS * 1;

        contract.lot_offer(
            seller_id.try_into().unwrap(),
            reserve_price.into(),
            buy_now_price.into(),
            duration,
        );

        let result = contract.lots.get(&lot_id);
        assert!(result.is_some(), "expected lot_saved is present");
        let result = result.unwrap();

        assert_eq!(result.lot_id, "alice".parse().unwrap());
        assert_eq!(result.seller_id, "bob".parse().unwrap());
        assert_eq!(
            result.start_timestamp,
            DAY_NANOSECONDS * 10,
            "expected start day ten"
        );
        assert_eq!(
            result.finish_timestamp,
            DAY_NANOSECONDS * 11,
            "expected finish day eleven"
        );
        assert_eq!(result.reserve_price, to_yocto(5).into());
        assert_eq!(result.buy_now_price, to_yocto(10).into());
    }

    #[test]
    fn internal_transfer() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        let profile_id: ProfileId = "alice".parse().unwrap();
        contract.internal_profile_rewards_transfer(&profile_id, to_yocto(3));
        let profile = contract.profiles.get(&profile_id);
        assert!(profile.is_some());
        let profile = profile.unwrap();

        assert_eq!(profile.rewards_available, to_yocto(3));

        contract.internal_profile_rewards_transfer(&profile_id, to_yocto(2));
        assert_eq!(contract.profiles.len(), 1);
        let profile = contract.profiles.get(&profile_id);
        assert!(profile.is_some());
        let profile = profile.unwrap();

        assert_eq!(
            profile.rewards_available,
            to_yocto(5),
            "expected balance 5 near after two transfers"
        );
    }

    #[test]
    fn last_bid_amount() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut lot = create_lot_bob_sells_alice();
        assert!(
            lot.last_bid_amount().is_none(),
            "expected last_bid_amount to be None"
        );

        let bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            timestamp: DAY_NANOSECONDS * 10,
            amount: to_yocto(6),
        };
        lot.bids.push(&bid);
        assert_eq!(lot.last_bid_amount().unwrap(), to_yocto(6));

        let bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            timestamp: DAY_NANOSECONDS * 10,
            amount: to_yocto(7),
        };
        lot.bids.push(&bid);
        assert_eq!(lot.last_bid_amount().unwrap(), to_yocto(7));
    }

    #[test]
    fn next_bid_amount() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut lot = create_lot_bob_sells_alice();
        assert!(
            lot.last_bid_amount().is_none(),
            "expected last_bid_amount to be None"
        );

        let time_now: Timestamp = DAY_NANOSECONDS * 12;
        assert!(
            lot.next_bid_amount(time_now).is_none(),
            "Expected next_bid_amount to be none for inactive lot",
        );

        let time_now: Timestamp = DAY_NANOSECONDS * 10;
        assert_eq!(lot.next_bid_amount(time_now).unwrap(), to_yocto(5));

        let bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            timestamp: DAY_NANOSECONDS * 10,
            amount: to_yocto(6),
        };
        lot.bids.push(&bid);
        assert_eq!(lot.next_bid_amount(time_now).unwrap(), to_yocto(6) + 1);
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
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto(7),
            timestamp: DAY_NANOSECONDS * 10,
        };
        contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);
        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert_eq!(lot.bids.len(), 1, "expected one bid for lot");

        let last_bid = lot.bids.get(lot.bids.len() - 1).unwrap();
        assert_eq!(
            last_bid.amount,
            to_yocto(7),
            "expected last bid to be 6 near"
        );
        assert_eq!(
            last_bid.bidder_id,
            "carol".parse().unwrap(),
            "expected carol as last bidder"
        );
        assert_eq!(
            last_bid.timestamp,
            DAY_NANOSECONDS * 10,
            "expected carol as last bidder"
        );

        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto(8),
            timestamp: DAY_NANOSECONDS * 10 + 1,
        };
        contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);
        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert_eq!(lot.bids.len(), 2, "expected one bid for lot");

        let last_bid = lot.bids.get(lot.bids.len() - 1).unwrap();
        assert_eq!(
            last_bid.amount,
            to_yocto(8),
            "expected last bid to be 6 near"
        );
        assert_eq!(
            last_bid.bidder_id,
            "carol".parse().unwrap(),
            "expected carol as last bidder"
        );
        assert_eq!(
            last_bid.timestamp,
            DAY_NANOSECONDS * 10 + 1,
            "expected carol as last bidder"
        );
    }

    #[test]
    #[should_panic(expected = "Expected lot to be active")]
    pub fn internal_lot_bid_fail_after_finish() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }
        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto(6),
            timestamp: DAY_NANOSECONDS * 13,
        };
        contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);
    }

    #[test]
    #[should_panic(expected = "Expected bigger bid")]
    pub fn internal_lot_bid_fail_bid_below_reserve() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }
        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto(4),
            timestamp: DAY_NANOSECONDS * 10,
        };
        contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);
    }

    #[test]
    #[should_panic(expected = "Expected bigger bid")]
    pub fn internal_lot_bid_fail_bid_below_prev_bid() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let bid: Bid = Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto(7),
                timestamp: DAY_NANOSECONDS * 10,
            };
            contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);
        }

        {
            let bid: Bid = Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto(6),
                timestamp: DAY_NANOSECONDS * 10 + 1,
            };
            contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);
        }
    }

    #[test]
    #[should_panic(expected = "Expected lot to be active")]
    pub fn internal_lot_bid_fail_bid_above_buy_now_price() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let bid: Bid = Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto(10),
                timestamp: DAY_NANOSECONDS * 10,
            };
            contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);
        }

        {
            let bid: Bid = Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto(11),
                timestamp: DAY_NANOSECONDS * 10 + 1,
            };
            contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);
        }
    }

    #[test]
    pub fn api_lot_bid() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let context = get_context_with_payer(
                &"carol".parse().unwrap(),
                to_yocto(7),
                DAY_NANOSECONDS * 10,
            );
            testing_env!(context);
            contract.lot_bid("alice".to_string().try_into().unwrap());
        }

        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        assert_eq!(lot.bids.len(), 1, "expected one bid for lot");

        let last_bid = lot.last_bid().unwrap();
        assert_eq!(
            last_bid.amount,
            to_yocto(7),
            "expected last bid to be 7 near"
        );
        assert_eq!(
            last_bid.bidder_id,
            "carol".parse().unwrap(),
            "expected carol as last bidder"
        );
        assert_eq!(
            last_bid.timestamp,
            DAY_NANOSECONDS * 10,
            "expected start as timestamp"
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
            assert_eq!(
                profile_bob.rewards_available,
                to_yocto(7),
                "seller profile should have bid balance"
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

        {
            let context = get_context_with_payer(
                &"dan".parse().unwrap(),
                to_yocto(8),
                DAY_NANOSECONDS * 10 + 1,
            );
            testing_env!(context);
            contract.lot_bid("alice".to_string().try_into().unwrap());
        }

        let lot = contract.lots.get(&"alice".parse().unwrap()).unwrap();
        let last_bid = lot.last_bid().unwrap();
        assert_eq!(
            last_bid.amount,
            to_yocto(8),
            "expected last bid to be 8 near"
        );
        assert_eq!(
            last_bid.bidder_id,
            "dan".parse().unwrap(),
            "expected dan as last bidder"
        );
        assert_eq!(
            last_bid.timestamp,
            DAY_NANOSECONDS * 10 + 1,
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
            assert_eq!(
                profile_bob.rewards_available,
                to_yocto(8),
                "lot profile should have bid balance"
            );
            contract.internal_profile_save(&profile_bob);
        }
        {
            let profile_carol = contract.internal_profile_extract(&"carol".parse().unwrap());
            assert_eq!(
                profile_carol.rewards_available,
                to_yocto(7),
                "first bidder profile should have prev bid balance"
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
    #[should_panic(expected = "Expected bigger bid")]
    pub fn api_lot_bid_fail_small_bid() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let context = get_context_with_payer(
                &"carol".parse().unwrap(),
                to_yocto(4),
                DAY_NANOSECONDS * 10,
            );
            testing_env!(context);
            contract.lot_bid("alice".to_string().try_into().unwrap());
        }
    }

    #[test]
    #[should_panic(expected = "Expected lot to be active")]
    pub fn api_lot_bid_fail_inactive() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let context = get_context_with_payer(
                &"carol".parse().unwrap(),
                to_yocto(6),
                DAY_NANOSECONDS * 11,
            );
            testing_env!(context);
            contract.lot_bid("alice".to_string().try_into().unwrap());
        }
    }

    #[test]
    pub fn api_lot_withdraw_success() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        let context = get_context_with_payer(
            &"bob".parse().unwrap(),
            0,
            DAY_NANOSECONDS * 13,
        );
        testing_env!(context);
        contract.lot_withdraw("alice".to_string().try_into().unwrap());
    }

    #[test]
    #[should_panic(expected = "Only seller can withdraw")]
    pub fn api_lot_withdraw_fail_not_seller() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        let context = get_context_with_payer(
            &"carol".parse().unwrap(),
            0,
            DAY_NANOSECONDS * 13,
        );
        testing_env!(context);
        contract.lot_withdraw("alice".to_string().try_into().unwrap());
    }

    #[test]
    #[should_panic(expected = "Bid exists, cannot withdraw")]
    pub fn api_lot_withdraw_fail_has_bids() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        let bid: Bid = Bid {
            bidder_id: "carol".parse().unwrap(),
            amount: to_yocto(7),
            timestamp: DAY_NANOSECONDS * 10,
        };
        contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);

        let context = get_context_with_payer(
            &"bob".parse().unwrap(),
            0,
            DAY_NANOSECONDS * 13,
        );
        testing_env!(context);
        contract.lot_withdraw("alice".to_string().try_into().unwrap());
    }

    #[test]
    #[should_panic(expected = "Lot already withdrawn")]
    pub fn api_lot_withdraw_fail_double() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        let context = get_context_with_payer(
            &"bob".parse().unwrap(),
            0,
            DAY_NANOSECONDS * 13,
        );
        testing_env!(context);
        contract.lot_withdraw("alice".to_string().try_into().unwrap());
        contract.lot_withdraw("alice".to_string().try_into().unwrap());
    }

    // derived from empty string
    const DEFAULT_PUBLIC_KEY: &str = "ed25519:Ga6C8S7jVG2inG88cos8UsdtGVWRFQasSdTdtHL7kBqL";

    #[test]
    pub fn api_lot_claim_success() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let bid: Bid = Bid {
                bidder_id: "carol".parse().unwrap(),
                amount: to_yocto(7),
                timestamp: DAY_NANOSECONDS * 10,
            };
            contract.internal_lot_bid(&"alice".parse().unwrap(), &bid);
        }

        {
            let context = get_context_with_payer(
                &"carol".parse().unwrap(),
                to_yocto(0),
                DAY_NANOSECONDS * 11,
            );
            testing_env!(context);
            let public_key: PublicKey = DEFAULT_PUBLIC_KEY.parse().unwrap();

            contract.lot_claim("alice".parse().unwrap(), public_key);
        }
    }

    #[test]
    pub fn api_lot_claim_by_seller_success_withdraw_after_finish() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        let context = get_context_with_payer(
            &"bob".parse().unwrap(),
            to_yocto(0),
            DAY_NANOSECONDS * 13,
        );
        testing_env!(context);
        contract.lot_withdraw("alice".to_string().try_into().unwrap());
        let public_key: PublicKey = DEFAULT_PUBLIC_KEY.parse().unwrap();
        contract.lot_claim("alice".parse().unwrap(), public_key);
    }

    #[test]
    #[should_panic(expected = "Expected lot to be not active")]
    pub fn api_lot_claim_fail_still_active() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        {
            let lot = create_lot_bob_sells_alice();
            contract.internal_lot_save(&lot);
        }

        {
            let context = get_context_with_payer(
                &"carol".parse().unwrap(),
                to_yocto(0),
                DAY_NANOSECONDS * 10,
            );
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
        let mut contract = Contract::default();
        let account_id: AccountId = "alice".parse().unwrap();
        let amount = MIN_PROFILE_REWARDS_CLAIM_AMOUNT - 1;

        contract.internal_profile_rewards_transfer(&account_id, amount);

        let context = get_context_pred_alice(false);
        testing_env!(context);

        contract.profile_rewards_claim();
    }

    #[test]
    pub fn profile_rewards_claim_success() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        let account_id: AccountId = "alice".parse().unwrap();
        let amount = MIN_PROFILE_REWARDS_CLAIM_AMOUNT;
        contract.internal_profile_rewards_transfer(&account_id, amount);

        let context = get_context_pred_alice(false);
        testing_env!(context);
        contract.profile_rewards_claim();
        let result = contract.profile_get(account_id.clone()).unwrap();
        assert_eq!(result.rewards_available.0, to_yocto(0), "Expected rewards_available 0 after claim");
        assert_eq!(result.rewards_claimed.0, amount, "Expected rewards_claimed amount after claim");
    }
}
