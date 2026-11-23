//! Numbers: exact rationals, with floating point as the inexact fallback.
//!
//! Integer literals stay exact (`1/3 + 2/6` is `2/3`). Decimal literals are
//! floats, and floats are contagious: anything combined with a float is a
//! float. Exact numbers have no size limit: small ones use i128, larger
//! ones switch to big integers automatically and never become floats.

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::rc::Rc;

/// Results with more bits than this stay symbolic (2^(10^9) is not expanded).
pub const MAX_BITS: u64 = 1 << 22;

#[derive(Clone, Debug)]
pub enum Number {
    /// numerator, denominator; denominator > 0 and gcd == 1
    Rat(i128, i128),
    /// an exact rational that does not fit in Rat (never one that does)
    Big(Rc<BigRational>),
    Float(f64),
}

use Number::*;

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// a normalized big rational as a Number, small when it fits
fn norm(r: BigRational) -> Number {
    match (r.numer().to_i128(), r.denom().to_i128()) {
        (Some(n), Some(d)) => Rat(n, d),
        _ => Big(Rc::new(r)),
    }
}

impl Number {
    pub fn int(n: i128) -> Number {
        Rat(n, 1)
    }
    pub fn from_bigint(n: BigInt) -> Number {
        norm(BigRational::from_integer(n))
    }
    pub fn from_big(r: BigRational) -> Number {
        norm(r)
    }
    pub fn zero() -> Number {
        Rat(0, 1)
    }
    pub fn one() -> Number {
        Rat(1, 1)
    }
    pub fn rat(n: i128, d: i128) -> Option<Number> {
        if d == 0 {
            return None;
        }
        let g = gcd(n, d);
        let (n, d) = (n / g, d / g);
        if d < 0 {
            return match (n.checked_neg(), d.checked_neg()) {
                (Some(n), Some(d)) => Some(Rat(n, d)),
                _ => Some(norm(BigRational::new(n.into(), d.into()))),
            };
        }
        Some(Rat(n, d))
    }
    /// Parse a numeric literal: integers are exact (of any length), decimals are floats.
    pub fn parse(s: &str) -> Option<Number> {
        let t = s.trim_end_matches(['f', 'F', 'd', 'D']);
        if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit() || c == '-') {
            if let Ok(n) = t.parse::<i128>() {
                return Some(Rat(n, 1));
            }
            if let Ok(n) = t.parse::<BigInt>() {
                return Some(Number::from_bigint(n));
            }
        }
        t.parse::<f64>().ok().map(Float)
    }

    /// the exact value as a big rational (None for floats)
    pub fn big(&self) -> Option<BigRational> {
        match self {
            Rat(n, d) => Some(BigRational::new_raw(BigInt::from(*n), BigInt::from(*d))),
            Big(r) => Some((**r).clone()),
            Float(_) => None,
        }
    }
    /// the exact integer value, of any size
    pub fn as_bigint(&self) -> Option<BigInt> {
        match self {
            Rat(n, 1) => Some(BigInt::from(*n)),
            Big(r) if r.is_integer() => Some(r.numer().clone()),
            _ => None,
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            Rat(n, d) => *n as f64 / *d as f64,
            Big(r) => r.to_f64().unwrap_or_else(|| {
                // too large for f64
                let s = if r.is_negative() { -1.0 } else { 1.0 };
                if r.numer().bits() > r.denom().bits() {
                    s * f64::INFINITY
                } else {
                    0.0
                }
            }),
            Float(f) => *f,
        }
    }
    pub fn is_exact(&self) -> bool {
        !matches!(self, Float(_))
    }
    pub fn is_zero(&self) -> bool {
        match self {
            Rat(n, _) => *n == 0,
            Big(_) => false,
            Float(f) => *f == 0.0,
        }
    }
    pub fn is_one(&self) -> bool {
        match self {
            Rat(n, d) => *n == 1 && *d == 1,
            Big(_) => false,
            Float(f) => *f == 1.0,
        }
    }
    pub fn is_minus_one(&self) -> bool {
        match self {
            Rat(n, d) => *n == -1 && *d == 1,
            Big(_) => false,
            Float(f) => *f == -1.0,
        }
    }
    pub fn is_negative(&self) -> bool {
        match self {
            Rat(n, _) => *n < 0,
            Big(r) => r.is_negative(),
            Float(f) => *f < 0.0,
        }
    }
    pub fn is_integer(&self) -> bool {
        match self {
            Rat(_, d) => *d == 1,
            Big(r) => r.is_integer(),
            Float(f) => f.fract() == 0.0 && f.is_finite(),
        }
    }
    /// Exact integer value, if this is an exact integer that fits in i128
    pub fn as_int(&self) -> Option<i128> {
        match self {
            Rat(n, 1) => Some(*n),
            _ => None,
        }
    }
    pub fn numer(&self) -> Option<i128> {
        if let Rat(n, _) = self { Some(*n) } else { None }
    }
    pub fn denom(&self) -> Option<i128> {
        if let Rat(_, d) = self { Some(*d) } else { None }
    }

    pub fn neg(&self) -> Number {
        match self {
            Rat(n, d) => n
                .checked_neg()
                .map(|n| Rat(n, *d))
                .unwrap_or_else(|| norm(-self.big().unwrap())),
            Big(r) => norm(-(**r).clone()),
            Float(f) => Float(-f),
        }
    }
    pub fn abs(&self) -> Number {
        if self.is_negative() {
            self.neg()
        } else {
            self.clone()
        }
    }

    pub fn add(&self, o: impl Borrow<Number>) -> Number {
        let o = o.borrow();
        if let (Rat(a, b), Rat(c, d)) = (self, o) {
            let (a, b, c, d) = (*a, *b, *c, *d);
            let r = (|| {
                let g = gcd(b, d);
                let l = (b / g).checked_mul(d)?;
                let n = a.checked_mul(l / b)?.checked_add(c.checked_mul(l / d)?)?;
                Number::rat(n, l)
            })();
            if let Some(r) = r {
                return r;
            }
        }
        match (self.big(), o.big()) {
            (Some(x), Some(y)) => norm(x + y),
            _ => Float(self.to_f64() + o.to_f64()),
        }
    }
    pub fn sub(&self, o: impl Borrow<Number>) -> Number {
        self.add(o.borrow().neg())
    }
    pub fn mul(&self, o: impl Borrow<Number>) -> Number {
        let o = o.borrow();
        if let (Rat(a, b), Rat(c, d)) = (self, o) {
            let (a, b, c, d) = (*a, *b, *c, *d);
            let g1 = gcd(a, d).max(1);
            let g2 = gcd(c, b).max(1);
            let r =
                (|| Number::rat((a / g1).checked_mul(c / g2)?, (b / g2).checked_mul(d / g1)?))();
            if let Some(r) = r {
                return r;
            }
        }
        match (self.big(), o.big()) {
            (Some(x), Some(y)) => norm(x * y),
            _ => Float(self.to_f64() * o.to_f64()),
        }
    }
    pub fn recip(&self) -> Option<Number> {
        match self {
            Rat(0, _) => None,
            Rat(n, d) => Number::rat(*d, *n),
            Big(r) => Some(norm(r.recip())),
            Float(f) => Some(Float(1.0 / f)),
        }
    }
    pub fn div(&self, o: impl Borrow<Number>) -> Option<Number> {
        Some(self.mul(o.borrow().recip()?))
    }

    /// self^e when the result is a plain number. Returns None when an exact
    /// result does not exist (e.g. 2^(1/2)) or would be gigantic, so the
    /// caller keeps it symbolic.
    pub fn pow(&self, e: impl Borrow<Number>) -> Option<Number> {
        let e = e.borrow();
        if !self.is_exact() || !e.is_exact() {
            let r = self.to_f64().powf(e.to_f64());
            return if r.is_nan() { None } else { Some(Float(r)) };
        }
        let eb = e.big()?;
        if eb.is_integer() {
            // 0, 1 and -1 to any whole power
            if self.is_zero() {
                return if eb.is_negative() {
                    None
                } else if eb.is_zero() {
                    Some(Number::one())
                } else {
                    Some(Number::zero())
                };
            }
            if self.is_one() {
                return Some(Number::one());
            }
            if self.is_minus_one() {
                let odd = (eb.numer() % BigInt::from(2)).is_zero();
                return Some(if odd { Number::one() } else { Number::int(-1) });
            }
            if eb.is_negative() {
                return self.recip()?.pow(e.neg());
            }
            let p = eb.numer().to_u32()?;
            if let Rat(n, d) = self {
                if let (Some(a), Some(b)) = (n.checked_pow(p), d.checked_pow(p)) {
                    return Number::rat(a, b);
                }
            }
            let b = self.big()?;
            let bits = b.numer().bits().max(b.denom().bits());
            if bits.saturating_mul(p as u64) > MAX_BITS {
                return None;
            }
            return Some(norm(BigRational::new_raw(
                b.numer().pow(p),
                b.denom().pow(p),
            )));
        }
        // rational exponent p/q: exact if numerator and denominator are perfect q-th powers
        if self.is_negative() {
            return None;
        }
        let q = eb.denom().to_u32()?;
        let b = self.big()?;
        let rn = b.numer().nth_root(q);
        let rd = b.denom().nth_root(q);
        if rn.pow(q) != *b.numer() || rd.pow(q) != *b.denom() {
            return None;
        }
        norm(BigRational::new_raw(rn, rd)).pow(Number::from_bigint(eb.numer().clone()))
    }

    pub fn cmp_value(&self, o: impl Borrow<Number>) -> Ordering {
        let o = o.borrow();
        if let (Rat(a, b), Rat(c, d)) = (self, o) {
            if let (Some(x), Some(y)) = (a.checked_mul(*d), c.checked_mul(*b)) {
                return x.cmp(&y);
            }
        }
        match (self.big(), o.big()) {
            (Some(x), Some(y)) => x.cmp(&y),
            _ => self.to_f64().total_cmp(&o.to_f64()),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, o: &Number) -> bool {
        match (self, o) {
            (Rat(a, b), Rat(c, d)) => a == c && b == d,
            (Big(a), Big(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            _ => false, // 1 and 1.0 are different expressions
        }
    }
}

pub fn fmt_float(f: f64) -> String {
    if f.is_nan() {
        return "NaN".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "Inf".into() } else { "-Inf".into() };
    }
    let a = f.abs();
    if a != 0.0 && !(1e-5..1e16).contains(&a) {
        let s = format!("{:e}", f);
        return s;
    }
    let s = format!("{}", f);
    if s.contains('.') || s.contains('e') {
        s
    } else {
        s + ".0"
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rat(n, 1) => write!(f, "{}", n),
            Rat(n, d) => write!(f, "{}/{}", n, d),
            Big(r) if r.is_integer() => write!(f, "{}", r.numer()),
            Big(r) => write!(f, "{}/{}", r.numer(), r.denom()),
            Float(x) => write!(f, "{}", fmt_float(*x)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rational_arithmetic() {
        let a = Number::rat(1, 3).unwrap();
        let b = Number::rat(2, 6).unwrap();
        assert_eq!(a.add(b), Number::rat(2, 3).unwrap());
        assert_eq!(
            Number::int(-12).div(Number::int(3)).unwrap(),
            Number::int(-4)
        );
        assert_eq!(
            Number::rat(4, 9).unwrap().pow(Number::rat(1, 2).unwrap()),
            Number::rat(2, 3)
        );
        assert_eq!(Number::int(2).pow(Number::rat(1, 2).unwrap()), None);
        assert_eq!(Number::int(2).pow(Number::int(-2)), Number::rat(1, 4));
        assert_eq!(format!("{}", Number::Float(3.0)), "3.0");
        assert_eq!(format!("{}", Number::rat(-5, 6).unwrap()), "-5/6");
    }
}
