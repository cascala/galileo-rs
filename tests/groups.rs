use galileo_rs::groups::*;
use galileo_rs::numtheory as nt;
use num_bigint::BigInt;

#[test]
fn class_numbers_and_composition() {
    // h(D) for some fundamental discriminants
    for (d, h) in [
        (-3i64, 1usize),
        (-4, 1),
        (-23, 3),
        (-47, 5),
        (-71, 7),
        (-84, 4),
        (-163, 1),
        (-1564, 14),
    ] {
        let fs = reduced_forms(d);
        assert_eq!(fs.len(), h, "h({})", d);
        for f in &fs {
            for g in &fs {
                let c = f.compose(g);
                assert_eq!(c.disc(), BigInt::from(d));
                assert!(
                    fs.contains(&c),
                    "{} * {} = {} not reduced",
                    f.text(),
                    g.text(),
                    c.text()
                );
            }
            assert!(f.pow(&BigInt::from(h as u64)).is_identity());
        }
    }
}

#[test]
fn group_factoring_methods() {
    let p = BigInt::from(1000000000039u64);
    let q = BigInt::from(10000000000037u64);
    let n = &p * &q;
    assert_eq!(squfof(&n).map(|x| x.0), Some(p.clone()));
    let f = class_factor(&n, 50_000, 3).unwrap().0;
    assert!(f == p || f == q);
    let m = BigInt::from(2).pow(67) - 1;
    assert_eq!(pm1(&m, 3000).0, Some(BigInt::from(193707721u64)));
    let p = nt::next_prime(&BigInt::from(7_000_000_000_000u64));
    let q = nt::next_prime(&BigInt::from(10u64).pow(40));
    assert_eq!(ecm(&(&p * &q), 100, 2000).map(|x| x.0), Some(p));
}
