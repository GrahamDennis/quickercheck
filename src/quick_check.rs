use generate::GenerateCtx;
use testable::{IntoTestable, Testable, TestStatus};

use std::{self, cmp};
use rand::{self, Rng, StdRng, SeedableRng};

pub type Result<T> = std::result::Result<T, QuickCheckError>;

#[derive(Clone, Debug)]
pub enum QuickCheckError {
    GaveUp {
        successful_tests: usize,
        attempts: usize
    },
    Failure {
        seed: usize,
        size: usize
    },
    NoExpectedFailure
}

pub struct QuickCheck {
    tests: usize,
    max_discard_ratio: usize,
    max_size: usize,
    rng: rand::ThreadRng
}

struct QuickCheckState {
    successful_tests: usize,
    recently_discarded_tests: usize
}

impl QuickCheckState {
    fn new() -> Self {
        QuickCheckState { successful_tests: 0, recently_discarded_tests: 0 }
    }

    fn test_passed(&mut self) {
        self.successful_tests += 1;
        self.recently_discarded_tests = 0;
    }

    fn test_discarded(&mut self) {
        self.recently_discarded_tests += 1;
    }

    fn gave_up_after(&self, attempts: usize) -> Result<usize> {
        Err(QuickCheckError::GaveUp {
            successful_tests: self.successful_tests,
            attempts: attempts
        })
    }

    fn test_failed<T: Testable>(&self, testable: T, seed: usize, size: usize) -> Result<usize> {
        match testable.is_expected_to_fail() {
            false => Err(QuickCheckError::Failure { seed: seed, size: size }),
            true => Ok(self.successful_tests)
        }
    }
}

impl QuickCheck
{
    pub fn new() -> Self {
        QuickCheck {
            tests: 100,
            max_discard_ratio: 10,
            max_size: 100,
            rng: rand::thread_rng()
        }
    }

    pub fn max_size(self, max_size: usize) -> Self {
        QuickCheck {
            max_size: max_size,
            ..self
        }
    }

    pub fn quicktest<T: IntoTestable>(&mut self, t: T) -> Result<usize> {
        let testable = t.into_testable();
        let max_tests = self.tests * self.max_discard_ratio;

        let mut state = QuickCheckState::new();

        for _ in 0..max_tests {
            if state.successful_tests >= self.tests { return Ok(state.successful_tests) }

            let seed = self.rng.gen();
            let mut test_rng = StdRng::from_seed(&[seed]);
            let size = self.size(&state);
            let mut ctx = GenerateCtx::new(&mut test_rng, size);

            let result = testable.test(&mut ctx);
            match result.status {
                TestStatus::Pass => state.test_passed(),
                TestStatus::Discard => state.test_discarded(),
                TestStatus::Fail => return state.test_failed(testable, seed, size)
            }
        }

        state.gave_up_after(max_tests)
    }

    fn size(&self, state: &QuickCheckState) -> usize {
        let n = state.successful_tests;
        let d = state.recently_discarded_tests;
        let max_size = self.max_size;

        fn round_down_to(value: usize, multiple: usize) -> usize { (value / multiple) * multiple }

        let proposed_size = {
            if (round_down_to(n, max_size) + max_size <= self.tests) || ((self.tests % max_size) == 0) {
                (n % max_size) + d / 10
            } else {
                ((n % max_size) * max_size) / (self.tests % max_size) + d / 10
            }
        };

        cmp::min(proposed_size, max_size)
    }

    pub fn quickcheck<T: IntoTestable>(&mut self, t: T) {
        let _ = ::env_logger::init();

        match self.quicktest(t) {
            Ok(ntests) => info!("(Passed {} QuickCheck tests.)", ntests),
            Err(err) => panic!("Failed: {:?}.", err)
        }
    }
}

pub fn quicktest<T: IntoTestable>(t: T) -> Result<usize> { QuickCheck::new().quicktest(t) }
pub fn quickcheck<T: IntoTestable>(t: T) { QuickCheck::new().quickcheck(t) }
