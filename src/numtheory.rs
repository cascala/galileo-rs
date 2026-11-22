//! Integer arithmetic for number theory: primality, factoring, modular
//! arithmetic. Integers are i128 (up to about 1.7e38); the fast paths use
//! u128 products for moduli below 2^64.

/// a*b mod m without overflow
pub fn mulmod(a: u128, b: u128, m: u128) -> u128 {
    if m == 0 {
        return 0;
    }
    let (a, b) = (a % m, b % m);
    if m <= u64::MAX as u128 {
        return a * b % m;
    }
    // double-and-add for big moduli
    let (mut r, mut x, mut y) = (0u128, a, b);
    while y > 0 {
        if y & 1 == 1 {
            r = addmod(r, x, m);
        }
        x = addmod(x, x, m);
        y >>= 1;
    }
    r
}

fn addmod(a: u128, b: u128, m: u128) -> u128 {
    if a >= m - b { a - (m - b) } else { a + b }
}

/// a^e mod m
pub fn powmod(a: u128, mut e: u128, m: u128) -> u128 {
    if m == 1 {
        return 0;
    }
    let (mut r, mut b) = (1u128, a % m);
    while e > 0 {
        if e & 1 == 1 {
            r = mulmod(r, b, m);
        }
        b = mulmod(b, b, m);
        e >>= 1;
    }
    r
}

pub fn gcd(a: u128, b: u128) -> u128 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// (g, x, y) with a*x + b*y = g = gcd(a, b)
pub fn ext_gcd(a: i128, b: i128) -> (i128, i128, i128) {
    if b == 0 {
        if a < 0 { (-a, -1, 0) } else { (a, 1, 0) }
    } else {
        let (g, x, y) = ext_gcd(b, a.rem_euclid(b));
        (g, y, x - a.div_euclid(b) * y)
    }
}

/// a^-1 mod m, if gcd(a, m) = 1
pub fn modinv(a: i128, m: i128) -> Option<i128> {
    let (g, x, _) = ext_gcd(a.rem_euclid(m), m);
    if g == 1 { Some(x.rem_euclid(m)) } else { None }
}

const SMALL: [u128; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

/// Miller-Rabin; deterministic below 3.3e24 with these bases, and
/// a strong probable-prime test beyond
pub fn is_prime(n: u128) -> bool {
    if n < 2 {
        return false;
    }
    for p in SMALL {
        if n % p == 0 {
            return n == p;
        }
    }
    let mut d = n - 1;
    let mut s = 0;
    while d % 2 == 0 {
        d /= 2;
        s += 1;
    }
    let bases: &[u128] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71];
    'outer: for &a in bases {
        let mut x = powmod(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 1..s {
            x = mulmod(x, x, n);
            if x == n - 1 {
                continue 'outer;
            }
        }
        return false;
    }
    true
}

pub fn next_prime(n: u128) -> u128 {
    let mut k = n + 1;
    while !is_prime(k) {
        k += 1;
    }
    k
}

pub fn prev_prime(n: u128) -> Option<u128> {
    let mut k = n.checked_sub(1)?;
    while k >= 2 {
        if is_prime(k) {
            return Some(k);
        }
        k -= 1;
    }
    None
}

/// one nontrivial factor of an odd composite n (Pollard's rho, Brent's variant)
pub fn pollard_rho(n: u128) -> Option<u128> {
    if n % 2 == 0 {
        return Some(2);
    }
    for c in 1..200u128 {
        let f = |x: u128| addmod(mulmod(x, x, n), c % n, n);
        let (mut y, mut r, mut q, m) = (2u128, 1u64, 1u128, 128u64);
        let (mut g, mut x, mut ys) = (1u128, 0u128, 0u128);
        let mut steps: u64 = 0;
        while g == 1 {
            x = y;
            for _ in 0..r {
                y = f(y);
            }
            let mut k = 0;
            while k < r && g == 1 {
                ys = y;
                for _ in 0..m.min(r - k) {
                    y = f(y);
                    q = mulmod(q, x.abs_diff(y), n);
                }
                g = gcd(q, n);
                k += m;
            }
            r *= 2;
            steps += r;
            if steps > 50_000_000 {
                return None;
            }
        }
        if g == n {
            loop {
                ys = f(ys);
                g = gcd(x.abs_diff(ys), n);
                if g > 1 {
                    break;
                }
            }
        }
        if g != n {
            return Some(g);
        }
    }
    None
}

