//! Smart constructors that keep expressions canonical.

use crate::expr::{cmp, term_cmp, Const, Expr, Func};
use crate::num::Number;
use Expr::*;

pub fn add(terms: Vec<Expr>) -> Expr {
    let mut constant = Number::zero();
    let mut groups: Vec<(Expr, Number)> = vec![];
    let mut stack: Vec<Expr> = terms;
    stack.reverse();
    while let Some(t) = stack.pop() {
        match t {
            Add(inner) => stack.extend(inner.into_iter().rev()),
            Num(n) => constant = constant.add(n),
            other => {
                let (c, rest) = other.coeff_and_rest();
                match groups.iter_mut().find(|(r, _)| *r == rest) {
                    Some(g) => g.1 = g.1.add(c),
                    None => groups.push((rest, c)),
                }
            }
        }
    }
    let mut out: Vec<Expr> = groups
        .into_iter()
        .filter(|(_, c)| !c.is_zero())
        .map(|(r, c)| scale(c, r))
        .collect();
    out.sort_by(term_cmp);
    if !constant.is_zero() {
        out.push(Num(constant));
    }
    match out.len() {
        0 => Num(constant), // keeps 0.0 a float
        1 => out.pop().unwrap(),
        _ => Add(out),
    }
}

/// c * e without re-simplifying e (e has no numeric coefficient)
fn scale(c: Number, e: Expr) -> Expr {
    if c.is_one() {
        return e;
    }
    match e {
        Mul(mut f) => {
            f.insert(0, Num(c));
            Mul(f)
        }
        Num(n) => Num(n.mul(c)),
        other => Mul(vec![Num(c), other]),
    }
}

pub fn mul(factors: Vec<Expr>) -> Expr {
    let mut coeff = Number::one();
    let mut groups: Vec<(Expr, Expr)> = vec![];
    let mut stack = factors;
    stack.reverse();
    while let Some(f) = stack.pop() {
        match f {
            Mul(inner) => stack.extend(inner.into_iter().rev()),
            Num(n) => coeff = coeff.mul(n),
            other => {
                let (b, e) = other.base_exp();
                match groups.iter_mut().find(|(gb, _)| *gb == b) {
                    Some(g) => g.1 = add(vec![g.1.clone(), e]),
                    None => groups.push((b, e)),
                }
            }
        }
    }
    if coeff.is_zero() && coeff.is_exact() {
        return Expr::zero();
    }
    let mut out = vec![];
    let mut again = false;
    for (b, e) in groups {
        let p = pow(b, e);
        match p {
            Num(n) => coeff = coeff.mul(n),
            Mul(_) => {
                again = true;
                out.push(p)
            }
            p if p.is_one() => {}
            p => out.push(p),
        }
    }
    if again {
        out.push(Num(coeff));
        return mul(out);
    }
    if coeff.is_zero() && coeff.is_exact() {
        return Expr::zero();
    }
    out.sort_by(cmp);
    if out.is_empty() {
        return Num(coeff);
    }
    if !coeff.is_one() {
        out.insert(0, Num(coeff));
    }
    if out.len() == 1 { out.pop().unwrap() } else { Mul(out) }
}

