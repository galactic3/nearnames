use crate::*;

pub fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    matches!(env::promise_result(0), PromiseResult::Successful(_))
}
