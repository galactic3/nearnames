use crate::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LotView {
    pub lot_id: LotId,
    pub seller_id: ProfileId,
    pub last_bidder_id: Option<ProfileId>,
    pub reserve_price: WrappedBalance,
    pub buy_now_price: WrappedBalance,
    pub start_timestamp: WrappedTimestamp,
    pub finish_timestamp: WrappedTimestamp,
    pub last_bid_amount: Option<WrappedBalance>,
    pub next_bid_amount: Option<WrappedBalance>,
    pub is_active: bool,
    pub is_withdrawn: bool,
    pub status: String,
}

// TODO: convert to regular meethod
impl From<(&Lot, Timestamp, &Contract)> for LotView {
    fn from(args: (&Lot, Timestamp, &Contract)) -> Self {
        let (lot, now, contract) = args;
        let last_bid = lot.last_bid();

        Self {
            lot_id: lot.lot_id.clone(),
            seller_id: lot.seller_id.clone(),
            last_bidder_id: last_bid.as_ref().map(|x| x.bidder_id.clone()),
            reserve_price: lot.reserve_price.into(),
            buy_now_price: lot.buy_now_price.into(),
            start_timestamp: lot.start_timestamp.into(),
            finish_timestamp: lot.finish_timestamp.into(),
            last_bid_amount: last_bid.as_ref().map(|x| x.amount.into()),
            next_bid_amount: lot
                .next_bid_amount(now, contract.bid_step.clone())
                .map(|x| x.into()),
            is_active: lot.is_active(now),
            is_withdrawn: lot.is_withdrawn,
            status: lot.status(now).to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct BidView {
    pub bidder_id: ProfileId,
    pub amount: WrappedBalance,
    pub timestamp: WrappedTimestamp,
}

impl PartialEq for BidView {
    fn eq(&self, other: &Self) -> bool {
        self.bidder_id == other.bidder_id
            && self.amount.0 == other.amount.0
            && self.timestamp.0 == other.timestamp.0
    }
}

impl Eq for BidView {}

impl From<Bid> for BidView {
    fn from(bid: Bid) -> Self {
        Self {
            bidder_id: bid.bidder_id.clone(),
            amount: bid.amount.into(),
            timestamp: bid.timestamp.into(),
        }
    }
}

impl Contract {
    pub(crate) fn internal_lot_extract(&mut self, lot_id: &LotId) -> Lot {
        self.lots.remove(&lot_id).unwrap()
    }

    pub(crate) fn internal_lot_save(&mut self, lot: &Lot) {
        assert!(self.lots.insert(&lot.lot_id, lot).is_none());
    }

    pub(crate) fn internal_lot_withdraw(&mut self, lot_id: &LotId, withdrawer_id: &ProfileId) {
        let mut lot = self.internal_lot_extract(lot_id);
        lot.validate_withdraw(withdrawer_id);
        lot.is_withdrawn = true;
        self.internal_lot_save(&lot);
    }
}

#[near_bindgen]
impl Contract {
    pub fn lot_bid_list(&self, lot_id: AccountId) -> Vec<BidView> {
        let lot: Lot = self.lots.get(&lot_id).unwrap();

        lot.bids.iter().map(|v| v.into()).collect()
    }

    pub fn lot_list(&self, limit: Option<u64>, offset: Option<u64>) -> Vec<LotView> {
        let now = env::block_timestamp();

        let idx_from = offset.unwrap_or(0);
        let idx_to = limit.map (|x| idx_from + x).unwrap_or(u64::MAX);
        let idx_to = std::cmp::min(idx_to, self.lots.len());

        let values_as_vector = self.lots.values_as_vector();

        (idx_from..idx_to)
            .map(|x| {
                let v = values_as_vector.get(x).unwrap();
                (&v, now, self).into()
            }).collect()
    }

    pub fn lot_list_offering_by(&self, profile_id: ProfileId, limit: Option<u64>, offset: Option<u64>) -> Vec<LotView> {
        let profile = self.internal_profile_get(&profile_id);
        let time_now = env::block_timestamp();
        let vector = profile.lots_offering.as_vector();

        let idx_from = offset.unwrap_or(0);
        let idx_to = limit.map (|x| idx_from + x).unwrap_or(u64::MAX);
        let idx_to = std::cmp::min(idx_to, vector.len());

        (idx_from..idx_to)
            .map(|idx| {
                let lot_id = vector.get(idx).unwrap();
                let lot = self.lots.get(&lot_id).unwrap();
                (&lot, time_now, self).into()
            })
            .collect()
    }

    pub fn lot_list_bidding_by(&self, profile_id: ProfileId, limit: Option<u64>, offset: Option<u64>) -> Vec<LotView> {
        let profile = self.internal_profile_get(&profile_id);
        let time_now = env::block_timestamp();
        let vector = profile.lots_bidding.as_vector();

        let idx_from = offset.unwrap_or(0);
        let idx_to = limit.map (|x| idx_from + x).unwrap_or(u64::MAX);
        let idx_to = std::cmp::min(idx_to, vector.len());

        (idx_from..idx_to)
            .map(|idx| {
                let lot_id = vector.get(idx).unwrap();
                let lot = self.lots.get(&lot_id).unwrap();
                (&lot, time_now, self).into()
            })
            .collect()
    }

    pub fn lot_offer(
        &mut self,
        seller_id: AccountId,
        reserve_price: WrappedBalance,
        buy_now_price: WrappedBalance,
        duration: WrappedDuration,
    ) -> bool {
        let lot_id: LotId = env::predecessor_account_id();
        let reserve_price: Balance = reserve_price.into();
        let buy_now_price: Balance = buy_now_price.into();
        let seller_id: ProfileId = seller_id.into();
        let time_now = env::block_timestamp();
        let duration: Duration = duration.into();

        let lot = Lot::new(
            lot_id.clone(),
            seller_id.clone(),
            reserve_price,
            buy_now_price,
            time_now,
            duration,
        );
        self.internal_lot_save(&lot);

        // update associations
        {
            let mut profile = self.internal_profile_extract(&seller_id);
            profile.lots_offering.insert(&lot_id);
            self.internal_profile_save(&profile);
        }

        true
    }

    #[payable]
    pub fn lot_bid(&mut self, lot_id: AccountId) -> bool {
        let lot_id: ProfileId = lot_id.into();
        let lot = self.lots.get(&lot_id).unwrap();
        let last_bid: Option<Bid> = lot.last_bid();

        let bidder_id: ProfileId = env::predecessor_account_id();
        let amount: Balance = env::attached_deposit();
        let timestamp = env::block_timestamp();

        let bid: Bid = Bid {
            bidder_id: bidder_id.clone(),
            amount,
            timestamp,
        };

        // TODO: rewrite to elliminate double read
        {
            let mut lot = self.internal_lot_extract(&lot_id);
            lot.place_bid(&bid, self.bid_step);
            self.internal_lot_save(&lot);
        }

        // update associations
        {
            let mut bidder = self.internal_profile_extract(&bidder_id);
            bidder.lots_bidding.insert(&lot_id);
            self.internal_profile_save(&bidder);
        }

        // redistribute balances
        match last_bid {
            Some(last_bid) => {
                let to_prev_bider = last_bid.amount;
                let to_seller = amount - to_prev_bider;
                let commission = self.seller_rewards_commission * to_seller;
                let to_seller = to_seller - commission;

                let prev_bidder_reward = self.prev_bidder_commission_share * commission;

                self.internal_profile_rewards_transfer(
                    &last_bid.bidder_id,
                    to_prev_bider + prev_bidder_reward,
                );
                self.internal_profile_rewards_transfer(&lot.seller_id, to_seller);
            }
            None => {
                let to_seller = amount;
                let commission = self.seller_rewards_commission * to_seller;
                let to_seller = to_seller - commission;
                self.internal_profile_rewards_transfer(&lot.seller_id, to_seller)
            }
        }

        true
    }

    pub fn lot_claim(&mut self, lot_id: AccountId, public_key: PublicKey) -> Promise {
        let claimer_id: ProfileId = env::predecessor_account_id();
        let time_now = env::block_timestamp();
        let lot: Lot = self.lots.get(&lot_id).unwrap();

        println!("{}", &claimer_id);

        lot.validate_claim(&claimer_id, time_now);

        ext_lock_contract::unlock(
            public_key,
            lot_id.clone(),
            NO_DEPOSIT,
            GAS_EXT_CALL_UNLOCK.into(),
        )
        .then(ext_self_contract::lot_after_claim_clean_up(
            lot_id.clone(),
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_EXT_CALL_CLEAN_UP.into(),
        ))
    }

    #[private]
    pub fn lot_after_claim_clean_up(&mut self, lot_id: LotId) -> bool {
        assert!(is_promise_success(), "{}", ERR_LOT_CLEAN_UP_UNLOCK_FAILED);
        let time_now = env::block_timestamp();
        let mut lot: Lot = self.internal_lot_extract(&lot_id);

        assert!(
            !lot.is_active(time_now),
            "{}",
            ERR_LOT_CLEAN_UP_STILL_ACTIVE
        );
        let bidder_ids_unique: HashSet<ProfileId> = lot.bids.iter().map(|x| x.bidder_id).collect();

        bidder_ids_unique.iter().for_each(|bidder_id| {
            // TODO: validate bid exists
            let mut profile = self.internal_profile_extract(&bidder_id);
            profile.lots_bidding.remove(&lot_id);
            self.internal_profile_save(&profile);
        });
        {
            let mut seller = self.internal_profile_extract(&lot.seller_id);
            seller.lots_offering.remove(&lot_id);
            self.internal_profile_save(&seller);
        }

        lot.clean_up();
        // lot is already deleted from lots storage, returning to persist changes

        // intentionally not inserting the lot back
        true
    }

    pub fn lot_withdraw(&mut self, lot_id: AccountId) -> bool {
        let lot_id: ProfileId = lot_id.into();
        let withdrawer_id: ProfileId = env::predecessor_account_id();
        self.internal_lot_withdraw(&lot_id, &withdrawer_id);
        true
    }
}
