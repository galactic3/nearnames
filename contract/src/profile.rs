use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Profile {
    pub available_rewards: Balance,
    pub profit_received: Balance,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProfileView {
    pub available_rewards: WrappedBalance,
    pub profit_received: WrappedBalance,
}

impl From<&Profile> for ProfileView {
    fn from(p: &Profile) -> Self {
        Self {
            available_rewards: p.available_rewards.into(),
            profit_received: p.profit_received.into(),
        }
    }
}

impl Contract {
    pub(crate) fn internal_profile_extract_or_create(&mut self, profile_id: &ProfileId) -> Profile {
        self.profiles
            .remove(&profile_id)
            .unwrap_or_else(|| Profile {
                available_rewards: 0,
                profit_received: 0,
            })
    }

    pub(crate) fn internal_profile_save_or_panic(
        &mut self,
        profile_id: &ProfileId,
        profile: &Profile,
    ) {
        assert!(self.profiles.insert(profile_id, profile).is_none());
    }
}

#[near_bindgen]
impl Contract {
    pub fn profile_get(&self, profile_id: ValidAccountId) -> Option<ProfileView> {
        self.profiles.get(profile_id.as_ref()).map(|p| (&p).into())
    }
}
