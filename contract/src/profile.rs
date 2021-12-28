use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Profile {
    pub profile_id: ProfileId,
    pub rewards_available: Balance,
    pub rewards_claimed: Balance,

    pub lots_offering: UnorderedSet<LotId>,
    pub lots_bidding: UnorderedSet<LotId>,
}

impl Profile {
    pub fn new(profile_id: &ProfileId) -> Profile {
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
    }

    pub fn rewards_transfer(&mut self, amount: Balance) {
        self.rewards_available += amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk_sim::to_yocto;

    use crate::tests::build_contract;

    #[test]
    fn test_profile_new() {
        let profile_id: ProfileId = "bob".parse().unwrap();
        let profile = Profile::new(&profile_id);

        assert_eq!(profile.profile_id, profile_id, "wrong profile_id");
        assert_eq!(profile.rewards_available, to_yocto("0"), "wrong rewards_available");
        assert_eq!(profile.rewards_claimed, to_yocto("0"), "wrong rewards_claimed");
    }

    #[test]
    fn test_profile_rewards_transfer() {
        let profile_id: ProfileId = "bob".parse().unwrap();
        let mut profile = Profile::new(&profile_id);

        profile.rewards_transfer(to_yocto("3"));
        profile.rewards_transfer(to_yocto("2"));
        assert_eq!(profile.rewards_available, to_yocto("5"), "wrong rewards_available");
    }
}
