mod lot;
mod profile;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{Vector, UnorderedMap};
use near_sdk::json_types::{ValidAccountId, WrappedBalance, WrappedTimestamp};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, Balance, Duration, Timestamp};

use crate::lot::*;
use crate::profile::*;

pub type LotId = AccountId;
pub type ProfileId = AccountId;

pub const PREFIX_PROFILES: &str = "u";
pub const PREFIX_LOTS: &str = "a";
pub const PREFIX_LOTS_BIDS: &str = "y";

setup_alloc!();

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
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn get_context_simple(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("alice_near".try_into().unwrap())
            .is_view(is_view)
            .block_timestamp(DAY_NANOSECONDS * 13)
            .build()
    }

    fn get_context_pred_alice(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .predecessor_account_id("alice".try_into().unwrap())
            .is_view(is_view)
            .block_timestamp(DAY_NANOSECONDS * 10)
            .build()
    }

    #[test]
    fn profile_get_missing() {
        let context = get_context_simple(false);
        testing_env!(context);
        let contract = Contract::default();

        assert!(
            contract.profile_get("alice".try_into().unwrap()).is_none(),
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
            profile_id: "alice".into(),
            rewards_available,
            rewards_claimed,
        };
        contract.internal_profile_save(&profile);

        let response: Option<ProfileView> = contract.profile_get("alice".try_into().unwrap());
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
        contract.internal_lot_extract(&AccountId::from("alice"));
    }

    const DAY_NANOSECONDS: u64 = 10u64.pow(9) * 60 * 60 * 24;

    fn create_lot_bob_sells_alice() -> Lot {
        let reserve_price = to_yocto(5);
        let buy_now_price = to_yocto(10);

        let time_now = DAY_NANOSECONDS * 10;
        let duration = DAY_NANOSECONDS * 1;

        Contract::internal_lot_create(
            "alice".into(),
            "bob".into(),
            reserve_price,
            buy_now_price,
            time_now,
            duration,
        )
    }

    #[test]
    fn internal_lot_create_fields() {
        let lot = create_lot_bob_sells_alice();
        assert_eq!(lot.lot_id, "alice", "expected lot.lot_id = alice");
        assert_eq!(lot.seller_id, "bob", "expected lot.seller_id = bob");
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

        assert_eq!(lot_found.lot_id, "alice", "{}", "expected lot_id == alice");
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
    fn lot_get_present() {
        let context = get_context_simple(false);
        testing_env!(context);
        let mut contract = Contract::default();
        let lot_bob_sells_alice = create_lot_bob_sells_alice();
        contract.internal_lot_save(&lot_bob_sells_alice);

        let response: Vec<LotView> = contract.lot_list();
        assert!(!response.is_empty());
        let response: &LotView = &response[0];

        assert_eq!(response.lot_id, "alice");
        assert_eq!(response.seller_id, "bob");
        assert_eq!(response.start_timestamp, (DAY_NANOSECONDS * 10).into());
        assert_eq!(response.finish_timestamp, (DAY_NANOSECONDS * 11).into());
        assert_eq!(response.reserve_price, to_yocto(5).into());
        assert_eq!(response.buy_now_price, to_yocto(10).into());
        assert_eq!(response.is_active, false);
    }

    #[test]
    fn lot_is_active() {
        let lot_bob_sells_alice = create_lot_bob_sells_alice();
        // we don't care about starting time, it's just for the record
        assert_eq!(lot_bob_sells_alice.is_active(DAY_NANOSECONDS * 9), true);
        assert_eq!(lot_bob_sells_alice.is_active(DAY_NANOSECONDS * 10), true);
        assert_eq!(lot_bob_sells_alice.is_active(DAY_NANOSECONDS * 11), true);
        assert_eq!(lot_bob_sells_alice.is_active(DAY_NANOSECONDS * 12), false);
    }

    #[test]
    fn lot_create_api() {
        let context = get_context_pred_alice(false);
        testing_env!(context);

        let mut contract = Contract::default();

        let lot_id: ProfileId = "alice".into();
        let seller_id: ProfileId = "bob".into();
        let reserve_price = to_yocto(5);
        let buy_now_price = to_yocto(10);
        let duration = DAY_NANOSECONDS * 1;

        contract.lot_offer(
            seller_id.try_into().unwrap(),
            reserve_price,
            buy_now_price,
            duration,
        );

        let result = contract.lots.get(&lot_id);
        assert!(result.is_some(), "expected lot_saved is present");
        let result = result.unwrap();

        assert_eq!(result.lot_id, "alice");
        assert_eq!(result.seller_id, "bob");
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
        let profile_id: ProfileId = "alice".to_string();
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
}
