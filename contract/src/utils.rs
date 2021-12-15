use crate::*;

pub fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    match env::promise_result(0) {
        PromiseResult::Successful(_) => true,
        _ => false,
    }
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct Fraction {
    num: u32,
    denom: u32,
    _private: ()
}

impl Fraction {
    pub fn new(num: u32, denom: u32) -> Self {
        assert!(num <= denom, "expected num <= denom");
        assert!(denom > 0, "expected denom > 0");
        Self { num, denom, _private: () }
    }

    pub fn num(&self) -> u32 {
        self.num
    }

    pub fn denom(&self) -> u32 {
        self.denom
    }
}

impl ops::Mul<Balance> for Fraction {
    type Output = Balance;

    fn mul(self, balance: Balance) -> Balance {
        (U256::from(self.num) * U256::from(balance) / U256::from(self.denom)).as_u128()
    }
}
