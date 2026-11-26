//! Finite rings and fields as values: zmod(n), gf(p), gf(p^k).

use galileo_rs::Output;
use galileo_rs::eval::Session;
use galileo_rs::ffield as ff;
use galileo_rs::print::show;
use num_bigint::BigInt;

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
fn commands() {
    for (code, want) in [
        ("gf(9)", "gf(9)"),
        ("modulus(gf(9))", "x^2+x+2"),
        ("modulus(gf(16))", "x^4+x+1"),
        ("F = gf(9); F(a + 1)^2", "gf(9, a+2)"),
        ("F = gf(9); x = F(2*a); x * x^-1", "gf(9, 1)"),
        ("order(gf(9, a + 1))", "8"),
        ("order(gf(27))", "27"),
        ("generator(gf(7))", "gf(7, 3)"),
        ("generator(zmod(18))", "zmod(18, 5)"),
        ("generator(zmod(12))", "ERROR"),
        ("zmod(12, 5)^-1", "zmod(12, 5)"),
        ("zmod(12, 4)^-1", "ERROR"),
        ("gf(7, 3) / 5", "gf(7, 2)"),
        ("gf(7, 1/2)", "gf(7, 4)"),
        ("gf(7, 3) == 10", "true"),
        ("gf(7, 3) != gf(7, 4)", "true"),
        ("gf(7, 3) + gf(5, 1)", "ERROR"),
        ("a = 5; gf(9, a)", "gf(9, a)"),
        ("F = gf(101); dlog(F(3), generator(F))", "69"),
        ("G = gf(2^61 - 1); dlog(G(3)^123456789, G(3))", "123456789"),
        (
            "elements(zmod(4))",
            "[zmod(4, 0), zmod(4, 1), zmod(4, 2), zmod(4, 3)]",
        ),
        ("isfield(zmod(12))", "false"),
        ("isfield(zmod(13))", "true"),
        ("characteristic(gf(81))", "3"),
        ("lift(gf(9, 2*a + 1))", "2*a+1"),
        ("gf(12)", "ERROR"),
        ("F = gf(9); gf(9, a + 1) == F(a + 1)", "true"),
    ] {
        let got = one(code);
        if want == "ERROR" {
            assert!(got.starts_with("ERROR"), "{} gave {}", code, got);
        } else {
            assert_eq!(got, want, "{}", code);
        }
    }
}

#[test]
fn field_axioms() {
    for q in [
        2u32, 4, 8, 9, 16, 25, 27, 32, 49, 64, 81, 121, 125, 243, 256,
    ] {
        let f = ff::gf(&BigInt::from(q)).unwrap();
        let es = f.elements().unwrap();
        assert_eq!(es.len(), q as usize);
        let g = f.generator().unwrap();
        assert_eq!(g.order().unwrap(), BigInt::from(q - 1), "gf({})", q);
        let mut seen = std::collections::HashSet::new();
        for x in &es {
            // Fermat: x^q = x
            assert_eq!(&x.pow(&BigInt::from(q)).unwrap(), x);
            if x.is_zero() {
                assert!(x.inv().is_err());
                continue;
            }
            assert!(x.mul(&x.inv().unwrap()).pow(&BigInt::from(7)).unwrap() == x.div(x).unwrap());
            let k = ff::dlog(x, &g).unwrap();
            assert_eq!(&g.pow(&k).unwrap(), x);
            assert!(seen.insert(k));
        }
    }
}

#[test]
fn zmod_units() {
    for n in 2u32..60 {
        let r = ff::zmod(&BigInt::from(n)).unwrap();
        let units = r
            .elements()
            .unwrap()
            .into_iter()
            .filter(|x| x.is_unit())
            .count();
        assert_eq!(BigInt::from(units), *r.units(), "zmod({})", n);
        if let Ok(g) = r.generator() {
            assert_eq!(g.order().unwrap(), *r.units());
        }
    }
}

