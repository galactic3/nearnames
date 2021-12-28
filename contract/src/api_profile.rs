use crate::*;

pub const ERR_PROFILE_INTERNAL_SAVE_ALREADY_EXISTS: &str = "internal_profile_save: profile already exists";
pub const ERR_PROFILE_REWARDS_CLAIM_NOT_ENOUGH: &str = "profile_rewards_claim: not enough rewards";

pub const MIN_PROFILE_REWARDS_CLAIM_AMOUNT: Balance = 10 * 10u128.pow(21);

pub const GAS_EXT_CALL_AFTER_REWARDS_CLAIM: u64 = 20_000_000_000_000;

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProfileView {
    pub profile_id: ProfileId,
    pub rewards_available: WrappedBalance,
    pub rewards_claimed: WrappedBalance,
}

impl From<&Profile> for ProfileView {
    fn from(p: &Profile) -> Self {
        Self {
            profile_id: p.profile_id.clone(),
            rewards_available: p.rewards_available.into(),
            rewards_claimed: p.rewards_claimed.into(),
        }
    }
}

impl Contract {
    pub(crate) fn internal_profile_extract(&mut self, profile_id: &ProfileId) -> Profile {
        self.profiles
            .remove(&profile_id)
            .unwrap_or_else(|| Profile::new(&profile_id))
    }

    pub(crate) fn internal_profile_get(&self, profile_id: &ProfileId) -> Profile {
        self.profiles
            .get(&profile_id)
            .unwrap_or_else(|| Profile::new(&profile_id))
    }

    pub(crate) fn internal_profile_save(&mut self, profile: &Profile) {
        assert!(
            self.profiles.insert(&profile.profile_id, profile).is_none(),
            "{}",
            ERR_PROFILE_INTERNAL_SAVE_ALREADY_EXISTS
        );
    }

    pub(crate) fn internal_profile_rewards_transfer(
        &mut self,
        profile_id: &ProfileId,
        value: Balance,
    ) {
        if value == 0 {
            return;
        }

        let mut profile = self.internal_profile_extract(profile_id);
        profile.rewards_transfer(value);
        self.internal_profile_save(&profile);
    }
}

#[near_bindgen]
impl Contract {
    pub fn profile_get(&self, profile_id: ProfileId) -> ProfileView {
        (&self.internal_profile_get(&profile_id)).into()
    }

    pub fn profile_rewards_claim(&mut self) -> Promise {
        let profile_id: ProfileId = env::predecessor_account_id();
        let mut profile = self.internal_profile_extract(&profile_id.clone());

        let rewards = profile.rewards_claim();
        assert!(
            rewards >= MIN_PROFILE_REWARDS_CLAIM_AMOUNT,
            "{}",
            ERR_PROFILE_REWARDS_CLAIM_NOT_ENOUGH,
        );
        self.internal_profile_save(&profile);

        Promise::new(profile_id.clone()).transfer(rewards).then(
            ext_self_contract::profile_after_rewards_claim(
                profile_id.clone(),
                rewards,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_EXT_CALL_AFTER_REWARDS_CLAIM.into(),
            ),
        )
    }

