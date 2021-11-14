# Marketplace contract

## Types

```
AccountId: String
Base58PublicKey: String
Balance: String,
Option<Type>: Type or null
LotStatus: Enum(“ACTIVE”, “FINISHED_SUCCESS”, “FINISHED_FAIL”)
WrappedTimestamp: String
```

## Account lifecycle

1. offer - lot offers itself to the market
2. configure - lot deploys lock contract, removes all access keys
3. bid - other accounts bid rising lot price
4. finish - auction time ends, winner is determined
4. claim - winner claims the lot by adding it's access key to it
5. cancel - if no bids were made beneficiary can regain access to lot
6. collect rewards - marketplace transfers rewards to users

## Endpoints

### lot_offer

Puts account on sale

Caller: lot account
Balance: some amount to kick off DDoSers
```
lot_offer( // lot_id is current user
  beneficiary_id: AccountId, // who gets money after lot is sold
  reserve_price: Option<Balance>, // minimum sale price
  buy_now_price: Option<Balance>, // price at which auction ends autmatically
  finish_time: Option<WrappedTimestamp>, // timeout for auction,
)
```
Response: true or error

### lot_bid

Adds bid to specific lot

Caller: bidder account
Balance: bid amount
```
lot_bid(
  lot_id: AccountId,
)
```
Response: true or error

### lot_claim

Transfers control to the winner by adding it's access key to account

Caller: latest bidder
Balance: none
```
lot_claim(
  lot_id: AccountId,
  publicKey: Base58PublicKey,
)
```
Response: true or error

### lot_cancel

Returns account to the seller, only if no bids were made

Caller: beneficiary
Balance: none
lot_cancel(
  lot_id: AccountId,
  publicKey: Base58PublicKey,
)
Response: true or error

### profile_rewards_collect

Transfers received rewards to the caller

Caller: anyone
Balance: none
```
profile_rewards_collect(
)
```
Response: Balance (transferred balance)

### bid_list

List all bids for specified lot.

Caller: anyone, it's a view method
```
bid_list(
  lot_id: AccountId,
)
```
Response:
```
Vector<{
  bidder_id: AccountId,
  amount: Balance,
  timestamp: WrappedTimestamp,
}>
```

### lot_list

Lists all available lots, optionally filtered by lot_id

Caller: anyone, it's a view method
```
lot_list(
  lot_id: Option(AccountId)
)
```
Response:
```
Vector<{
  lot_id: AccountId,
  beneficiary_id: AccountId,
  reserve_price: Option<Balance>,
  buy_now_price: Option<Balance>,
  finish_time: Option<WrappedTimestamp>,
  status: LotStatus,
  latest_bid: Balance,
  next_bid: Balance, // latest_bid_price plus minimal allowed step
})
```

### profile_get

Gets profile info for profile

Caller: anyone, it's a view method
```
profile_get(
  profile_id: AccountId
)
```

Response:
```
{
  lots: Vector<ProfileId>,
  lots_participating: Vector<ProfileId>,
  lots_acquired: Vector<ProfileId>,
  available_rewards: Balance,
  profit_received: Balance,
}
```