pub fn pow(b: Expr, e: Expr) -> Expr {
    // an integral float exponent means the same as the exact integer: x^2.0 = x^2
    let e = match e {
        Num(Number::Float(f)) if f.fract() == 0.0 && f.abs() < 9.0e15 => Expr::int(f as i128),
        other => other,
    };
    if let Num(en) = e {
        if en.is_zero() {
            return Expr::one();
        }
        if en.is_one() {
            return b;
        }
        match &b {
            Num(bn) => {
                if let Some(r) = bn.pow(en) {
                    return Num(r);
                }
                // pull perfect powers out of roots: 8^(1/2) = 2*2^(1/2)
                if let (Number::Rat(n, d), Number::Rat(p, q)) = (*bn, en) {
                    if n > 0 && q > 1 {
                        let (kn, mn) = split_root(n, q);
                        let (kd, md) = split_root(d, q);
                        if kn != 1 || kd != 1 {
                            let k = Num(Number::rat(kn, kd).unwrap());
                            let m = Num(Number::rat(mn, md).unwrap());
                            return mul(vec![pow(k, Expr::int(p)), pow(m, e.clone())]);
                        }
                    }
                }
                // (-n)^(p/2) = i^p * n^(p/2)
                if bn.is_negative() && en.is_exact() && en.denom() == Some(2) {
                    let p = en.numer().unwrap();
                    return mul(vec![
                        pow(Const(Const::I), Expr::int(p)),
                        pow(Num(bn.neg()), e.clone()),
                    ]);
                }
            }
            Const(Const::I) if en.is_integer() && en.is_exact() => {
                return match en.as_int().unwrap().rem_euclid(4) {
                    0 => Expr::one(),
                    1 => Const(Const::I),
                    2 => Expr::int(-1),
                    _ => mul(vec![Expr::int(-1), Const(Const::I)]),
                };
            }
            Pow(b2, e2) if en.is_integer() => {
                return pow((**b2).clone(), mul(vec![(**e2).clone(), e]));
            }
            Mul(fs) if en.is_integer() => {
                return mul(fs.iter().map(|f| pow(f.clone(), e.clone())).collect());
            }
            _ => {}
        }
    } else if b.is_one() {
        return b;
    }
    // e^log(x) = x
    if b == Const(Const::E) {
        if let Func(Func::Log, x) = &e {
            return (**x).clone();
        }
    }
    Pow(Box::new(b), Box::new(e))
}

/// n = k^q * m with k as large as trial division (up to 10^5) finds
fn split_root(n: i128, q: i128) -> (i128, i128) {
    let (mut k, mut m) = (1i128, n);
    let mut p = 2i128;
    while p <= 100_000 && p * p <= m {
        let pq = p.checked_pow(q as u32);
        match pq {
            Some(pq) if m % pq == 0 => {
                m /= pq;
                k *= p;
            }
            _ => p += 1,
        }
    }
    (k, m)
}

pub fn neg(e: Expr) -> Expr {
    mul(vec![Expr::int(-1), e])
}
pub fn sub(a: Expr, b: Expr) -> Expr {
    add(vec![a, neg(b)])
}
pub fn div(a: Expr, b: Expr) -> Expr {
    mul(vec![a, pow(b, Expr::int(-1))])
}
pub fn add2(a: Expr, b: Expr) -> Expr {
    add(vec![a, b])
}
pub fn mul2(a: Expr, b: Expr) -> Expr {
    mul(vec![a, b])
}
pub fn exp(x: Expr) -> Expr {
    pow(Const(Const::E), x)
}
pub fn sqrt(x: Expr) -> Expr {
    pow(x, Num(Number::rat(1, 2).unwrap()))
}
pub fn rat(n: i128, d: i128) -> Expr {
    Num(Number::rat(n, d).unwrap())
}
pub fn pi() -> Expr {
    Const(Const::Pi)
}

/// `k*pi` with rational k -> Some(k)
fn pi_multiple(e: &Expr) -> Option<Number> {
    match e {
        Const(Const::Pi) => Some(Number::one()),
        Mul(f) if f.len() == 2 && f[1] == Const(Const::Pi) => f[0].num().filter(|n| n.is_exact()),
        _ => None,
    }
}

/// exact sin(k*pi) for k with denominator 1, 2, 3, 4 or 6
fn sin_pi(k: Number) -> Option<Expr> {
    let (n, d) = (k.numer()?, k.denom()?);
    // reduce to k in [0, 2) in units of 1/12
    let twelfths = (n * (12 / if 12 % d == 0 { d } else { return None })).rem_euclid(24);
    let s3 = || mul(vec![rat(1, 2), sqrt(Expr::int(3))]);
    let s2 = || mul(vec![rat(1, 2), sqrt(Expr::int(2))]);
    let v = match twelfths {
        0 | 12 => Expr::zero(),
        2 | 10 => rat(1, 2),
        3 | 9 => s2(),
        4 | 8 => s3(),
        6 => Expr::one(),
        14 | 22 => rat(-1, 2),
        15 | 21 => neg(s2()),
        16 | 20 => neg(s3()),
        18 => Expr::int(-1),
        _ => return None,
    };
    Some(v)
}

