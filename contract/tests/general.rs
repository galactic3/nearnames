use near_sdk::serde_json::json;
use near_sdk::Balance;
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount, DEFAULT_GAS,
    STORAGE_AMOUNT,
};

use marketplace::{ContractContract, LotView, ProfileView, Fraction};

// not using lazy static because it breaks my language server
pub const CONTRACT_BYTES: &[u8] = include_bytes!("../res/marketplace.wasm");
pub const LOCK_CONTRACT_BYTES: &[u8] =
    include_bytes!("../../lock_unlock_account_contract/res/lock_unlock_account.wasm");
const DEFAULT_PUBLIC_KEY: &str = "ed25519:Ga6C8S7jVG2inG88cos8UsdtGVWRFQasSdTdtHL7kBqL";

fn from_yocto(amount: Balance) -> String {
    let yocto_in_near: Balance = 10u128.pow(24);
    let fraction = amount % yocto_in_near;
    let whole = amount / yocto_in_near;
    format!("{}.{:024}", whole, fraction)
}

// near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
//     COUNTER_BYTES => "res/marketplace.wasm",
// }

fn init() -> (UserAccount, ContractAccount<ContractContract>) {
    let root = init_simulator(None);

    // Deploy the compiled Wasm bytes
    let counter: ContractAccount<ContractContract> = deploy!(
        contract: ContractContract,
        contract_id: "marketplace".to_string(),
        bytes: &CONTRACT_BYTES,
        signer_account: root
    );

    (root, counter)
}

fn init_locked() -> (UserAccount, UserAccount, UserAccount) {
    // Use `None` for default genesis configuration; more info below
    let root = init_simulator(None);

    let contract = root.deploy(
        &LOCK_CONTRACT_BYTES,
        "locked".parse().unwrap(),
        STORAGE_AMOUNT, // attached deposit
    );

    let alice = root.create_user(
        "alice".parse().unwrap(),
        to_yocto("100"), // initial balance
    );

    (root, contract, alice)
}

// useless for now, will be helpful later
fn create_user(root: &UserAccount, name: &str) -> UserAccount {
    root.create_user(name.parse().unwrap(), to_yocto("100"))
}