/// prime factorization as (prime, exponent), primes increasing
pub fn factorize(n: u128) -> Result<Vec<(u128, u32)>, String> {
    let mut primes = vec![];
    let mut n = n;
    for p in [2u128, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47] {
        while n % p == 0 {
            primes.push(p);
            n /= p;
        }
    }
    let mut p = 53u128;
    while p * p <= n && p < 10_000 {
        while n % p == 0 {
            primes.push(p);
            n /= p;
        }
        p += 2;
    }
    let mut todo = if n > 1 { vec![n] } else { vec![] };
    while let Some(m) = todo.pop() {
        if is_prime(m) {
            primes.push(m);
        } else {
            let d = pollard_rho(m).ok_or_else(|| format!("could not factor {} in reasonable time", m))?;
            todo.push(d);
            todo.push(m / d);
        }
    }
    primes.sort();
    let mut out: Vec<(u128, u32)> = vec![];
    for p in primes {
        match out.last_mut() {
            Some((q, e)) if *q == p => *e += 1,
            _ => out.push((p, 1)),
        }
    }
    Ok(out)
}

pub fn divisors(n: u128) -> Result<Vec<u128>, String> {
    let mut ds = vec![1u128];
    for (p, e) in factorize(n)? {
        let mut next = vec![];
        for d in &ds {
            let mut pk = 1u128;
            for _ in 0..=e {
                next.push(d * pk);
                pk *= p;
            }
        }
        ds = next;
    }
    ds.sort();
    Ok(ds)
}

pub fn totient(n: u128) -> Result<u128, String> {
    let mut t = n;
    for (p, _) in factorize(n)? {
        t = t / p * (p - 1);
    }
    Ok(t)
}

/// smallest k > 0 with a^k = 1 mod n
pub fn mult_order(a: u128, n: u128) -> Result<u128, String> {
    if gcd(a, n) != 1 {
        return Err(format!("{} has no order mod {}: they share a factor", a, n));
    }
    let phi = totient(n)?;
    let mut k = phi;
    for (p, _) in factorize(phi)? {
        while k % p == 0 && powmod(a, k / p, n) == 1 {
            k /= p;
        }
    }
    Ok(k)
}

/// smallest primitive root mod n, if there is one
pub fn primitive_root(n: u128) -> Result<Option<u128>, String> {
    if n == 2 {
        return Ok(Some(1));
    }
    let phi = totient(n)?;
    let ps: Vec<u128> = factorize(phi)?.into_iter().map(|(p, _)| p).collect();
    for g in 2..n.min(100_000) {
        if gcd(g, n) == 1 && ps.iter().all(|p| powmod(g, phi / p, n) != 1) {
            return Ok(Some(g));
        }
    }
    Ok(None)
}

/// primes up to n (sieve of Eratosthenes)
pub fn primes_upto(n: usize) -> Vec<usize> {
    if n < 2 {
        return vec![];
    }
    let mut sieve = vec![true; n + 1];
    sieve[0] = false;
    sieve[1] = false;
    let mut i = 2;
    while i * i <= n {
        if sieve[i] {
            let mut j = i * i;
            while j <= n {
                sieve[j] = false;
                j += i;
            }
        }
        i += 1;
    }
    (0..=n).filter(|&k| sieve[k]).collect()
}

/// integer square root
pub fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = (n as f64).sqrt() as u128;
    while x * x > n {
        x -= 1;
    }
    while (x + 1).checked_mul(x + 1).is_some_and(|y| y <= n) {
        x += 1;
    }
    x
}

/// x with x = r_i mod m_i for all i (moduli pairwise coprime)
pub fn crt(rs: &[i128], ms: &[i128]) -> Option<(i128, i128)> {
    let (mut x, mut m) = (0i128, 1i128);
    for (&r, &mi) in rs.iter().zip(ms) {
        let inv = modinv(m.rem_euclid(mi), mi)?;
        let t = (r - x).rem_euclid(mi);
        let k = mulmod(t as u128, inv as u128, mi as u128) as i128;
        x += m.checked_mul(k)?;
        m = m.checked_mul(mi)?;
        x = x.rem_euclid(m);
    }
    Some((x, m))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn primes_and_factors() {
        assert!(is_prime(1_000_000_007));
        assert!(!is_prime(561));
        assert!(is_prime(170141183460469231731687303715884105727)); // 2^127 - 1
        assert_eq!(factorize(360).unwrap(), vec![(2, 3), (3, 2), (5, 1)]);
        let n = 1_000_000_007u128 * 998_244_353;
        assert_eq!(factorize(n).unwrap(), vec![(998_244_353, 1), (1_000_000_007, 1)]);
        assert_eq!(totient(36).unwrap(), 12);
        assert_eq!(modinv(3, 11), Some(4));
        assert_eq!(mult_order(2, 7).unwrap(), 3);
        assert_eq!(primitive_root(7).unwrap(), Some(3));
        assert_eq!(crt(&[2, 3, 2], &[3, 5, 7]), Some((23, 105)));
        assert_eq!(divisors(12).unwrap(), vec![1, 2, 3, 4, 6, 12]);
    }
}
