use crate::*;

pub const ERR_PROFILE_REWARDS_CLAIM_NOT_ENOUGH: &str = "Not enough rewards for transfer";
pub const MIN_PROFILE_REWARDS_CLAIM_AMOUNT: Balance = 200 * 10u128.pow(21);
pub const GAS_EXT_CALL_AFTER_REWARDS_COLLECT: u64 = 100_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Profile {
    pub profile_id: ProfileId,
    pub rewards_available: Balance,
    pub rewards_claimed: Balance,

    pub lots_offering: UnorderedSet<LotId>,
    pub lots_bidding: UnorderedSet<LotId>,
}

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
        self.profiles.remove(&profile_id).unwrap_or_else(|| {
            let mut prefix_offering: Vec<u8> = Vec::with_capacity(33);
            prefix_offering.extend(PREFIX_PROFILE_LOTS_OFFERING.as_bytes());
            prefix_offering.extend(env::sha256(profile_id.as_bytes()));

            let mut prefix_bidding: Vec<u8> = Vec::with_capacity(33);
            prefix_bidding.extend(PREFIX_PROFILE_LOTS_BIDDING.as_bytes());
            prefix_bidding.extend(env::sha256(profile_id.as_bytes()));

            Profile {
                profile_id: profile_id.clone(),
                rewards_available: 0,
                rewards_claimed: 0,
                lots_offering: UnorderedSet::new(prefix_offering),
                lots_bidding: UnorderedSet::new(prefix_bidding),
            }
        })
    }

    pub(crate) fn internal_profile_save(&mut self, profile: &Profile) {
        assert!(self.profiles.insert(&profile.profile_id, profile).is_none());
    }

    pub(crate) fn internal_profile_rewards_transfer(
        &mut self,
        profile_id: &ProfileId,
        value: Balance,
    ) {
        let mut profile = self.internal_profile_extract(profile_id);
        profile.rewards_available += value;
        self.internal_profile_save(&profile);
    }
}

#[near_bindgen]
impl Contract {
    pub fn profile_get(&self, profile_id: AccountId) -> Option<ProfileView> {
        self.profiles.get(&profile_id).map(|p| (&p).into())
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
                GAS_EXT_CALL_AFTER_REWARDS_COLLECT.into(),
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
