use crate::*;

pub const ERR_LOT_SELLS_SELF: &str = "Expected lot id not equal to seller id";
pub const ERR_LOT_PRICE_RESERVE_GREATER_THAN_BUY_NOW: &str =
    "Expected reserve_price greater or equal buy_now_price";
pub const ERR_LOT_BID_LOT_NOT_ACTIVE: &str = "Expected lot to be active, cannot bid";
pub const ERR_LOT_BID_BID_TOO_SMALL: &str = "Expected bigger bid, try again";
pub const ERR_LOT_BID_SELLER_BIDS_SELF: &str = "Expected bidder_id is not equal to seller_id";
pub const ERR_LOT_CLAIM_LOT_STILL_ACTIVE: &str = "Expected lot to be not active";
pub const ERR_LOT_CLAIM_WRONG_CLAIMER: &str = "This account cannot claim this lot";
pub const ERR_LOT_CLEAN_UP_STILL_ACTIVE: &str = "UNREACHABLE: cannot clean up still active lot";
pub const ERR_LOT_CLEAN_UP_UNLOCK_FAILED: &str = "Expected unlock promise to be successful";
pub const ERR_LOT_WITHDRAW_HAS_BID: &str = "Bid exists, cannot withdraw";
pub const ERR_LOT_WITHDRAW_ALREADY_WITHDRAWN: &str = "Lot already withdrawn";
pub const ERR_LOT_WITHDRAW_NOT_SELLER: &str = "Only seller can withdraw";
pub const ERR_LOT_CLAIM_BY_SELLER_NOT_WITHDRAWN: &str = "Seller cannot claim not withdrwn lot";

