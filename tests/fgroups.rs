//! Groups as values: orders, classes, quotients, Sylow subgroups.

use galileo_rs::Output;
use galileo_rs::eval::Session;
use galileo_rs::print::show;

fn one(code: &str) -> String {
    let mut s = Session::new();
    let mut last = String::new();
    for r in s.run(code) {
        last = match r {
            Ok(Output::Value(v)) => show(&v),
            Ok(o) => format!("{:?}", o),
            Err(e) => format!("ERROR {}", e),
        };
    }
    last
}

#[test]
fn orders_and_properties() {
    for (code, want) in [
        ("order(sym(5))", "120"),
        ("order(alt(5))", "60"),
        ("order(dihedral(6))", "12"),
        ("order(units(20))", "8"),
        ("order(group(\"Q8\"))", "8"),
        ("order(dprod(cyclic(2), cyclic(3)))", "6"),
        ("order(center(dihedral(4)))", "2"),
        ("order(center(group(\"Q8\")))", "2"),
        ("order(derived(sym(4)))", "12"),
        ("order(sylow(sym(4), 2))", "8"),
        ("order(sylow(alt(5), 5))", "5"),
        ("issimple(alt(5))", "true"),
        ("issimple(alt(4))", "false"),
        ("issolvable(sym(4))", "true"),
        ("issolvable(sym(5))", "false"),
        ("isabelian(units(20))", "true"),
        ("isabelian(group(\"Q8\"))", "false"),
        ("size(classes(sym(5)))", "[7, 5]"),
    ] {
        assert_eq!(one(code), want, "{}", code);
    }
}

#[test]
fn quotients_and_names() {
    let k = "G = sym(4); V = subgroup(G, [2, 1, 4, 3], [3, 4, 1, 2]);";
    assert_eq!(one(&format!("{} isnormal(G, V)", k)), "true");
    assert_eq!(one(&format!("{} order(quotient(G, V))", k)), "6");
    assert_eq!(one(&format!("{} isabelian(quotient(G, V))", k)), "false");
    assert!(one("quotient(sym(3), subgroup(sym(3), [2, 1, 3]))").starts_with("ERROR"));
    // a group prints as the call that rebuilds it
    let shown = one("sylow(sym(4), 2)");
    assert_eq!(one(&format!("order({})", shown)), "8");
    assert_eq!(
        one("dprod(cyclic(2), cyclic(3))"),
        "dprod(cyclic(2), cyclic(3))"
    );
}
