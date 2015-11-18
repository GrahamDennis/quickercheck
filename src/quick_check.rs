use generate::GenerateCtx;
use testable::{IntoTestable, Testable, Status};

use std;
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
    rng: rand::ThreadRng
}

impl QuickCheck
{
    pub fn new() -> Self {
        QuickCheck {
            tests: 100,
            max_discard_ratio: 10,
            rng: rand::thread_rng()
        }
    }

    pub fn quicktest<T: IntoTestable>(&mut self, t: T) -> Result<usize> {
        let testable = t.into_testable();
        let max_tests = self.tests * self.max_discard_ratio;

        let mut successful_tests: usize = 0;
        for _ in 0..max_tests {
            if successful_tests >= self.tests {
                return Ok(successful_tests);
            }
            let seed = self.rng.gen();
            let mut test_rng = StdRng::from_seed(&[seed]);
            let size = self.tests;
            let mut ctx = GenerateCtx::new(&mut test_rng, size);

            let mut result = testable.test(&mut ctx);
            match result.status {
                Status::Pass => successful_tests += 1,
                Status::Discard => continue,
                Status::Fail => return Err(QuickCheckError::Failure { seed: seed, size:  size })
            }
        }

        Err(QuickCheckError::GaveUp {
            successful_tests: successful_tests,
            attempts: max_tests
        })
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
