mod lot;
mod profile;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::{ValidAccountId, WrappedBalance};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, Balance, Duration, Timestamp};

use crate::lot::*;
use crate::profile::*;

pub type LotId = AccountId;
pub type ProfileId = AccountId;

pub const PREFIX_PROFILES: &str = "u";
pub const PREFIX_LOTS: &str = "a";

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

    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn get_context_simple(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("alice_near".try_into().unwrap())
            .is_view(is_view)
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

        let yocto_in_near: u128 = 10u128.pow(24);

        let profile_id: AccountId = "alice".into();
        let available_rewards: u128 = yocto_in_near * 2;
        let profit_received: u128 = yocto_in_near * 3;
        let profile: Profile = Profile {
            available_rewards,
            profit_received,
        };
        contract.internal_profile_save_or_panic(&profile_id, &profile);

        let response: Option<ProfileView> = contract.profile_get("alice".try_into().unwrap());
        assert!(response.is_some());
        let response = response.unwrap();

        assert_eq!(
            response.available_rewards,
            available_rewards.into(),
            "available_rewards mismatch"
        );
        assert_eq!(
            response.profit_received,
            profit_received.into(),
            "profit_received mismatch"
        );
    }
}
