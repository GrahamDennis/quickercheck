use quick_check::{
    quickcheck,
    quicktest
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
    quicktest(prop as fn() -> bool);
    // match quicktest(prop as fn() -> bool) {
    //     Ok(n) => panic!("prop_oob should fail with a runtime error \
    //                      but instead it passed {} tests.", n),
    //     _ => return
    // }
}

#[test]
fn reverse_single() {
    let prop = Property::<(Vec<usize>,)>
        ::when(|xs| xs.len() == 1)
        .property(|xs| xs == xs.clone().into_iter().rev().collect::<Vec<_>>())
        // ::new(|xs| xs == xs.clone().into_iter().rev().collect::<Vec<_>>())
        .resize(|_| 1);

    quickcheck(prop);
}
