use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Copy)]
pub struct Fraction {
    num: u32,
    denom: u32,
    _private: (),
}

// TODO: consider removing validations from here and adding to contract init
impl Fraction {
    pub fn new(num: u32, denom: u32) -> Self {
        assert!(num <= denom, "expected num <= denom");
        assert!(denom > 0, "expected denom > 0");
        Self {
            num,
            denom,
            _private: (),
        }
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

#[cfg(test)]
mod tests {
    use crate::tests::*;

    #[test]
    pub fn test_fractions_new() {
        {
            Fraction::new(0, 1);
            Fraction::new(1, 1);
            Fraction::new(7, 13);
            Fraction::new(13, 13);
        }
    }

    #[test]
    #[should_panic(expected = "expected denom > 0")]
    pub fn test_fractions_new_fail_zero_denum() {
        {
            Fraction::new(0, 0);
        }
    }

    #[test]
    #[should_panic(expected = "expected num <= denom")]
    pub fn test_fractions_new_fail_greater_than_one() {
        {
            Fraction::new(2, 1);
        }
    }

    #[test]
    pub fn test_fractions_mul() {
        assert_eq!(
            Fraction::new(0, 13) * 10,
            0,
            "expected zero mul for zero fraction"
        );
        assert_eq!(
            Fraction::new(7, 13) * 0,
            0,
            "expected zero mul for zero balance"
        );
        assert_eq!(
            Fraction::new(13, 13) * 100,
            100,
            "expected same mul one fraction"
        );
        assert_eq!(
            Fraction::new(7, 13) * 100,
            53,
            "expected zero mul for zero balance"
        );
        assert_eq!(Fraction::new(2, 3) * 10, 6, "expected floor rounding");
    }
}