pub fn func(f: Func, x: Expr) -> Expr {
    use Func::*;
    if let Num(n) = x {
        if !n.is_exact() {
            let v = n.to_f64();
            return Expr::float(f.apply(v));
        }
    }
    let negated = x.is_negative_term();
    match f {
        Sin | Tan | Asin | Atan | Sinh | Tanh | Asinh | Atanh if negated => return neg(func(f, neg(x))),
        Cos | Abs | Cosh if negated => return func(f, neg(x)),
        _ => {}
    }
    match f {
        Sin => {
            if let Some(v) = pi_multiple(&x).and_then(sin_pi) {
                return v;
            }
            if x.is_zero() {
                return Expr::zero();
            }
        }
        Cos => {
            if let Some(k) = pi_multiple(&x) {
                if let Some(v) = sin_pi(k.add(Number::rat(1, 2).unwrap())) {
                    return v;
                }
            }
            if x.is_zero() {
                return Expr::one();
            }
        }
        Sinh | Tanh | Asinh | Atanh if x.is_zero() => return Expr::zero(),
        Cosh if x.is_zero() => return Expr::one(),
        Acosh if x.is_one() => return Expr::zero(),
        Tan => {
            if x.is_zero() {
                return Expr::zero();
            }
            if let Some(k) = pi_multiple(&x) {
                if k.is_integer() {
                    return Expr::zero();
                }
                if k.denom() == Some(4) {
                    return Expr::int(if k.numer().unwrap().rem_euclid(4) == 1 { 1 } else { -1 });
                }
            }
        }
        Asin => {
            if x.is_zero() {
                return Expr::zero();
            }
            if x.is_one() {
                return mul(vec![rat(1, 2), pi()]);
            }
            if x == rat(1, 2) {
                return mul(vec![rat(1, 6), pi()]);
            }
        }
        Acos => {
            if x.is_zero() {
                return mul(vec![rat(1, 2), pi()]);
            }
            if x.is_one() {
                return Expr::zero();
            }
            if x == Expr::int(-1) {
                return pi();
            }
            if x == rat(1, 2) {
                return mul(vec![rat(1, 3), pi()]);
            }
        }
        Atan => {
            if x.is_zero() {
                return Expr::zero();
            }
            if x.is_one() {
                return mul(vec![rat(1, 4), pi()]);
            }
        }
        Log => {
            if x.is_one() {
                return Expr::zero();
            }
            if x == Const(Const::E) {
                return Expr::one();
            }
            if let Pow(b, e) = &x {
                if **b == Const(Const::E) {
                    return (**e).clone();
                }
            }
        }
        Abs => {
            if let Num(n) = x {
                return Num(n.abs());
            }
            if let Const(Const::Pi | Const::E) = x {
                return x;
            }
        }
        _ => {}
    }
    Expr::Func(f, Box::new(x))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::print::show;
    fn x() -> Expr {
        Expr::sym("x")
    }
    fn y() -> Expr {
        Expr::sym("y")
    }
    #[test]
    fn collects_like_terms() {
        let e = add(vec![x(), mul(vec![Expr::int(2), x()]), Expr::int(3), y(), neg(y())]);
        assert_eq!(e, add(vec![mul(vec![Expr::int(3), x()]), Expr::int(3)]));
    }
    #[test]
    fn combines_powers() {
        assert_eq!(mul(vec![x(), x(), pow(x(), Expr::int(3))]), pow(x(), Expr::int(5)));
        assert_eq!(div(x(), x()), Expr::one());
    }
    #[test]
    fn imaginary_unit() {
        let i = Const(Const::I);
        assert_eq!(mul(vec![i.clone(), i.clone()]), Expr::int(-1));
        assert_eq!(sqrt(Expr::int(-1)), i);
        assert_eq!(show(&pow(i, Expr::int(3))), "-i");
    }
    #[test]
    fn trig_values() {
        assert_eq!(func(Func::Cos, mul(vec![Expr::int(3), pi()])), Expr::int(-1));
        assert_eq!(func(Func::Sin, mul(vec![rat(1, 6), pi()])), rat(1, 2));
        assert_eq!(func(Func::Cos, neg(x())), func(Func::Cos, x()));
    }
}