#[test]
fn groebner_bases() {
    for (code, want) in [
        ("groebner([x^2 + y^2 - 1, x - y], [x, y])", "[x-y, y^2-1/2]"),
        (
            "groebner([x^2 + y + z - 1, x + y^2 + z - 1, x + y + z^2 - 1], [x, y, z])",
            "[x+y+z^2-1, y^2-y-z^2+z, y*z^2+z^4/2-z^2/2, z^6-4*z^4+4*z^3-z^2]",
        ),
        (
            "groebner([x^3 - 2*x*y, x^2*y - 2*y^2 + x], [x, y], \"grlex\")",
            "[x^2, x*y, y^2-x/2]",
        ),
        ("groebner([x^2 + 1, x + 1], [x], 7)", "[1]"),
        ("groebner([x^2 + 1, x + 3], [x], 5)", "[x+3]"),
        ("polyrem(x^3*y, [x^2 + y^2 - 1, x - y], [x, y])", "1/4"),
        ("inideal(2*y^2 - 1, [x^2 + y^2 - 1, x - y], [x, y])", "true"),
        ("inideal(y, [x^2 + y^2 - 1, x - y], [x, y])", "false"),
    ] {
        assert_eq!(one(code), want, "{}", code);
    }
}

#[test]
fn number_fields_and_quotients() {
    for (code, want) in [
        (
            "K = numfield(x^2 - 2); K(x + 1)^2",
            "numfield(x^2-2, 2*x+3)",
        ),
        (
            "K = numfield(x^2 - 2); 1 / K(x + 1)",
            "numfield(x^2-2, x-1)",
        ),
        ("K = numfield(x^2 - 2); norm(K(x + 1))", "-1"),
        ("K = numfield(x^2 - 2); trace(K(x + 1))", "2"),
        ("K = numfield(x^2 - 2); minpoly(K(x + 1))", "x^2-2*x-1"),
        ("L = numfield(x^3 - 2); minpoly(L(x^2 + x))", "x^3-6*x-6"),
        ("L = numfield(x^3 - 2); norm(L(x^2 + x))", "6"),
        ("L = numfield(x^3 - 2); minpoly(L(3))", "x-3"),
        ("numfield(x^2 - 1)", "ERROR"),
        (
            "R = quotient(x^2 - 1, x); R(x + 1) * R(x - 1)",
            "quotient(x^2-1, x, 0, 0)",
        ),
        ("R = quotient(x^2 - 1, x); R(x + 1)^-1", "ERROR"),
        (
            "F = quotient(x^2 + 1, x, 3); F(x + 1)^8",
            "quotient(x^2+1, x, 3, 1)",
        ),
        ("isfield(quotient(x^2 + 1, x, 5))", "false"),
        ("order(quotient(x^3 + x + 1, x, 2))", "8"),
        (
            "quotient(x^2+1, x, 3, x+1) == quotient(x^2 + 1, x, 3)(x + 1)",
            "ERROR",
        ),
        (
            "F = quotient(x^2 + 1, x, 3); quotient(x^2+1, x, 3, x+1) == F(x+1)",
            "true",
        ),
        ("numfield(x^2-2, x+1) + 1", "numfield(x^2-2, x+2)"),
        ("trace([1, 2; 3, 4])", "5"),
    ] {
        let got = one(code);
        if want == "ERROR" {
            assert!(got.starts_with("ERROR"), "{} gave {}", code, got);
        } else {
            assert_eq!(got, want, "{}", code);
        }
    }
}

#[test]
fn quotient_identities() {
    use galileo_rs::qring as qr;
    use num_rational::BigRational;
    let mut seed = 12345u64;
    let mut rnd = |m: i64| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) % (2 * m as u64 + 1)) as i64 - m
    };
    for case in 0..60 {
        let p = [None, Some(5), Some(7)][case % 3].map(BigInt::from);
        let n = 2 + case % 3;
        let mut f = format!("x^{}", n);
        for k in 0..n {
            f.push_str(&format!(" + ({})*x^{}", rnd(4), k));
        }
        let fe = galileo_rs::eval::Session::new().run(&f).pop().unwrap().ok();
        let Some(galileo_rs::Output::Value(fe)) = fe else { panic!() };
        let r = qr::make(&fe, "x", p, false).unwrap();
        for _ in 0..5 {
            let v: Vec<BigRational> = (0..n).map(|_| BigRational::from_integer(rnd(5).into())).collect();
            let a = r.elem(v);
            if let Ok(b) = a.inv() {
                assert_eq!(a.mul(&b), r.elem(vec![BigRational::from_integer(1.into())]));
            }
            // minpoly(a)(a) = 0
            let m = a.minpoly();
            let mut acc = r.elem(vec![]);
            for c in m.iter().rev() {
                acc = acc.mul(&a).add(&r.elem(vec![c.clone()]));
            }
            assert!(acc.v.is_empty(), "minpoly fails in {}", r.name());
        }
    }
}
