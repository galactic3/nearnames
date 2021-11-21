use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Profile {
    pub profile_id: ProfileId,
    pub rewards_available: Balance,
    pub rewards_claimed: Balance,
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
        self.profiles
            .remove(&profile_id)
            .unwrap_or_else(|| Profile {
                profile_id: profile_id.clone(),
                rewards_available: 0,
                rewards_claimed: 0,
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
}