    #[private]
    pub fn profile_after_rewards_claim(&mut self, profile_id: ProfileId, rewards: Balance) -> bool {
        let rewards_transferred = is_promise_success();
        if !rewards_transferred {
            // In case of failure, put the amount back
            let mut profile = self.internal_profile_extract(&profile_id);
            profile.rewards_claim_revert(rewards);
            self.internal_profile_save(&profile);
        }
        rewards_transferred
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk_sim::{to_ts, to_yocto};
    use near_sdk::testing_env;

    use crate::tests::build_contract;
    use crate::api_lot::tests::{get_context_view, get_context_call};
    use crate::profile::tests::create_profile_bob;

    fn create_contract_with_profile_bob() -> (Contract, ProfileId) {
        let mut contract = build_contract();
        let profile = create_profile_bob();
        contract.internal_profile_save(&profile);

        (contract, profile.profile_id.clone())
    }

    #[test]
    fn test_api_profile_internal_get() {
        let (contract, profile_id) = create_contract_with_profile_bob();

        testing_env!(get_context_view(to_ts(11)));
        let profile = contract.internal_profile_get(&profile_id);
        assert_eq!(contract.profiles.len(), 1, "wrong profiles len");
        assert_eq!(profile.profile_id, profile_id, "wrong rewards_available");
        assert_eq!(profile.rewards_available, to_yocto("3"), "wrong rewards_available");
        assert_eq!(profile.rewards_claimed, to_yocto("2"), "wrong rewards_claimed");
    }

    #[test]
    fn test_api_profile_internal_extract() {
        let (mut contract, profile_id) = create_contract_with_profile_bob();

        let profile = contract.internal_profile_extract(&profile_id);
        assert_eq!(contract.profiles.len(), 0, "wrong profiles len");
        assert_eq!(profile.profile_id, profile_id, "wrong rewards_available");
        assert_eq!(profile.rewards_available, to_yocto("3"), "wrong rewards_available");
        assert_eq!(profile.rewards_claimed, to_yocto("2"), "wrong rewards_claimed");
    }

    #[test]
    fn test_api_profile_internal_save_success() {
        let (contract, _profile_id) = create_contract_with_profile_bob();
        assert_eq!(contract.profiles.len(), 1, "wrong profiles len");
    }

    #[test]
    #[should_panic(expected="internal_profile_save: profile already exists")]
    fn test_api_profile_internal_save_fail_already_exists() {
        let (mut contract, profile_id) = create_contract_with_profile_bob();
        let profile = Profile::new(&profile_id);
        contract.internal_profile_save(&profile);
    }

    #[test]
    fn test_api_profile_internal_rewards_transfer() {
        let mut contract = build_contract();
        let profile_id: ProfileId = "bob".parse().unwrap();

        contract.internal_profile_rewards_transfer(&profile_id, to_yocto("3"));
        assert_eq!(contract.profiles.len(), 1);

        let profile = contract.profiles.get(&profile_id).unwrap();
        assert_eq!(profile.rewards_available, to_yocto("3"));

        contract.internal_profile_rewards_transfer(&profile_id, to_yocto("2"));
        let profile = contract.profiles.get(&profile_id).unwrap();
        assert_eq!(profile.rewards_available, to_yocto("5"), "wrong amount");
    }

    #[test]
    fn test_api_profile_get_missing() {
        let contract = build_contract();
        let profile_id: ProfileId = "alice".parse().unwrap();

        testing_env!(get_context_view(to_ts(11)));
        let profile = contract.profile_get(profile_id.clone());
        assert_eq!(profile.profile_id, profile_id, "wrong profile_id");
    }

    #[test]
    fn test_api_profile_get_present() {
        let (contract, profile_id) = create_contract_with_profile_bob();

        testing_env!(get_context_view(to_ts(11)));
        let response: ProfileView = contract.profile_get(profile_id.clone());
        assert_eq!(
            response.profile_id,
            profile_id,
            "wrong rewards_available",
        );
        assert_eq!(
            response.rewards_available,
            to_yocto("3").into(),
            "wrong rewards_available",
        );
        assert_eq!(
            response.rewards_claimed,
            to_yocto("2").into(),
            "wrong rewards_claimed",
        );
    }

    #[test]
    pub fn test_api_profile_rewards_claim_success() {
        let (mut contract, profile_id) = create_contract_with_profile_bob();

        testing_env!(get_context_call(to_ts(11), &profile_id));
        contract.profile_rewards_claim();

        let profile = contract.profile_get(profile_id.clone());
        assert_eq!(profile.rewards_available.0, to_yocto("0"), "wrong rewards_available");
        assert_eq!(profile.rewards_claimed.0, to_yocto("5"), "wrong rewards_claimed");
    }

    #[test]
    #[should_panic(expected="profile_rewards_claim: not enough rewards")]
    pub fn test_api_profile_rewards_claim_fail_not_enough() {
        let mut contract = build_contract();
        let profile_id: ProfileId = "alice".parse().unwrap();

        testing_env!(get_context_call(to_ts(11), &profile_id));
        contract.profile_rewards_claim();
    }
}