fn create_user_locked(root: &UserAccount, name: &str) -> UserAccount {
    let alice = root.deploy(
        &LOCK_CONTRACT_BYTES,
        name.parse().unwrap(),
        STORAGE_AMOUNT, // attached deposit
    );
    let result = alice.call(
        alice.account_id(),
        "lock",
        &json!({ "owner_id": "marketplace".to_string() })
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    assert!(result.is_ok());

    alice
}

const DAY_NANOSECONDS: u64 = 10u64.pow(9) * 60 * 60 * 24;

fn subtract_seller_reward_commission(reward: Balance, commission: Fraction) -> Balance {
    reward - commission * reward
}

#[test]
fn simulate_lot_offer_buy_now() {
    let (root, contract) = init();
    let alice: UserAccount = create_user_locked(&root, "alice");
    let bob: UserAccount = create_user(&root, "bob");
    let carol: UserAccount = create_user(&root, "carol");

    let commission = Fraction::new(1, 8);

    let balance_to_reserve = to_yocto("0.002");
    root.transfer(bob.account_id(), balance_to_reserve); // storage and future gas
    bob.transfer(root.account_id(), to_yocto("100")); // storage and future gas

    let result = call!(
        alice,
        contract.lot_offer(
            bob.account_id.clone(),
            to_yocto("3").into(),
            to_yocto("10").into(),
            (DAY_NANOSECONDS * 10).into()
        )
    );
    assert!(result.is_ok());

    let result = call!(
        carol,
        contract.lot_bid(alice.account_id.clone()),
        deposit = to_yocto("10")
    );
    assert!(result.is_ok());

    let result = view!(contract.lot_list());
    assert!(result.is_ok());

    let result: Vec<LotView> = result.unwrap_json();
    let result: &LotView = result.get(0).unwrap();

    assert_eq!(
        result.is_active, false,
        "expected lot inactive after buy now bid"
    );
    assert_eq!(
        result.last_bid_amount,
        Some(to_yocto("10").into()),
        "expected last bid 10 near"
    );
    assert_eq!(result.next_bid_amount, None, "expected next bid none");

    let result = call!(
        carol,
        contract.lot_claim(alice.account_id(), DEFAULT_PUBLIC_KEY.parse().unwrap())
    );
    assert!(result.is_ok());

    let result = view!(contract.lot_list());
    assert!(result.is_ok());
    let result: Vec<LotView> = result.unwrap_json();
    assert!(
        result.is_empty(),
        "Expected empty lot list after cleanup callback"
    );

    let result = view!(contract.lot_list_offering_by(bob.account_id()));
    assert!(result.is_ok());
    let result: Vec<LotView> = result.unwrap_json();
    assert!(
        result.is_empty(),
        "Expected empty lot list offering after claim"
    );

    let result = view!(contract.lot_list_bidding_by(carol.account_id()));
    assert!(result.is_ok());
    let result: Vec<LotView> = result.unwrap_json();
    assert!(
        result.is_empty(),
        "Expected empty lot list bidding after claim"
    );

    let result = view!(contract.profile_get(bob.account_id()));
    assert!(result.is_ok());
    let result: Option<ProfileView> = result.unwrap_json();
    let result = result.unwrap();

    let seller_rewards = subtract_seller_reward_commission(to_yocto("10"), commission.clone());
    assert_eq!(Balance::from(result.rewards_available), seller_rewards, "wrong seller reward");

    root.transfer(bob.account_id(), to_yocto("0.2")); // storage and future gas
    let result = call!(bob, contract.profile_rewards_claim());
    assert!(result.is_ok());

    bob.transfer(root.account_id(), seller_rewards);
}

#[test]
fn simulate_lot_offer_withdraw() {
    let (root, contract) = init();
    let alice: UserAccount = create_user_locked(&root, "alice");
    let bob: UserAccount = create_user(&root, "bob");

    let balance_to_reserve = to_yocto("0.002");
    root.transfer(bob.account_id(), balance_to_reserve); // storage and future gas
    bob.transfer(root.account_id(), to_yocto("100")); // storage and future gas

    let result = call!(
        alice,
        contract.lot_offer(
            bob.account_id.clone(),
            to_yocto("3").into(),
            to_yocto("10").into(),
            (DAY_NANOSECONDS * 10).into()
        )
    );
    assert!(result.is_ok());

    let balance_to_reserve = to_yocto("0.2");
    root.transfer(bob.account_id(), balance_to_reserve); // storage and future gas
    let result = call!(bob, contract.lot_withdraw(alice.account_id.clone()));
    assert!(result.is_ok());

    let result = view!(contract.lot_list());
    assert!(result.is_ok());

    let result: Vec<LotView> = result.unwrap_json();
    let result: &LotView = result.get(0).unwrap();

    assert_eq!(
        result.is_active, false,
        "expected lot inactive after withdraw"
    );
    assert_eq!(result.is_withdrawn, true, "expected lot to be withdrawn",);

    let result = call!(
        bob,
        contract.lot_claim(alice.account_id(), DEFAULT_PUBLIC_KEY.parse().unwrap())
    );
    assert!(result.is_ok());

    let result = view!(contract.lot_list());
    assert!(result.is_ok());
    let result: Vec<LotView> = result.unwrap_json();
    assert!(
        result.is_empty(),
        "Expected empty lot list after cleanup callback"
    );

    let result = view!(contract.lot_list_offering_by(bob.account_id()));
    assert!(result.is_ok());
    let result: Vec<LotView> = result.unwrap_json();
    assert!(
        result.is_empty(),
        "Expected empty lot list offering after claim"
    );

    let result = view!(contract.profile_get(bob.account_id()));
    assert!(result.is_ok());
    let result: Option<ProfileView> = result.unwrap_json();
    assert!(
        result.is_some(),
        "profile not created, but should be, because of associations"
    );
    let result = result.unwrap();

    assert_eq!(
        Balance::from(result.rewards_available),
        0,
        "no rewards should be given on withdraw"
    );

    println!("{}", from_yocto(bob.account().unwrap().amount));
}

#[test]
fn simulate_lock_unlock() {
    let (root, contract, alice) = init_locked();

    let result = contract.call(
        contract.account_id(),
        "lock",
        &json!({
            "owner_id": "alice".to_string(),
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    assert!(result.is_ok());

    let result: String = root
        .view(
            contract.account_id(),
            "get_owner",
            &json!({}).to_string().into_bytes(),
        )
        .unwrap_json();
    assert_eq!(result, "alice".to_string(), "expected owner alice");

    let result = alice.call(
        contract.account_id(),
        "unlock",
        &json!({
            "public_key": DEFAULT_PUBLIC_KEY.to_string(),
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    assert!(result.is_ok());

    println!("{:?}", contract.account());
}
