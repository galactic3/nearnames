use crate::*;

pub const ERR_PROFILE_INTERNAL_SAVE_ALREADY_EXISTS: &str =
    "internal_profile_save: profile already exists";

pub const LOT_OFFER_MIN_RESERVE_PRICE: Balance = 500 * 10u128.pow(21);
pub const LOT_OFFER_MAX_DURATION: Duration = 90 * 24 * 60 * 60 * 10u64.pow(9);
pub const LOT_REMOVE_UNSAFE_GRACE_DURATION: Duration = 2 * 60 * 60 * 10u64.pow(9);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    profiles: UnorderedMap<ProfileId, Profile>,
    pub lots: UnorderedMap<LotId, Lot>,
    pub seller_rewards_commission: Fraction,
    pub bid_step: Fraction,
    pub prev_bidder_commission_share: Fraction,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct FractionView {
    pub num: u32,
    pub denom: u32,
}

impl From<&Fraction> for FractionView {
    fn from(fraction: &Fraction) -> FractionView {
        FractionView {
            num: fraction.num(),
            denom: fraction.denom(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractConfigView {
    pub seller_rewards_commission: FractionView,
    pub bid_step: FractionView,
    pub prev_bidder_commission_share: FractionView,
    pub lot_offer_min_reserve_price: WrappedBalance,
    pub lot_offer_max_duration: WrappedDuration,
    pub lot_remove_unsafe_grace_duration: WrappedDuration,
}

impl From<&Contract> for ContractConfigView {
    fn from(contract: &Contract) -> ContractConfigView {
        ContractConfigView {
            seller_rewards_commission: (&contract.seller_rewards_commission).into(),
            bid_step: (&contract.bid_step).into(),
            prev_bidder_commission_share: (&contract.prev_bidder_commission_share).into(),
            lot_offer_min_reserve_price: LOT_OFFER_MIN_RESERVE_PRICE.into(),
            lot_offer_max_duration: LOT_OFFER_MAX_DURATION.into(),
            lot_remove_unsafe_grace_duration: LOT_REMOVE_UNSAFE_GRACE_DURATION.into(),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn config_get(&self) -> ContractConfigView {
        self.into()
    }

    #[init(ignore_state)]
    pub fn new(
        seller_rewards_commission: FractionView,
        bid_step: FractionView,
        prev_bidder_commission_share: FractionView,
    ) -> Self {
        Self {
            profiles: UnorderedMap::new(PREFIX_PROFILES.as_bytes().to_vec()),
            lots: UnorderedMap::new(PREFIX_LOTS.as_bytes().to_vec()),
            seller_rewards_commission: Fraction::new(
                seller_rewards_commission.num,
                seller_rewards_commission.denom,
            ),
            bid_step: Fraction::new(bid_step.num, bid_step.denom),
            prev_bidder_commission_share: Fraction::new(
                prev_bidder_commission_share.num,
                prev_bidder_commission_share.denom,
            ),
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

    #[cfg(test)]
    pub(crate) fn profiles_len(&self) -> u64 {
        self.profiles.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;

    #[test]
    fn contract_config_get() {
        let contract = build_contract();

        let config = contract.config_get();
        assert_eq!(
            config.seller_rewards_commission,
            FractionView { num: 1, denom: 10 },
            "wrong seller rewards commission",
        );
        assert_eq!(
            config.bid_step,
            FractionView { num: 1, denom: 5 },
            "wrong bid_step",
        );
        assert_eq!(
            config.prev_bidder_commission_share,
            FractionView { num: 4, denom: 5 },
            "wrong seller rewards commission",
        );
        assert_eq!(
            config.lot_offer_min_reserve_price,
            LOT_OFFER_MIN_RESERVE_PRICE.into(),
            "wrong min reserve price",
        );
        assert_eq!(
            config.lot_offer_max_duration,
            LOT_OFFER_MAX_DURATION.into(),
            "wrong max duration",
        );
        assert_eq!(
            config.lot_remove_unsafe_grace_duration,
            LOT_REMOVE_UNSAFE_GRACE_DURATION.into(),
            "wrong grace_duration",
        );
    }
}

#[cfg(test)]
pub mod tests_profile {
    use crate::tests::*;

    use crate::profile::tests::*;

    pub fn create_contract_with_profile_bob() -> (Contract, ProfileId) {
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
        assert_eq!(contract.profiles_len(), 1, "wrong profiles len");
        assert_eq!(profile.profile_id, profile_id, "wrong rewards_available");
        assert_eq!(
            profile.rewards_available(),
            to_yocto("3"),
            "wrong rewards_available"
        );
        assert_eq!(
            profile.rewards_claimed(),
            to_yocto("2"),
            "wrong rewards_claimed"
        );
    }

    #[test]
    fn test_api_profile_internal_extract() {
        let (mut contract, profile_id) = create_contract_with_profile_bob();

        let profile = contract.internal_profile_extract(&profile_id);
        assert_eq!(contract.profiles_len(), 0, "wrong profiles len");
        assert_eq!(profile.profile_id, profile_id, "wrong rewards_available");
        assert_eq!(
            profile.rewards_available(),
            to_yocto("3"),
            "wrong rewards_available"
        );
        assert_eq!(
            profile.rewards_claimed(),
            to_yocto("2"),
            "wrong rewards_claimed"
        );
    }

    #[test]
    fn test_api_profile_internal_save_success() {
        let (contract, _profile_id) = create_contract_with_profile_bob();
        assert_eq!(contract.profiles_len(), 1, "wrong profiles len");
    }

    #[test]
    #[should_panic(expected = "internal_profile_save: profile already exists")]
    fn test_api_profile_internal_save_fail_already_exists() {
        let (mut contract, profile_id) = create_contract_with_profile_bob();
        let profile = Profile::new(&profile_id);
        contract.internal_profile_save(&profile);
    }
}
