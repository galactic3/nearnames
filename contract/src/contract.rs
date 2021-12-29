use crate::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub profiles: UnorderedMap<ProfileId, Profile>,
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
}

impl From<&Contract> for ContractConfigView {
    fn from(contract: &Contract) -> ContractConfigView {
        ContractConfigView {
            seller_rewards_commission: (&contract.seller_rewards_commission).into(),
            bid_step: (&contract.bid_step).into(),
            prev_bidder_commission_share: (&contract.prev_bidder_commission_share).into(),
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
    }
}
