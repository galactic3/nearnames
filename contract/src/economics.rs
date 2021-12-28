use crate::*;

pub fn calc_lot_bid_rewards(
    prev_amount: Option<Balance>,
    amount: Balance,
    seller_rewards_commission: Fraction,
    prev_bidder_commission_share: Fraction,
) -> (Option<Balance>, Balance) {
    match prev_amount {
        Some(prev_amount) => {
            let to_prev_bidder_bid = prev_amount;
            let to_seller = amount - to_prev_bidder_bid;
            let commission = seller_rewards_commission * to_seller;
            let to_seller = to_seller - commission;
            let to_prev_bidder_reward = prev_bidder_commission_share * commission;
            let to_prev_bidder = to_prev_bidder_bid + to_prev_bidder_reward;

            (Some(to_prev_bidder), to_seller)
        }
        None => {
            let to_seller = amount;
            let commission = seller_rewards_commission * to_seller;
            let to_seller = to_seller - commission;
            (None, to_seller)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk_sim::to_yocto;

    #[test]
    fn test_economics_calc_lot_bid_rewards_zero_commissions() {
        let z = Fraction::new(0, 1);

        let (to_prev_bidder, to_seller) = calc_lot_bid_rewards(None, to_yocto("10"), z, z);
        assert_eq!(to_prev_bidder, None);
        assert_eq!(to_seller, to_yocto("10"));

        let (to_prev_bidder, to_seller) =
            calc_lot_bid_rewards(Some(to_yocto("10")), to_yocto("15"), z, z);
        assert_eq!(to_prev_bidder, Some(to_yocto("10")));
        assert_eq!(to_seller, to_yocto("5"));
    }

    #[test]
    fn test_economics_calc_lot_bid_rewards_nonzero_commissions() {
        let c = Fraction::new(1, 10);
        let cs = Fraction::new(4, 5);

        let (to_prev_bidder, to_seller) = calc_lot_bid_rewards(None, to_yocto("10"), c, cs);
        assert_eq!(to_prev_bidder, None);
        assert_eq!(to_seller, to_yocto("9"));

        let (to_prev_bidder, to_seller) =
            calc_lot_bid_rewards(Some(to_yocto("10")), to_yocto("15"), c, cs);
        assert_eq!(to_prev_bidder, Some(to_yocto("10.4")));
        assert_eq!(to_seller, to_yocto("4.5"));
    }
}
