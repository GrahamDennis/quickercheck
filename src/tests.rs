use quick_check::{
    quickcheck,
    quicktest,
    QuickCheck,
    QuickCheckError
};

use property::{
    Property
};

use testable::{
    IntoTestable
};

#[test]
fn prop_reverse_reverse() {
    fn prop(input: Vec<u8>) -> bool {
        let mut revrev = input.clone();
        revrev.reverse();
        revrev.reverse();
        revrev == input
    }
    quickcheck(prop as fn(Vec<u8>) -> bool);
}

#[test]
#[should_panic]
fn prop_oob() {
    fn prop() -> bool {
        let zero: Vec<bool> = vec![];
        zero[0]
    }
    quickcheck(prop as fn() -> bool);
}

#[test]
fn reverse_single() {
    let prop = Property::<(Vec<usize>,)>
        ::when(|xs| xs.len() == 1)
        .property(|xs| xs == xs.clone().into_iter().rev().collect::<Vec<_>>())
        .resize(|_| 5);

    quickcheck(prop);
}

#[test]
fn reverse_app() {
    fn prop(xs: Vec<usize>, ys: Vec<usize>) -> bool {
        let mut app = xs.clone();
        app.extend(ys.iter().cloned());
        let app_rev: Vec<usize> = app.into_iter().rev().collect();

        let rxs: Vec<usize> = xs.into_iter().rev().collect();
        let mut rev_app = ys.into_iter().rev().collect::<Vec<usize>>();
        rev_app.extend(rxs.into_iter());

        app_rev == rev_app
    }
    quickcheck(prop as fn(Vec<usize>, Vec<usize>) -> bool);
}

#[test]
fn max() {
    let prop = Property::<(isize, isize)>
        ::when(|x, y| x <= y)
        .property(|x, y| ::std::cmp::max(x, y) == y);

    quickcheck(prop);
}

#[test]
fn sort() {
    fn prop(mut xs: Vec<isize>) -> bool {
        xs.sort_by(|x, y| x.cmp(y));
        let upto = if xs.len() == 0 { 0 } else { xs.len()-1 };
        for i in 0..upto {
            if xs[i] > xs[i+1] {
                return false
            }
        }
        true
    }
    quickcheck(prop as fn(Vec<isize>) -> bool);
}

fn sieve(n: usize) -> Vec<usize> {
    if n <= 1 {
        return vec![];
    }

    let mut marked = vec![false; n+1];
    marked[0] = true;
    marked[1] = true;
    marked[2] = true;
    for p in 2..n {
        for i in (2*p..n).filter(|&n| n % p == 0) {
            marked[i] = true;
        }
    }
    marked.iter()
          .enumerate()
          .filter_map(|(i, &m)| if m { None } else { Some(i) })
          .collect()
}

fn is_prime(n: usize) -> bool {
    n != 0 && n != 1 && (2..).take_while(|i| i*i <= n).all(|i| n % i != 0)
}

#[test]
fn sieve_not_prime() {
    let prop_all_prime = Property::<(usize,)>
        ::new(|n| sieve(n).into_iter().all(is_prime))
        .expect_failure();
    quickcheck(prop_all_prime);
}

#[test]
fn sieve_not_all_primes() {
    let prop_prime_iff_in_the_sieve = Property::<(usize,)>
        ::new(|n| sieve(n) == (0..(n + 1)).filter(|&i| is_prime(i)).collect::<Vec<_>>())
        .expect_failure();
    quickcheck(prop_prime_iff_in_the_sieve);
}

#[test]
fn testable_result() {
    let prop = Property::<()>::new(|| -> Result<bool, String> { Ok(true) });
    quickcheck(prop);
}

#[test]
#[should_panic]
fn testable_result_err() {
    quickcheck(Err::<bool, i32> as fn(i32) -> Result<bool, i32>);
}

#[test]
fn testable_result_err2() {
    quickcheck((Err::<bool, i32> as fn(i32) -> Result<bool, i32>).expect_failure());
}

#[test]
fn testable_unit() {
    fn do_nothing() {}
    quickcheck(do_nothing as fn());
}

#[test]
#[should_panic]
fn failing_property() {
    fn all_equal1(x: isize, y: isize, z: isize) -> bool { (x == y) && (y == z) }
    fn all_equal2(x: isize, y: isize, z: isize) -> bool { 2*x == y + z }

    let prop = Property::<(isize, isize, isize)>
        ::new(|x, y, z| all_equal1(x, y, z) == all_equal2(x, y, z));

    QuickCheck::new().max_size(5).quickcheck(prop);
}

#[test]
fn failing_reverse_combine() {
    fn concat(xs: Vec<isize>, ys: Vec<isize>) -> Vec<isize> {
        xs.into_iter().chain(ys.into_iter()).collect()
    }
    fn reverse(xs: Vec<isize>) -> Vec<isize> {
        xs.into_iter().rev().collect()
    }

    fn reverse_combine(xs: Vec<isize>, ys: Vec<isize>) -> bool {
        reverse(concat(xs.clone(), ys.clone())) == concat(reverse(xs.clone()), reverse(ys.clone()))
    }

    let result = quicktest(reverse_combine as fn(Vec<isize>, Vec<isize>) -> bool);
    match result {
        Err(QuickCheckError::Failure {ref input, .. }) =>
            assert!(input == "([0], [1])" || input == "([1], [0])", "Didn't get the expected shrunk result.  Instead found {:?}", result),
        _ => assert!(false, "Test didn't fail: {:?}", result)
    }
}
