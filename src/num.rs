//! Numbers: exact rationals, with floating point as the inexact fallback.
//!
//! Integer literals stay exact (`1/3 + 2/6` is `2/3`). Decimal literals are
//! floats, and floats are contagious: anything combined with a float is a
//! float. Rational overflow falls back to floats rather than failing.

use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub enum Number {
    /// numerator, denominator; denominator > 0 and gcd == 1
    Rat(i128, i128),
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

impl Number {
    pub fn int(n: i128) -> Number {
        Rat(n, 1)
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
        let (mut n, mut d) = (n / g, d / g);
        if d < 0 {
            n = -n;
            d = -d;
        }
        Some(Rat(n, d))
    }
    /// Parse a numeric literal: integers are exact, decimals are floats.
    pub fn parse(s: &str) -> Option<Number> {
        let t = s.trim_end_matches(['f', 'F', 'd', 'D']);
        if t.chars().all(|c| c.is_ascii_digit() || c == '-') {
            if let Ok(n) = t.parse::<i128>() {
                return Some(Rat(n, 1));
            }
        }
        t.parse::<f64>().ok().map(Float)
    }

    pub fn to_f64(self) -> f64 {
        match self {
            Rat(n, d) => n as f64 / d as f64,
            Float(f) => f,
        }
    }
    pub fn is_exact(self) -> bool {
        matches!(self, Rat(..))
    }
    pub fn is_zero(self) -> bool {
        self.to_f64() == 0.0
    }
    pub fn is_one(self) -> bool {
        match self {
            Rat(n, d) => n == 1 && d == 1,
            Float(f) => f == 1.0,
        }
    }
    pub fn is_minus_one(self) -> bool {
        match self {
            Rat(n, d) => n == -1 && d == 1,
            Float(f) => f == -1.0,
        }
    }
    pub fn is_negative(self) -> bool {
        self.to_f64() < 0.0
    }
    pub fn is_integer(self) -> bool {
        match self {
            Rat(_, d) => d == 1,
            Float(f) => f.fract() == 0.0 && f.is_finite(),
        }
    }
    /// Exact integer value, if this is an exact integer
    pub fn as_int(self) -> Option<i128> {
        match self {
            Rat(n, 1) => Some(n),
            _ => None,
        }
    }
    pub fn numer(self) -> Option<i128> {
        if let Rat(n, _) = self { Some(n) } else { None }
    }
    pub fn denom(self) -> Option<i128> {
        if let Rat(_, d) = self { Some(d) } else { None }
    }

    pub fn neg(self) -> Number {
        match self {
            Rat(n, d) => n.checked_neg().map(|n| Rat(n, d)).unwrap_or(Float(-(n as f64) / d as f64)),
            Float(f) => Float(-f),
        }
    }
    pub fn abs(self) -> Number {
        if self.is_negative() { self.neg() } else { self }
    }

    pub fn add(self, o: Number) -> Number {
        if let (Rat(a, b), Rat(c, d)) = (self, o) {
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
        Float(self.to_f64() + o.to_f64())
    }
    pub fn sub(self, o: Number) -> Number {
        self.add(o.neg())
    }
    pub fn mul(self, o: Number) -> Number {
        if let (Rat(a, b), Rat(c, d)) = (self, o) {
            let g1 = gcd(a, d).max(1);
            let g2 = gcd(c, b).max(1);
            let r = (|| Number::rat((a / g1).checked_mul(c / g2)?, (b / g2).checked_mul(d / g1)?))();
            if let Some(r) = r {
                return r;
            }
        }
        Float(self.to_f64() * o.to_f64())
    }
    pub fn recip(self) -> Option<Number> {
        match self {
            Rat(0, _) => None,
            Rat(n, d) => Number::rat(d, n),
            Float(f) => Some(Float(1.0 / f)),
        }
    }
    pub fn div(self, o: Number) -> Option<Number> {
        Some(self.mul(o.recip()?))
    }

    /// self^e when the result is a plain number. Returns None when an exact
    /// result does not exist (e.g. 2^(1/2)), so the caller keeps it symbolic.
    pub fn pow(self, e: Number) -> Option<Number> {
        match (self, e) {
            (Rat(n, d), Rat(p, 1)) => {
                if p >= 0 {
                    let p = u32::try_from(p).ok()?;
                    match (n.checked_pow(p), d.checked_pow(p)) {
                        (Some(a), Some(b)) => Number::rat(a, b),
                        _ => Some(Float((n as f64 / d as f64).powi(p as i32))),
                    }
                } else {
                    Number::rat(n, d)?.recip()?.pow(Rat(-p, 1))
                }
            }
            (Rat(n, d), Rat(p, q)) => {
                // exact root if numerator and denominator are perfect q-th powers
                if n < 0 {
                    return None;
                }
                let q32 = u32::try_from(q).ok()?;
                let rn = iroot(n, q32)?;
                let rd = iroot(d, q32)?;
                Number::rat(rn, rd)?.pow(Rat(p, 1))
            }
            _ => {
                let r = self.to_f64().powf(e.to_f64());
                if r.is_nan() { None } else { Some(Float(r)) }
            }
        }
    }

    pub fn cmp_value(self, o: Number) -> Ordering {
        match (self, o) {
            (Rat(a, b), Rat(c, d)) => match (a.checked_mul(d), c.checked_mul(b)) {
                (Some(x), Some(y)) => x.cmp(&y),
                _ => self.to_f64().total_cmp(&o.to_f64()),
            },
            _ => self.to_f64().total_cmp(&o.to_f64()),
        }
    }
}

/// exact integer q-th root
fn iroot(n: i128, q: u32) -> Option<i128> {
    if n < 0 {
        return None;
    }
    if n < 2 {
        return Some(n);
    }
    let guess = (n as f64).powf(1.0 / q as f64).round() as i128;
    for c in [guess - 1, guess, guess + 1] {
        if c >= 0 && c.checked_pow(q) == Some(n) {
            return Some(c);
        }
    }
    None
}

impl PartialEq for Number {
    fn eq(&self, o: &Number) -> bool {
        match (self, o) {
            (Rat(a, b), Rat(c, d)) => a == c && b == d,
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
    if s.contains('.') || s.contains('e') { s } else { s + ".0" }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rat(n, 1) => write!(f, "{}", n),
            Rat(n, d) => write!(f, "{}/{}", n, d),
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
        assert_eq!(Number::int(-12).div(Number::int(3)).unwrap(), Number::int(-4));
        assert_eq!(Number::rat(4, 9).unwrap().pow(Number::rat(1, 2).unwrap()), Number::rat(2, 3));
        assert_eq!(Number::int(2).pow(Number::rat(1, 2).unwrap()), None);
        assert_eq!(Number::int(2).pow(Number::int(-2)), Number::rat(1, 4));
        assert_eq!(format!("{}", Number::Float(3.0)), "3.0");
        assert_eq!(format!("{}", Number::rat(-5, 6).unwrap()), "-5/6");
    }
}
