//! Printing expressions in Galileo's input syntax (output can be pasted back).

use crate::expr::{Const, Expr};
use crate::num::Number;
use Expr::*;

const P_ADD: u8 = 1;
const P_MUL: u8 = 2;
const P_POW: u8 = 3;
const P_ATOM: u8 = 4;

pub fn show(e: &Expr) -> String {
    match e {
        Matrix(rows) => rows
            .iter()
            .map(|r| r.iter().map(show).collect::<Vec<_>>().join("\t"))
            .collect::<Vec<_>>()
            .join("\n"),
        Str(s) => s.clone(),
        Obj(o) => o.text(),
        _ => fmt(e).0,
    }
}

fn wrap(s: (String, u8), min: u8) -> String {
    if s.1 < min { format!("({})", s.0) } else { s.0 }
}

fn num_prec(n: Number) -> u8 {
    if n.is_negative() {
        P_ADD
    } else if !n.is_integer() && n.is_exact() {
        P_MUL
    } else {
        P_ATOM
    }
}

/// (text, precedence of the outermost operator)
fn fmt(e: &Expr) -> (String, u8) {
    match e {
        Num(n) => (n.to_string(), num_prec(*n)),
        Sym(s) => (s.clone(), P_ATOM),
        Const(c) => (
            match c {
                Const::Pi => "pi",
                Const::E => "e",
                Const::I => "i",
            }
            .into(),
            P_ATOM,
        ),
        Bool(b) => (b.to_string(), P_ATOM),
        Str(s) => (format!("\"{}\"", s), P_ATOM),
        Obj(o) => (format!("<{}>", o.kind()), P_ATOM),
        Matrix(rows) => (
            format!(
                "[{}]",
                rows.iter()
                    .map(|r| r.iter().map(|x| fmt(x).0).collect::<Vec<_>>().join(", "))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            P_ATOM,
        ),
        Func(f, a) => (format!("{}({})", f.name(), fmt(a).0), P_ATOM),
        Apply(f, v) if f == "==" && v.len() == 2 => (format!("{} == {}", fmt(&v[0]).0, fmt(&v[1]).0), 0),
        Apply(f, v) => (format!("{}({})", f, v.iter().map(|x| fmt(x).0).collect::<Vec<_>>().join(", ")), P_ATOM),
        Add(terms) => {
            // lead with a positive term when there is one: b-a rather than -a+b
            let mut terms = terms.clone();
            // complex numbers read real part first: 3-2*i, not -2*i+3
            let gaussian = crate::build::gauss(e).is_some();
            if gaussian {
                terms.rotate_right(1);
            } else {
                if let Some(k) = terms.iter().position(|t| !t.is_negative_term()) {
                    let t = terms.remove(k);
                    terms.insert(0, t);
                }
                // a + b*i: the imaginary part last
                let (re, im): (Vec<Expr>, Vec<Expr>) = terms.into_iter().partition(|t| !crate::extra_cmd::has_i(t));
                if !re.is_empty() && !im.is_empty() && im[0].is_negative_term() && !re.iter().any(|t| !t.is_negative_term()) {
                    // keep a positive term first when the real part is all negative
                    terms = im.into_iter().chain(re).collect();
                } else {
                    terms = re.into_iter().chain(im).collect();
                }
            }
            let mut s = String::new();
            for (i, t) in terms.iter().enumerate() {
                if t.is_negative_term() {
                    s.push('-');
                    s.push_str(&wrap(fmt(&crate::build::neg(t.clone())), P_MUL));
                } else {
                    if i > 0 {
                        s.push('+');
                    }
                    s.push_str(&wrap(fmt(t), P_MUL));
                }
            }
            (s, P_ADD)
        }
        Mul(_) => fmt_mul(e),
        Pow(b, x) => {
            if let Num(n) = **x {
                if n.is_negative() {
                    return fmt_mul(e);
                }
                if n == Number::rat(1, 2).unwrap() {
                    return (format!("sqrt({})", fmt(b).0), P_ATOM);
                }
            }
            if **b == Const(Const::E) {
                return (format!("exp({})", fmt(x).0), P_ATOM);
            }
            (format!("{}^{}", wrap(fmt(b), P_ATOM), wrap(fmt(x), P_ATOM)), P_POW)
        }
    }
}

/// Products, written as `sign coefficient*numerator/denominator`
fn fmt_mul(e: &Expr) -> (String, u8) {
    let factors: Vec<Expr> = match e {
        Mul(f) => f.clone(),
        other => vec![other.clone()],
    };
    let mut coeff = Number::one();
    let mut numer: Vec<Expr> = vec![];
    let mut denom: Vec<Expr> = vec![];
    for f in factors {
        match &f {
            Num(n) => coeff = coeff.mul(*n),
            Pow(b, x) if x.num().is_some_and(|n| n.is_negative()) => {
                denom.push(crate::build::pow((**b).clone(), Num(x.num().unwrap().neg())))
            }
            _ => numer.push(f),
        }
    }
    let negative = coeff.is_negative();
    let coeff = coeff.abs();
    let mut ns: Vec<String> = vec![];
    let mut ds: Vec<String> = vec![];
    match coeff {
        Number::Rat(n, d) => {
            if n != 1 || numer.is_empty() {
                ns.push(n.to_string());
            }
            if d != 1 {
                ds.push(d.to_string());
            }
        }
        Number::Float(_) => {
            if !coeff.is_one() || numer.is_empty() {
                ns.push(coeff.to_string())
            }
        }
    }
    ns.extend(numer.iter().map(|f| wrap(fmt(f), P_POW)));
    ds.extend(denom.iter().map(|f| wrap(fmt(f), P_POW)));
    let mut s = ns.join("*");
    if !ds.is_empty() {
        let d = if ds.len() > 1 { format!("({})", ds.join("*")) } else { ds.pop().unwrap() };
        s = format!("{}/{}", s, d);
    }
    if negative {
        s = format!("-{}", s);
        (s, P_ADD)
    } else {
        (s, P_MUL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::*;
    fn x() -> Expr {
        Expr::sym("x")
    }
    #[test]
    fn prints_nicely() {
        let e = sub(mul(vec![Expr::int(3), pow(x(), Expr::int(2))]), mul(vec![Expr::int(2), x()]));
        assert_eq!(show(&e), "3*x^2-2*x");
        assert_eq!(show(&div(x(), Expr::int(3))), "x/3");
        assert_eq!(show(&div(Expr::one(), mul(vec![x(), Expr::sym("y")]))), "1/(x*y)");
        assert_eq!(show(&neg(add(vec![x(), Expr::one()]))), "-(x+1)");
        assert_eq!(show(&pow(add(vec![x(), Expr::one()]), Expr::int(2))), "(x+1)^2");
        assert_eq!(show(&sqrt(x())), "sqrt(x)");
        assert_eq!(show(&add(vec![x(), Expr::int(-9)])), "x-9");
    }
}
