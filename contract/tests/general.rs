use near_sdk::serde_json::json;
use near_sdk::{AccountId, Balance};
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount, STORAGE_AMOUNT, DEFAULT_GAS};

use marketplace::{ContractContract, LotView, ERR_LOT_SELLS_SELF};

pub const CONTRACT_BYTES: &[u8] = include_bytes!("../res/marketplace.wasm");
pub const LOCK_CONTRACT_BYTES: &[u8] = include_bytes!("../../lock_unlock_account_contract/res/lock_unlock_account.wasm");
const DEFAULT_PUBLIC_KEY: &str = "ed25519:Ga6C8S7jVG2inG88cos8UsdtGVWRFQasSdTdtHL7kBqL";

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
        STORAGE_AMOUNT // attached deposit
    );

    let alice = root.create_user(
        "alice".parse().unwrap(),
        to_yocto("100") // initial balance
    );

    (root, contract, alice)
}

// useless for now, will be helpful later
fn create_user(root: &UserAccount, name: &str) -> UserAccount {
    root.create_user(name.parse().unwrap(), to_yocto("10"))
}

const DAY_NANOSECONDS: u64 = 10u64.pow(9) * 60 * 60 * 24;

#[test]
fn simulate_lot_offer_self() {
    let (root, contract) = init();
    let alice: UserAccount = create_user(&root, "alice");
    // let bob: UserAccount = create_user(&root, "bob");

    let account_id: AccountId = alice.account_id.clone();

    let result = call!(
        alice,
        contract.lot_offer(
            alice.account_id.clone(),
            to_yocto("3").into(),
            to_yocto("10").into(),
            DAY_NANOSECONDS * 10
        )
    );
    assert!(format!("{:?}", result.status()).contains(ERR_LOT_SELLS_SELF));
    assert!(!result.is_ok(), "Should panic");
}

#[test]
fn simulate_lock_unlock() {
    let (root, contract, alice) = init_locked();

    let result = contract.call(
        contract.account_id(),
        "lock",
        &json!({
            "owner_id": "alice".to_string(),
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        0,
    );
    assert!(result.is_ok());

    let result: String = root.view(
        contract.account_id(),
        "get_owner",
        &json!({}).to_string().into_bytes(),
    ).unwrap_json();
    assert_eq!(result, "alice".to_string(), "expected owner alice");

    let result = alice.call(
        contract.account_id(),
        "unlock",
        &json!({
            "public_key": DEFAULT_PUBLIC_KEY.to_string(),
        }).to_string().into_bytes(),
        DEFAULT_GAS,
        0,
    );
    assert!(result.is_ok());

    println!("{:?}", contract.account());
}
