use generate::GenerateCtx;
use testable::{IntoTestable, Testable};

use rand::{self};

#[derive(Clone)]
pub struct CheckResult {
    num_tests: usize,
    status: CheckStatus
}

#[derive(Clone)]
pub enum CheckStatus {
    Success,
    GaveUp,
    Failure {
        num_shrinks: usize,
        used_seed: usize,
        used_size: usize,
        reason: String
    },
    NoExpectedFailure,
    InsufficientCoverage
}

pub fn quicktest<T: IntoTestable>(t: T) -> CheckResult {
    let testable = t.into_testable();
    let mut rng = rand::thread_rng();
    let mut ctx = GenerateCtx { rng: &mut rng, size: 5 };
    let _test_result = testable.test(&mut ctx);
    CheckResult { num_tests: 1 , status: CheckStatus::Success }
}

pub fn quickcheck<T: IntoTestable>(t: T) -> CheckResult
{
    quicktest(t)
}