pub const NO_DEPOSIT: Balance = 0;
pub const GAS_EXT_CALL_UNLOCK: u64 = 40_000_000_000_000;
pub const GAS_EXT_CALL_CLEAN_UP: u64 = 40_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Bid {
    pub bidder_id: ProfileId,
    pub amount: Balance,
    pub timestamp: Timestamp,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Lot {
    pub lot_id: LotId,
    pub seller_id: ProfileId,
    pub reserve_price: Balance,
    pub buy_now_price: Balance,
    pub start_timestamp: Timestamp,
    pub finish_timestamp: Timestamp,
    pub is_withdrawn: bool,

    pub bids: Vector<Bid>,
}

impl Lot {
    pub fn last_bid(&self) -> Option<Bid> {
        if self.bids.is_empty() {
            None
        } else {
            Some(self.bids.get(self.bids.len() - 1).unwrap())
        }
    }

    pub fn is_active(&self, time_now: Timestamp) -> bool {
        if time_now >= self.finish_timestamp {
            return false;
        }
        if let Some(last_bid_amount) = self.last_bid_amount() {
            if last_bid_amount >= self.buy_now_price {
                return false;
            }
        }
        if self.is_withdrawn {
            return false;
        }

        true
    }

    pub fn last_bid_amount(&self) -> Option<Balance> {
        self.last_bid().map(|x| x.amount)
    }

    pub fn next_bid_amount(&self, time_now: Timestamp) -> Option<Balance> {
        if !self.is_active(time_now) {
            return None;
        }
        if let Some(last_bid_amount) = self.last_bid_amount() {
            Some(std::cmp::max(self.reserve_price, last_bid_amount + 1))
        } else {
            Some(self.reserve_price)
        }
    }

    pub fn potential_claimer_id(&self) -> Option<ProfileId> {
        self.last_bid().map(|x| x.bidder_id)
    }

    pub fn clean_up(&mut self) {
        self.bids.clear()
    }

    pub fn validate_claim_by_buyer(&self, claimer_id: &ProfileId, time_now: Timestamp) {
        assert!(
            !self.is_active(time_now),
            "{}",
            ERR_LOT_CLAIM_LOT_STILL_ACTIVE,
        );
        assert_eq!(
            self.potential_claimer_id().as_ref(),
            Some(claimer_id),
            "{}",
            ERR_LOT_CLAIM_WRONG_CLAIMER,
        );
    }

    pub fn validate_claim_by_seller(&self, claimer_id: &ProfileId) {
        assert!(
            self.is_withdrawn,
            "{}",
            ERR_LOT_CLAIM_BY_SELLER_NOT_WITHDRAWN,
        );
        assert_eq!(
            &self.seller_id, claimer_id,
            "{}",
            ERR_LOT_CLAIM_WRONG_CLAIMER,
        );
    }

    pub fn validate_claim(&self, claimer_id: &ProfileId, time_now: Timestamp) {
        if claimer_id == &self.seller_id {
            self.validate_claim_by_seller(claimer_id)
        } else {
            self.validate_claim_by_buyer(claimer_id, time_now)
        }
    }

    pub fn validate_withdraw(&self, withdrawer_id: &ProfileId) {
        assert_eq!(
            &self.seller_id, withdrawer_id,
            "{}",
            ERR_LOT_WITHDRAW_NOT_SELLER,
        );
        assert!(self.last_bid().is_none(), "{}", ERR_LOT_WITHDRAW_HAS_BID,);
        assert!(!self.is_withdrawn, "{}", ERR_LOT_WITHDRAW_ALREADY_WITHDRAWN,);
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LotView {
    pub lot_id: LotId,
    pub seller_id: ProfileId,
    pub reserve_price: WrappedBalance,
    pub buy_now_price: WrappedBalance,
    pub start_timestamp: WrappedTimestamp,
    pub finish_timestamp: WrappedTimestamp,
    pub last_bid_amount: Option<WrappedBalance>,
    pub next_bid_amount: Option<WrappedBalance>,
    pub is_active: bool,
    pub is_withdrawn: bool,
    pub profile_id: Option<ProfileId>,
    pub profile_role: Option<String>,
    pub profile_status: Option<String>,
    pub profile_action: Option<String>,
}

struct LotProfileRelationInfo {
    pub profile_id: Option<ProfileId>,
    pub profile_role: Option<String>,
    pub profile_status: Option<String>,
    pub profile_action: Option<String>,
}

impl LotProfileRelationInfo {
    pub fn new(lot: &Lot, time_now: Timestamp, profile_id: Option<&ProfileId>) -> Self {
        if let Some(profile_id) = profile_id {
            if &lot.seller_id == profile_id {
                Self::new_offering(lot, time_now, profile_id)
            } else {
                unreachable!();
            }
        } else {
            Self::new_none(profile_id)
        }
    }

    pub fn new_none(profile_id: Option<&ProfileId>) -> Self {
        Self {
            profile_id: profile_id.map(|x| x.clone()),
            profile_role: None,
            profile_status: None,
            profile_action: None,
        }
    }

    pub fn new_offering(lot: &Lot, time_now: Timestamp, profile_id: &ProfileId) -> Self {
        let profile_status = if lot.is_active(time_now) {
            "on_sale"
        } else if lot.is_withdrawn {
            "withdrawn"
        } else {
            match lot.last_bid() {
                Some(_) => "sale_success",
                None => "sale_fail",
            }
        };

        let profile_action = match profile_status {
            "on_sale" => "withdraw",
            "withdrawn" => "claim",
            "sale_success" => "",
            "sale_fail" => "withdraw",
            _ => unreachable!(),
        };

        Self {
            profile_id: Some(profile_id.clone()),
            profile_role: Some("seller".to_string()),
            profile_status: Some(profile_status.to_string()),
            profile_action: Some(profile_action.to_string()),
        }
    }
}

// impl LotView {
//     fn add_profile_info(&mut self, profile_id: Option<&ProfileId>) {
//     }
// }

impl From<(&Lot, Timestamp, Option<&ProfileId>)> for LotView {
    fn from(args: (&Lot, Timestamp, Option<&ProfileId>)) -> Self {
        let (lot, now, profile_id) = args;
        let profile_info = LotProfileRelationInfo::new(lot, now, profile_id);

        Self {
            lot_id: lot.lot_id.clone(),
            seller_id: lot.seller_id.clone(),
            reserve_price: lot.reserve_price.into(),
            buy_now_price: lot.buy_now_price.into(),
            start_timestamp: lot.start_timestamp.into(),
            finish_timestamp: lot.finish_timestamp.into(),
            last_bid_amount: lot.last_bid_amount().map(|x| x.into()),
            next_bid_amount: lot.next_bid_amount(now).map(|x| x.into()),
            is_active: lot.is_active(now),
            is_withdrawn: lot.is_withdrawn,
            profile_id: profile_info.profile_id,
            profile_role: profile_info.profile_role,
            profile_status: profile_info.profile_status,
            profile_action: profile_info.profile_action,
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
    pub(crate) fn internal_lot_create(
        &mut self,
        lot_id: LotId,
        seller_id: ProfileId,
        reserve_price: Balance,
        buy_now_price: Balance,
        time_now: Timestamp,
        duration: Duration,
    ) -> Lot {
        assert_ne!(lot_id, seller_id, "{}", ERR_LOT_SELLS_SELF);
        assert!(
            reserve_price <= buy_now_price,
            "{}",
            ERR_LOT_PRICE_RESERVE_GREATER_THAN_BUY_NOW,
        );

        // TODO: do we still nid to hash the key
        let mut prefix: Vec<u8> = Vec::with_capacity(33);
        prefix.extend(PREFIX_LOTS_BIDS.as_bytes());
        prefix.extend(env::sha256(lot_id.as_bytes()));

        Lot {
            lot_id,
            seller_id,
            reserve_price,
            buy_now_price,
            start_timestamp: time_now,
            finish_timestamp: time_now + duration,
            is_withdrawn: false,
            bids: Vector::new(prefix),
        }
    }

    pub(crate) fn internal_lot_extract(&mut self, lot_id: &LotId) -> Lot {
        self.lots.remove(&lot_id).unwrap()
    }

    pub(crate) fn internal_lot_save(&mut self, lot: &Lot) {
        assert!(self.lots.insert(&lot.lot_id, lot).is_none());
    }

    pub(crate) fn internal_lot_bid(&mut self, lot_id: &LotId, bid: &Bid) {
        let mut lot = self.internal_lot_extract(lot_id);
        assert!(
            lot.is_active(bid.timestamp),
            "{}",
            ERR_LOT_BID_LOT_NOT_ACTIVE
        );
        assert!(
            bid.amount >= lot.next_bid_amount(bid.timestamp).unwrap(),
            "{}",
            ERR_LOT_BID_BID_TOO_SMALL
        );
        assert_ne!(
            lot.seller_id, bid.bidder_id,
            "{}",
            ERR_LOT_BID_SELLER_BIDS_SELF
        );
        assert_ne!(
            lot.lot_id, bid.bidder_id,
            "{}",
            ERR_LOT_BID_SELLER_BIDS_SELF
        );

        lot.bids.push(bid);
        self.internal_lot_save(&lot);
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

    pub fn lot_list(&self) -> Vec<LotView> {
        let now = env::block_timestamp();
        self.lots.values().map(|v| (&v, now, None).into()).collect()
    }

    pub fn lot_list_offered_by(&self, profile_id: ProfileId) -> Vec<LotView> {
        let profile = self.profiles.get(&profile_id).unwrap();
        let time_now = env::block_timestamp();

        profile
            .lots_offering
            .iter()
            .map(|lot_id| {
                let lot = self.lots.get(&lot_id).unwrap();
                let lot_view: LotView = (&lot, time_now, Some(&profile_id)).into();

                lot_view
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

        let lot = self.internal_lot_create(
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
        self.internal_lot_bid(&lot_id, &bid);

        // update associations
        {
            let mut bidder = self.internal_profile_extract(&bidder_id);
            bidder.lots_bidding.insert(&lot_id);
            self.internal_profile_save(&bidder);
        }

        // redistribute balances
        match last_bid {
            Some(last_bid) => {
                let to_last_bid = last_bid.amount;
                let to_seller = amount - to_last_bid;
                self.internal_profile_rewards_transfer(&last_bid.bidder_id, to_last_bid);
                self.internal_profile_rewards_transfer(&lot.seller_id, to_seller);
            }
            None => {
                let to_seller = amount;
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
        lot.clean_up();
        // lot is already deleted from lots storage, returning to persist changes
        true
    }

    pub fn lot_withdraw(&mut self, lot_id: AccountId) -> bool {
        let lot_id: ProfileId = lot_id.into();
        let withdrawer_id: ProfileId = env::predecessor_account_id();
        self.internal_lot_withdraw(&lot_id, &withdrawer_id);
        true
    }
}
