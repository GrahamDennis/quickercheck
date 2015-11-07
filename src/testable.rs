use generate::{Generator, GenerateCtx, Constant, Map};
use arbitrary::Arbitrary;
use std::convert::{Into, From};

use rand;

#[derive(Copy, Clone)]
pub struct TestResult {
    status: Status,
}

#[derive(Copy, Clone)]
pub enum Status { Pass, Fail, Discard }

trait Testable<Args> {
    fn test(&self, Args) -> TestResult;
}

#[derive(Copy, Clone)]
struct Empty;

impl Arbitrary for Empty {
    type ArbitraryGenerator = Constant<Empty>;

    fn arbitrary() -> Self::ArbitraryGenerator {
        Constant(Empty)
    }
}

pub trait IntoTestable<Args> {
    type Testable: Testable<Args>;

    fn into_testable(self) -> Self::Testable;
}

impl <T: Testable<Args>, Args> IntoTestable<Args> for T
{
    type Testable = Self;
    #[inline]
    fn into_testable(self) -> Self {
        self
    }
}

impl <T: Clone + Into<TestResult>> Testable<Empty> for T {
    #[inline]
    fn test(&self, _: Empty) -> TestResult {
        self.clone().into()
    }
}

impl From<u8> for TestResult {
    #[inline]
    fn from(_: u8) -> TestResult {
        TestResult { status: Status::Pass }
    }
}

#[derive(Copy, Clone)]
pub struct CheckResult;

fn qckchk<Args: Arbitrary, T: IntoTestable<Args>>(t: T) -> CheckResult
{
    let testable = t.into_testable();
    new_quickcheck(testable)
}

fn new_quickcheck<Args: Arbitrary, T>(t: T) -> CheckResult
    where T: Testable<Args>
{
    let g = Args::arbitrary();
    new_quickcheck_with_gen(t, g)
}

fn new_quickcheck_with_gen<Args, T: Testable<Args>, G: Generator<Output=Args>>(t: T, g: G) -> CheckResult {
    let mut ctx = GenerateCtx { rng: rand::thread_rng(), size: 5 };
    let args = g.generate(&mut ctx);
    let test_result: TestResult = t.test(args);
    CheckResult
}

struct FnArgs<Args>(Args);

impl <G: Generator> Generator for FnArgs<G> {
    type Output = FnArgs<G::Output>;

    #[inline]
    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> Self::Output {
        FnArgs(self.0.generate(ctx))
    }
}

impl <Args: Arbitrary> Arbitrary for FnArgs<Args> {
    type ArbitraryGenerator = FnArgs<Args::ArbitraryGenerator>;

    #[inline]
    fn arbitrary() -> Self::ArbitraryGenerator {
        FnArgs(Args::arbitrary())
    }
}

#[cfg(feature = "no_function_casts")]
mod unstable {
    use super::{TestResult, Testable, FnArgs};

    impl <F: Fn<Args, Output=T>, T: Into<TestResult>, Args> Testable<FnArgs<Args>> for F
    {
        #[inline]
        fn test(&self, args: FnArgs<Args>) -> TestResult {
            self.call(args.0).into()
        }
    }
}

#[cfg(not(feature = "no_function_casts"))]
mod stable {
    use super::{TestResult, Testable, FnArgs};

    impl <Args, T: Into<TestResult>> Testable<FnArgs<(Args,)>> for fn(Args) -> T {
        #[inline]
        fn test(&self, args: FnArgs<(Args,)>) -> TestResult {
            self((args.0).0).into()
        }
    }

    impl <T: Into<TestResult>> Testable<FnArgs<()>> for fn() -> T {
        #[inline]
        fn test(&self, _: FnArgs<()>) -> TestResult {
            self().into()
        }
    }
}

pub fn main() {
    qckchk(TestResult { status: Status::Pass });

    fn my_prop(_: usize) -> TestResult {
        TestResult { status: Status::Pass }
    }
    qckchk(my_prop as fn(usize) -> TestResult);

    fn my_prop2(_: usize) -> u8 {
        3u8
    }
    qckchk(my_prop2 as fn(usize) -> u8);

    let _ = qckchk(6u8);
}

#[cfg(feature = "no_function_casts")]
fn unstable_checks() {
    qckchk(TestResult { status: Status::Pass });
    qckchk(|| TestResult { status: Status::Pass });
    fn my_prop(_: usize) -> TestResult {
        TestResult { status: Status::Pass }
    }
    qckchk(my_prop);
    qckchk(|_:usize| TestResult { status: Status::Pass });
    let _ = qckchk(6u8);
}
