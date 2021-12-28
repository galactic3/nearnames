use crate::*;

pub const ERR_PROFILE_INTERNAL_SAVE_ALREADY_EXISTS: &str = "internal_profile_save: profile already exists";
pub const ERR_PROFILE_REWARDS_CLAIM_NOT_ENOUGH: &str = "Not enough rewards for transfer";

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

        let rewards = profile.rewards_available;
        assert!(
            rewards >= MIN_PROFILE_REWARDS_CLAIM_AMOUNT,
            "{}",
            ERR_PROFILE_REWARDS_CLAIM_NOT_ENOUGH,
        );

        profile.rewards_available = 0;
        profile.rewards_claimed += rewards;
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
            profile.rewards_available = rewards;
            profile.rewards_claimed -= rewards;
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
    use crate::api_lot::tests::get_context_view;

    #[test]
    fn test_api_profile_internal_get() {
        let mut contract = build_contract();
        let profile_id: ProfileId = "bob".parse().unwrap();

        let mut profile = contract.internal_profile_get(&profile_id);

        profile.rewards_transfer(to_yocto("3"));
        contract.internal_profile_save(&profile);

        testing_env!(get_context_view(to_ts(11)));
        let profile = contract.internal_profile_get(&profile_id);
        assert_eq!(contract.profiles.len(), 1, "wrong profiles len");
        assert_eq!(profile.rewards_available, to_yocto("3"), "wrong rewards");
    }

    #[test]
    fn test_api_profile_internal_extract() {
        let mut contract = build_contract();
        let profile_id: ProfileId = "bob".parse().unwrap();

        let mut profile = contract.internal_profile_extract(&profile_id);
        profile.rewards_transfer(to_yocto("3"));
        contract.internal_profile_save(&profile);

        let profile = contract.internal_profile_extract(&profile_id);
        assert_eq!(contract.profiles.len(), 0, "wrong profiles len");
        assert_eq!(profile.rewards_available, to_yocto("3"), "wrong rewards");
    }

    #[test]
    fn test_api_profile_internal_save_success() {
        let mut contract = build_contract();
        let profile_id: ProfileId = "bob".parse().unwrap();

        assert_eq!(contract.profiles.len(), 0, "wrong profiles len");
        let profile = Profile::new(&profile_id);
        contract.internal_profile_save(&profile);
        assert_eq!(contract.profiles.len(), 1, "wrong profiles len");
    }

    #[test]
    #[should_panic(expected="internal_profile_save: profile already exists")]
    fn test_api_profile_internal_save_fail_already_exists() {
        let mut contract = build_contract();
        let profile_id: ProfileId = "bob".parse().unwrap();
        let profile = Profile::new(&profile_id);
        contract.internal_profile_save(&profile);
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
        let profile_id: ProfileId = "bob".parse().unwrap();

        testing_env!(get_context_view(to_ts(11)));
        let profile = contract.profile_get(profile_id.clone());
        assert_eq!(profile.profile_id, profile_id, "wrong profile_id");
    }

    #[test]
    fn profile_get_present() {
        let mut contract = build_contract();
        let profile_id: ProfileId = "bob".parse().unwrap();
        let mut profile = Profile::new(&profile_id);
        let rewards_available: Balance = to_yocto("3");
        let rewards_claimed: Balance = to_yocto("0");
        profile.rewards_transfer(rewards_available);
        contract.internal_profile_save(&profile);

        testing_env!(get_context_view(to_ts(11)));
        let response: ProfileView = contract.profile_get(profile_id.clone());
        assert_eq!(
            response.rewards_available,
            rewards_available.into(),
            "wrong rewards_available",
        );
        assert_eq!(
            response.rewards_claimed,
            rewards_claimed.into(),
            "wrong rewards_claimed",
        );
    }
}
