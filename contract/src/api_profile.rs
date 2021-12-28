use crate::*;

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
        assert!(self.profiles.insert(&profile.profile_id, profile).is_none());
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

    use near_sdk_sim::to_yocto;

    use crate::tests::build_contract;

    fn test_profile_internal_profile_rewards_transfer() {
        let mut contract = build_contract();
        let profile_id: ProfileId = "alice".parse().unwrap();

        contract.internal_profile_rewards_transfer(&profile_id, to_yocto("3"));
        assert_eq!(contract.profiles.len(), 1);

        let profile = contract.profiles.get(&profile_id).unwrap();
        assert_eq!(profile.rewards_available, to_yocto("3"));

        contract.internal_profile_rewards_transfer(&profile_id, to_yocto("2"));
        let profile = contract.profiles.get(&profile_id).unwrap();
        assert_eq!(profile.rewards_available, to_yocto("5"), "wrong amount");
    }
}