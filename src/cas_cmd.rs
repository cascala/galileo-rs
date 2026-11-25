//! Commands for the symbolic tools in [`crate::cas`], integer functions,
//! more matrix functions, and plot labels.

use crate::build::*;
use crate::calc::{self, simplify};
use crate::cas::{self, Point, Side};
use crate::eval::{R, Session};
use crate::expr::{Expr, Obj};
use crate::linalg;
use crate::num::Number;
use crate::numeric;
use crate::parser::Node;
use crate::print::show;
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use std::rc::Rc;

fn name(n: &Node, what: &str) -> R<String> {
    match n {
        Node::Ident(s) => Ok(s.clone()),
        _ => Err(format!("{} must be a variable name", what)),
    }
}

fn names(n: &Node, what: &str) -> R<Vec<String>> {
    match n {
        Node::Matrix(rows) => rows.iter().flatten().map(|x| name(x, what)).collect(),
        Node::Ident(s) => Ok(vec![s.clone()]),
        _ => Err(format!("{} must be a list of names such as [x y]", what)),
    }
}

fn row(v: Vec<Expr>) -> Expr {
    if v.len() == 1 {
        v.into_iter().next().unwrap()
    } else {
        Expr::Matrix(vec![v])
    }
}

fn is_inf(e: &Expr) -> i32 {
    match e {
        Expr::Sym(s) if s == "inf" || s == "Inf" => 1,
        Expr::Mul(f)
            if f.len() == 2
                && f[0].num().is_some_and(|n| n.is_minus_one())
                && is_inf(&f[1]) == 1 =>
        {
            -1
        }
        Expr::Num(n) if !n.is_exact() && n.to_f64().is_infinite() => {
            if n.to_f64() > 0.0 {
                1
            } else {
                -1
            }
        }
        _ => 0,
    }
}

fn point(e: &Expr, side: Side) -> Point {
    match is_inf(e) {
        1 => Point::PosInf,
        -1 => Point::NegInf,
        _ => Point::At(e.clone(), side),
    }
}

fn integer(e: &Expr, f: &str) -> R<i128> {
    if let Some(n) = exact_int(e) {
        return Ok(n);
    }
    let orig = e;
    let e = calc::evalf(e);
    let v = e
        .num()
        .map(|n| n.to_f64())
        .ok_or_else(|| crate::extra_cmd::not_whole(orig, f))?;
    if v.fract() != 0.0 || v.abs() > 1e30 {
        return Err(crate::extra_cmd::not_whole(orig, f));
    }
    Ok(v as i128)
}

fn big_integer(e: &Expr, f: &str) -> R<BigInt> {
    match exact_big(e) {
        Some(n) => Ok(n),
        None => integer(e, f).map(BigInt::from),
    }
}

fn modulo(va: &Expr, vb: &Expr) -> R<Expr> {
    if let (Some(x), Some(y)) = (exact_int(va), exact_int(vb)) {
        if y == 0 {
            return Err("mod: division by zero".into());
        }
        return Ok(Expr::int(x.rem_euclid(y)));
    }
    if let (Some(x), Some(y)) = (exact_big(va), exact_big(vb)) {
        if y.is_zero() {
            return Err("mod: division by zero".into());
        }
        let r = x % &y;
        let r = if r.is_negative() { r + y.abs() } else { r };
        return Ok(crate::extra_cmd::bnum(r));
    }
    for v in [va, vb] {
        if crate::extra_cmd::is_huge(v) {
            return Err(crate::extra_cmd::not_whole(v, "mod"));
        }
    }
    let (x, y) = (real(va, "mod")?, real(vb, "mod")?);
    Ok(Expr::float(x - y * (x / y).floor()))
}

/// c/(a + b*sqrt(k)) -> c*(a - b*sqrt(k))/(a^2 - b^2*k), when that is simpler
fn rationalize_root(e: &Expr) -> Expr {
    let (n, d) = calc::num_den(e);
    let Expr::Add(terms) = &d else {
        return e.clone();
    };
    let is_root = |t: &Expr| format!("{:?}", t).contains("Pow") && show(t).contains("sqrt");
    if terms.len() != 2 || terms.iter().filter(|t| is_root(t)).count() != 1 {
        return e.clone();
    }
    let conj = add(terms
        .iter()
        .map(|t| {
            if is_root(t) {
                neg(t.clone())
            } else {
                t.clone()
            }
        })
        .collect());
    let top = calc::expand(&mul2(n, conj.clone()));
    let bottom = simplify(&calc::expand(&mul2(d.clone(), conj)));
    if bottom.num().is_none() {
        return e.clone();
    }
    let r = simplify(&calc::expand(&div(top, bottom)));
    if r.size() <= e.size() + 2 {
        r
    } else {
        e.clone()
    }
}

/// Eigenvectors (columns) and the matching eigenvalues, from the null space
/// of A - λI for each distinct exact eigenvalue.
pub(crate) fn eigvec(m: &linalg::Mat) -> R<(linalg::Mat, Vec<Expr>)> {
    let (r, c) = linalg::dims(m);
    if r != c {
        return Err("eigvec needs a square matrix".into());
    }
    let p = charpoly(m, "λ")?;
    let co = cas::poly_coeffs(&p, "λ").ok_or("eigvec: no characteristic polynomial")?;
    let Expr::Matrix(roots) = cas::poly_roots(&co).map(|v| Expr::Matrix(vec![v]))? else {
        unreachable!()
    };
    let mut distinct: Vec<Expr> = vec![];
    for l in roots.into_iter().flatten() {
        if !distinct
            .iter()
            .any(|d| simplify(&sub(d.clone(), l.clone())).is_zero())
        {
            distinct.push(l);
        }
    }
    let mut cols: Vec<Vec<Expr>> = vec![];
    let mut vals = vec![];
    for l in distinct {
        if !l.num().map_or(true, |n| n.is_exact()) {
            return Err("eigvec: the eigenvalues are only known numerically".into());
        }
        let shifted: linalg::Mat = (0..r)
            .map(|i| {
                (0..r)
                    .map(|j| {
                        if i == j {
                            simplify(&sub(m[i][j].clone(), l.clone()))
                        } else {
                            m[i][j].clone()
                        }
                    })
                    .collect()
            })
            .collect();
        let n = linalg::nullspace(&shifted);
        for k in 0..linalg::dims(&n).1 {
            // scale so the entries are as simple as possible: last nonzero entry 1
            cols.push((0..r).map(|i| rationalize_root(&n[i][k])).collect());
            vals.push(l.clone());
        }
    }
    if cols.is_empty() {
        return Err("eigvec: no exact eigenvectors found".into());
    }
    Ok((linalg::transpose(&cols), vals))
}

fn exact_int(e: &Expr) -> Option<i128> {
    e.num().filter(|n| n.is_exact()).and_then(|n| n.as_int())
}

fn exact_big(e: &Expr) -> Option<BigInt> {
    e.num().and_then(|n| n.as_bigint())
}

/// floor, ceil or round, exact for exact numbers of any size
fn round_exact(v: &Expr, f: &str) -> Option<R<Expr>> {
    match v {
        Expr::Matrix(m) => {
            let mut out = vec![];
            for r in m {
                let mut o = vec![];
                for e in r {
                    o.push(round_exact(e, f)?);
                }
                out.push(o.into_iter().collect::<R<Vec<_>>>());
            }
            Some(out.into_iter().collect::<R<Vec<_>>>().map(Expr::Matrix))
        }
        Expr::Num(n) if n.is_exact() => {
            let r = n.big()?;
            let r = match f {
                "floor" => r.floor(),
                "ceil" => r.ceil(),
                _ => r.round(),
            };
            Some(Ok(Expr::Num(Number::from_big(r))))
        }
        _ => None,
    }
}

fn real(e: &Expr, f: &str) -> R<f64> {
    if is_inf(e) != 0 {
        return Ok(is_inf(e) as f64 * f64::INFINITY);
    }
    let v = numeric::value(e);
    if v.is_finite() {
        Ok(v)
    } else {
        Err(format!("{} needs a number, got {}", f, show(e)))
    }
}

fn label_text(n: &Node, s: &mut Session) -> R<String> {
    match n {
        Node::Str(t) => Ok(t.clone()),
        Node::Ident(t) => Ok(t.clone()),
        other => Ok(show(&s.eval(other)?)),
    }
}

impl Session {
    /// Some(result) if f is one of the commands here
    pub(crate) fn cas_call(&mut self, f: &str, args: &[Node]) -> Option<R<Expr>> {
        let r = match f {
            "integrate" | "int" => self.integrate(args),
            "nintegrate" => self.nintegrate(args),
            "limit" => self.limit(args),
            "taylor" | "series" => self.taylor(args),
            "solve" => self.solve(args),
            "fsolve" => self.fsolve(args),
            "sum" | "prod" | "product" => self.sum(f != "sum", args),
            "factorial" | "floor" | "ceil" | "round" | "numer" | "denom" | "trace" | "size"
            | "eig" | "rref" | "rank" | "null" | "eigvec" => self.unary(f, args),
            "binomial" | "mod" | "gcd" | "lcm" | "dot" | "cross" | "charpoly" | "degree" => {
                self.binary(f, args)
            }
            "coeff" => self.coeff(args),
            "polarplot" => self.polarplot(args),
            "title" | "xlabel" | "ylabel" | "zlabel" | "labels" | "legend" => self.labels(f, args),
            _ => return None,
        };
        Some(r)
    }

    fn integrate(&mut self, args: &[Node]) -> R<Expr> {
        let usage = "usage: integrate(f, x) or integrate(f, x, a, b)";
        if args.len() != 2 && args.len() != 4 {
            return Err(usage.into());
        }
        let x = name(&args[1], "integrate(f, x): x")?;
        let fx = self.eval_hidden(&args[0], std::slice::from_ref(&x))?;
        if let Expr::Matrix(m) = &fx {
            // element by element
            let mut out = vec![];
            for r in m {
                let mut o = vec![];
                for e in r {
                    o.push(self.integrate_one(e, &x, args)?);
                }
                out.push(o);
            }
            return Ok(Expr::Matrix(out));
        }
        self.integrate_one(&fx, &x, args)
    }

    fn integrate_one(&mut self, fx: &Expr, x: &str, args: &[Node]) -> R<Expr> {
        if args.len() == 2 {
            return cas::antiderivative(fx, x)
                .ok_or_else(|| format!("integrate: can't find an antiderivative of {}; for a number use integrate(f, {}, a, b)", show(fx), x));
        }
        let a = self.eval(&args[2])?;
        let b = self.eval(&args[3])?;
        definite(fx, x, &a, &b, 0)
    }

    fn nintegrate(&mut self, args: &[Node]) -> R<Expr> {
        let [f, x, a, b] = args else {
            return Err("usage: nintegrate(f, x, a, b)".into());
        };
        let x = name(x, "nintegrate: x")?;
        let fx = self.eval_hidden(f, std::slice::from_ref(&x))?;
        let free = numeric::free(&fx, &[x.as_str()]);
        if !free.is_empty() {
            return Err(format!(
                "nintegrate: give values to {} first",
                free.join(", ")
            ));
        }
        let (a, b) = (self.eval(a)?, self.eval(b)?);
        let v = cas::nintegrate(&fx, &x, real(&a, "nintegrate")?, real(&b, "nintegrate")?)
            .map_err(|e| format!("nintegrate: {}", e))?;
        Ok(Expr::float(v))
    }

    fn limit(&mut self, args: &[Node]) -> R<Expr> {
        if args.len() != 3 && args.len() != 4 {
            return Err("usage: limit(f, x, a) or limit(f, x, a, left) / limit(f, x, a, right); a can be inf or -inf".into());
        }
        let x = name(&args[1], "limit(f, x, a): x")?;
        let fx = self.eval_hidden(&args[0], std::slice::from_ref(&x))?;
        let a = self.eval(&args[2])?;
        let side = match args.get(3) {
            None => Side::Both,
            Some(Node::Ident(s) | Node::Str(s)) if s == "left" || s == "-" => Side::Left,
            Some(Node::Ident(s) | Node::Str(s)) if s == "right" || s == "+" => Side::Right,
            Some(_) => return Err("limit: the fourth argument is left or right".into()),
        };
        cas::limit(&fx, &x, &point(&a, side))
    }

    fn taylor(&mut self, args: &[Node]) -> R<Expr> {
        let usage =
            "usage: taylor(f, x, a, n): the Taylor polynomial of f around x = a up to (x-a)^n";
        let (f, x, a, n) = match args {
            [f, x, a, n] => (f, x, Some(a), n),
            [f, x, n] => (f, x, None, n),
            _ => return Err(usage.into()),
        };
        let x = name(x, "taylor: x")?;
        let fx = self.eval_hidden(f, std::slice::from_ref(&x))?;
        let a = match a {
            Some(a) => self.eval(a)?,
            None => Expr::zero(),
        };
        let n = self.eval(n)?;
        let n = exact_int(&n)
            .filter(|k| (0..=40).contains(k))
            .ok_or("taylor: the order n must be a whole number from 0 to 40")?;
        cas::taylor(&fx, &x, &a, n as usize)
    }

    fn solve(&mut self, args: &[Node]) -> R<Expr> {
        if let [f, x, n] = args {
            return self.solve_mod(f, x, n);
        }
        let [f, x] = args else {
            return Err("usage: solve(f, x) solves f = 0; solve([f1 f2], [x y]) solves a linear system; A \\ b solves A*x = b".into());
        };
        let xs = names(x, "solve: the unknowns")?;
        let fx = self.eval_hidden(f, &xs)?;
        let eqs: Vec<Expr> = match fx {
            Expr::Matrix(m) => m
                .into_iter()
                .flatten()
                .map(|e| crate::eval::equation_lhs(&e))
                .collect(),
            e => vec![crate::eval::equation_lhs(&e)],
        };
        if eqs.len() == 1 && xs.len() == 1 {
            crate::radicals::begin();
            let r = cas::solve(&eqs[0], &xs[0]);
            let traces = crate::radicals::take();
            if r.is_ok() {
                if let Some(md) = crate::radicals::notes(&traces, &xs[0], self.teaching()) {
                    self.note2(crate::teach::plain(&md), md);
                }
            }
            if r.is_err() {
                // sin(x) = 1/2 and friends: the solutions in [0, 2 pi)
                if let Some(t) = crate::extra_cmd::solve_trig(&eqs[0], &xs[0]) {
                    return t.map(row);
                }
                if let Some(t) = crate::extra_cmd::solve_inverse(&eqs[0], &xs[0]) {
                    return t.map(row);
                }
            }
            return r.map(row);
        }
        cas::solve_linear(&eqs, &xs).map(row)
    }

    /// solve(f, x, N): the roots of a polynomial in Z/N
    fn solve_mod(&mut self, f: &Node, x: &Node, n: &Node) -> R<Expr> {
        let x = name(x, "solve: x")?;
        let n = match self.eval(n)? {
            Expr::Num(v) => v.as_bigint(),
            _ => None,
        }
        .ok_or("solve(f, x, N): N must be an integer")?;
        let fx = crate::eval::equation_lhs(&self.eval_hidden(f, std::slice::from_ref(&x))?);
        let c = cas::poly_coeffs(&fx, &x)
            .ok_or_else(|| format!("solve(f, x, N): {} is not a polynomial in {}", show(&fx), x))?;
        let mut ints = vec![];
        for e in &c {
            let r = e
                .num()
                .and_then(|v| v.big())
                .ok_or("solve(f, x, N): the coefficients must be integers or fractions")?;
            let inv = crate::numtheory::modinv(&r.denom().clone(), &n).ok_or_else(|| {
                format!(
                    "solve(f, x, N): the denominator {} has no inverse mod {}",
                    r.denom(),
                    n
                )
            })?;
            ints.push(r.numer() * inv);
        }
        let t = crate::modsolve::solve(&ints, &n)?;
        let md = crate::modsolve::notes(&t, &show(&fx), &x, self.teaching());
        self.note2(crate::teach::plain(&md), md);
        let roots: Vec<Expr> = t
            .roots
            .iter()
            .cloned()
            .map(crate::extra_cmd::bnum)
            .collect();
        Ok(if roots.is_empty() {
            Expr::Matrix(vec![])
        } else {
            row(roots)
        })
    }

    fn fsolve(&mut self, args: &[Node]) -> R<Expr> {
        let [f, x, g] = args else {
            return Err(
                "usage: fsolve(f, x, guess): a numeric root of f = 0 near the guess".into(),
            );
        };
        let x = name(x, "fsolve: x")?;
        let fx = crate::eval::equation_lhs(&self.eval_hidden(f, std::slice::from_ref(&x))?);
        let free = numeric::free(&fx, &[x.as_str()]);
        if !free.is_empty() {
            return Err(format!("fsolve: give values to {} first", free.join(", ")));
        }
        let g = self.eval(g)?;
        Ok(Expr::float(cas::fsolve(&fx, &x, real(&g, "fsolve")?)?))
    }

    fn sum(&mut self, product: bool, args: &[Node]) -> R<Expr> {
        let fname = if product { "prod" } else { "sum" };
        let [f, k, a, b] = args else {
            // sum(A): sum of the entries of a matrix
            if let [m] = args {
                if let Expr::Matrix(rows) = self.eval(m)? {
                    let all: Vec<Expr> = rows.into_iter().flatten().collect();
                    return Ok(if product { mul(all) } else { add(all) });
                }
            }
            return Err(format!(
                "usage: {}(f, k, a, b) for k = a, a+1, ..., b, or {}(A) over the entries of a matrix",
                fname, fname
            ));
        };
        let k = name(k, &format!("{}: k", fname))?;
        let (a, b) = (self.eval(a)?, self.eval(b)?);
        if !product && (is_inf(&b) == 1 || !b.symbols().is_empty()) {
            // closed forms: sum(k^2, k, 1, n), sum(1/2^k, k, 0, inf)
            let fk = self.eval_hidden(f, std::slice::from_ref(&k))?;
            return crate::extra_cmd::symbolic_sum(&fk, &k, &a, &b, is_inf(&b) == 1);
        }
        let (a, b) = (integer(&a, fname)?, integer(&b, fname)?);
        if b - a > 100_000 {
            return Err(format!("{}: at most 100000 terms", fname));
        }
        // evaluate the body with k = a, a+1, ..., b, so factorial(k) etc. work
        let saved = self.vars.remove(&k);
        let mut terms: Vec<Expr> = vec![];
        let mut err = None;
        for i in a..=b {
            self.vars.insert(k.clone(), Expr::int(i));
            match self.eval(f) {
                Ok(v) => terms.push(v),
                Err(e) => {
                    err = Some(e);
                    break;
                }
            }
        }
        match saved {
            Some(v) => {
                self.vars.insert(k.clone(), v);
            }
            None => {
                self.vars.remove(&k);
            }
        }
        if let Some(e) = err {
            return Err(e);
        }
        let r = if product { mul(terms) } else { add(terms) };
        Ok(if r.size() < 200 { simplify(&r) } else { r })
    }

    fn unary(&mut self, f: &str, args: &[Node]) -> R<Expr> {
        let [a] = args else {
            return Err(format!("{} takes 1 argument", f));
        };
        let v = self.eval(a)?;
        let round = |g: fn(f64) -> f64| -> R<Expr> {
            elementwise_num(&v, f, &|x| {
                let r = g(x);
                if r.abs() < 1e30 {
                    Expr::int(r as i128)
                } else {
                    Expr::float(r)
                }
            })
        };
        match f {
            "factorial" => cas::factorial(integer(&v, f)?),
            "floor" | "ceil" | "round" if round_exact(&v, f).is_some() => {
                round_exact(&v, f).unwrap()
            }
            "floor" => round(f64::floor),
            "ceil" => round(f64::ceil),
            "round" => round(f64::round),
            "numer" | "denom" => {
                let (n, d) = calc::num_den(&calc::together(&v));
                Ok(if f == "numer" { n } else { d })
            }
            "rref" | "rank" | "null" => {
                let Expr::Matrix(m) = v else {
                    return Err(format!("{} needs a matrix", f));
                };
                match f {
                    "rref" => Ok(Expr::Matrix(linalg::rref(&m).0)),
                    "rank" => Ok(Expr::int(linalg::rref(&m).1.len() as i128)),
                    _ => {
                        let n = linalg::nullspace(&m);
                        Ok(if n.is_empty() {
                            Expr::Matrix(vec![])
                        } else {
                            Expr::Matrix(n)
                        })
                    }
                }
            }
            "eigvec" => {
                let Expr::Matrix(m) = v else {
                    return Err("eigvec needs a square matrix".into());
                };
                eigvec(&m).map(|(vecs, _)| Expr::Matrix(vecs))
            }
            "trace" | "size" | "eig" => {
                let m = match v {
                    Expr::Matrix(m) => m,
                    _ if f == "size" => {
                        return Ok(Expr::Matrix(vec![vec![Expr::one(), Expr::one()]]));
                    }
                    s => vec![vec![s]],
                };
                let (r, c) = linalg::dims(&m);
                match f {
                    "size" => Ok(Expr::Matrix(vec![vec![
                        Expr::int(r as i128),
                        Expr::int(c as i128),
                    ]])),
                    _ if r != c => Err(format!("{} needs a square matrix", f)),
                    "trace" => Ok(simplify(&add((0..r).map(|i| m[i][i].clone()).collect()))),
                    _ => {
                        let p = charpoly(&m, "λ")?;
                        let c =
                            cas::poly_coeffs(&p, "λ").ok_or("eig: no characteristic polynomial")?;
                        cas::poly_roots(&c).map(row)
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, f: &str, args: &[Node]) -> R<Expr> {
        let [a, b] = args else {
            return Err(format!("{} takes 2 arguments", f));
        };
        if f == "charpoly" || f == "degree" {
            let x = name(b, &format!("{}: the second argument", f))?;
            let v = self.eval_hidden(a, std::slice::from_ref(&x))?;
            if f == "degree" {
                let c = cas::poly_coeffs(&v, &x)
                    .ok_or_else(|| format!("degree: {} is not a polynomial in {}", show(&v), x))?;
                return Ok(Expr::int(c.len() as i128 - 1));
            }
            let Expr::Matrix(m) = v else {
                return Err("charpoly needs a matrix".into());
            };
            return charpoly(&m, &x);
        }
        let (va, vb) = (self.eval(a)?, self.eval(b)?);
        match f {
            "binomial" => Ok(cas::binomial(integer(&va, f)?, integer(&vb, f)?)),
            "gcd" | "lcm" => {
                let (x, y) = (big_integer(&va, f)?, big_integer(&vb, f)?);
                let g = num_integer::Integer::gcd(&x, &y);
                Ok(crate::extra_cmd::bnum(if f == "gcd" {
                    g
                } else if g.is_zero() {
                    g
                } else {
                    (x / &g * y).abs()
                }))
            }
            "mod" => match (&va, &vb) {
                (Expr::Matrix(m), b) => Ok(Expr::Matrix(
                    m.iter()
                        .map(|r| r.iter().map(|x| modulo(x, b)).collect::<R<_>>())
                        .collect::<R<_>>()?,
                )),
                (a, Expr::Matrix(m)) => Ok(Expr::Matrix(
                    m.iter()
                        .map(|r| r.iter().map(|y| modulo(a, y)).collect::<R<_>>())
                        .collect::<R<_>>()?,
                )),
                (a, b) => modulo(a, b),
            },
            "dot" | "cross" => {
                let flat = |e: Expr| -> Vec<Expr> {
                    match e {
                        Expr::Matrix(m) => m.into_iter().flatten().collect(),
                        o => vec![o],
                    }
                };
                let (u, v) = (flat(va), flat(vb));
                if u.len() != v.len() {
                    return Err(format!("{}: the vectors must have the same length", f));
                }
                if f == "dot" {
                    return Ok(simplify(&add(u
                        .iter()
                        .zip(&v)
                        .map(|(p, q)| mul2(p.clone(), q.clone()))
                        .collect())));
                }
                if u.len() != 3 {
                    return Err("cross needs two vectors of length 3".into());
                }
                let c = |i: usize, j: usize| {
                    simplify(&sub(
                        mul2(u[i].clone(), v[j].clone()),
                        mul2(u[j].clone(), v[i].clone()),
                    ))
                };
                Ok(Expr::Matrix(vec![vec![c(1, 2), c(2, 0), c(0, 1)]]))
            }
            _ => unreachable!(),
        }
    }

    fn coeff(&mut self, args: &[Node]) -> R<Expr> {
        let [p, x, n] = args else {
            return Err("usage: coeff(p, x, n): the coefficient of x^n in p".into());
        };
        let x = name(x, "coeff: x")?;
        let v = self.eval_hidden(p, std::slice::from_ref(&x))?;
        let n = self.eval(n)?;
        let n = exact_int(&n)
            .filter(|k| *k >= 0)
            .ok_or("coeff: n must be a whole number")? as usize;
        let c = cas::poly_coeffs(&v, &x)
            .ok_or_else(|| format!("coeff: {} is not a polynomial in {}", show(&v), x))?;
        Ok(c.get(n).map(simplify).unwrap_or_else(Expr::zero))
    }

    fn polarplot(&mut self, args: &[Node]) -> R<Expr> {
        let [r, t, a, b] = args else {
            return Err(
                "usage: polarplot(r, phi, from, to): the curve at distance r(phi) from the origin"
                    .into(),
            );
        };
        let tn = name(t, "polarplot: the angle")?;
        let rv = self.eval_hidden(r, std::slice::from_ref(&tn))?;
        let (a, b) = (self.eval(a)?, self.eval(b)?);
        let (a, b) = (real(&a, "polarplot")?, real(&b, "polarplot")?);
        let free = numeric::free(&rv, &[tn.as_str()]);
        if !free.is_empty() {
            return Err(format!(
                "polarplot: give values to {} first",
                free.join(", ")
            ));
        }
        let c = numeric::compile(&rv, &[tn.as_str()]);
        let pts = crate::plot::steps(a, b, crate::plot::SAMPLES)
            .iter()
            .map(|&s| {
                let rr = numeric::run(&c, &[s]);
                (rr * s.cos(), rr * s.sin())
            })
            .collect();
        let s = crate::plot::Series {
            label: format!("r = {}", crate::parser::source(r)),
            points: pts,
            ..Default::default()
        };
        Ok(Expr::Obj(Rc::new(Obj::Figure(crate::plot::Figure::new(
            vec![s],
            "x",
            "y",
            true,
        )))))
    }

    fn labels(&mut self, f: &str, args: &[Node]) -> R<Expr> {
        let usage = match f {
            "labels" => {
                "usage: labels(p, \"x label\", \"y label\") (and a third for z on 3D plots)"
            }
            "legend" => "usage: legend(p, \"first curve\", \"second curve\", ...)",
            _ => {
                "usage: title(p, \"text\"), xlabel(p, \"text\"), ylabel(p, \"text\"), zlabel(p, \"text\")"
            }
        };
        if args.len() < 2 {
            return Err(usage.into());
        }
        let p = self.eval(&args[0])?;
        let mut texts = vec![];
        for a in &args[1..] {
            texts.push(label_text(a, self)?);
        }
        let Expr::Obj(o) = &p else {
            return Err(format!("{}: the first argument must be a plot", f));
        };
        let out = match &**o {
            Obj::Figure(fig) => {
                let mut g = fig.clone();
                match f {
                    "title" => g.title = texts.join(" "),
                    "xlabel" => g.x_label = texts.join(" "),
                    "ylabel" => g.y_label = texts.join(" "),
                    "labels" => {
                        g.x_label = texts[0].clone();
                        if let Some(y) = texts.get(1) {
                            g.y_label = y.clone();
                        }
                    }
                    "legend" => {
                        for (s, t) in g.series.iter_mut().zip(&texts) {
                            s.label = t.clone();
                        }
                    }
                    _ => return Err("zlabel needs a 3D plot".into()),
                }
                Obj::Figure(g)
            }
            Obj::Figure3D(fig) => {
                let mut g = fig.clone();
                match f {
                    "title" => g.title = texts.join(" "),
                    "xlabel" => g.labels[0] = texts.join(" "),
                    "ylabel" => g.labels[1] = texts.join(" "),
                    "zlabel" => g.labels[2] = texts.join(" "),
                    "labels" => {
                        for (l, t) in g.labels.iter_mut().zip(&texts) {
                            *l = t.clone();
                        }
                    }
                    "legend" => {
                        let mut it = texts.iter();
                        for s in g.surfaces.iter_mut() {
                            if let Some(t) = it.next() {
                                s.label = t.clone();
                            }
                        }
                        for c in g.curves.iter_mut() {
                            if let Some(t) = it.next() {
                                c.label = t.clone();
                            }
                        }
                    }
                    _ => unreachable!(),
                }
                Obj::Figure3D(g)
            }
            _ => return Err(format!("{}: the first argument must be a plot", f)),
        };
        Ok(Expr::Obj(Rc::new(out)))
    }
}

fn elementwise_num(v: &Expr, f: &str, g: &dyn Fn(f64) -> Expr) -> R<Expr> {
    match v {
        Expr::Matrix(m) => {
            let mut out = vec![];
            for r in m {
                let mut o = vec![];
                for e in r {
                    o.push(elementwise_num(e, f, g)?);
                }
                out.push(o);
            }
            Ok(Expr::Matrix(out))
        }
        e => Ok(g(real(e, f)?)),
    }
}

/// det(A - x I), expanded
fn charpoly(m: &[Vec<Expr>], x: &str) -> R<Expr> {
    let (r, c) = linalg::dims(&m.to_vec());
    if r != c {
        return Err("charpoly needs a square matrix".into());
    }
    let xv = Expr::sym(x);
    let a: Vec<Vec<Expr>> = (0..r)
        .map(|i| {
            (0..r)
                .map(|j| {
                    if i == j {
                        sub(m[i][j].clone(), xv.clone())
                    } else {
                        m[i][j].clone()
                    }
                })
                .collect()
        })
        .collect();
    let d = linalg::det(&a)?;
    let p = calc::expand(&d);
    Ok(match cas::poly_coeffs(&p, x) {
        Some(c) => add(c
            .iter()
            .enumerate()
            .map(|(k, v)| mul2(simplify(v), pow(xv.clone(), Expr::int(k as i128))))
            .collect()),
        None => p,
    })
}

/// definite integral; splits at singular points inside the interval
fn definite(fx: &Expr, x: &str, a: &Expr, b: &Expr, depth: usize) -> R<Expr> {
    let free = numeric::free(fx, &[x]);
    let (an, bn) = (real(a, "integrate").ok(), real(b, "integrate").ok());
    let numeric_ok = free.is_empty() && an.is_some() && bn.is_some();
    // singularities strictly inside: zeros of the denominator
    if let (Some(lo), Some(hi)) = (an, bn) {
        if depth < 4 {
            let (_, d) = calc::num_den(&calc::together(fx));
            if d.contains_sym(x) {
                if let Ok(roots) = cas::solve(&d, x) {
                    let (l, h) = if lo < hi { (lo, hi) } else { (hi, lo) };
                    let mut inside: Vec<(f64, Expr)> = roots
                        .into_iter()
                        .filter(|r| r.symbols().is_empty())
                        .filter_map(|r| {
                            let v = numeric::value(&r);
                            (v.is_finite() && v > l && v < h).then_some((v, r))
                        })
                        .collect();
                    inside.sort_by(|p, q| p.0.partial_cmp(&q.0).unwrap());
                    if !inside.is_empty() {
                        let mut cuts = vec![if lo < hi { a.clone() } else { b.clone() }];
                        cuts.extend(inside.into_iter().map(|p| p.1));
                        cuts.push(if lo < hi { b.clone() } else { a.clone() });
                        let mut parts = vec![];
                        for w in cuts.windows(2) {
                            parts.push(definite(fx, x, &w[0], &w[1], depth + 1)?);
                        }
                        let s = simplify(&add(parts));
                        return Ok(if lo < hi { s } else { neg(s) });
                    }
                }
            }
        }
    }
    let quad = || -> R<f64> { cas::nintegrate(fx, x, an.unwrap(), bn.unwrap()) };
    let forward = match (an, bn) {
        (Some(p), Some(q)) => p <= q,
        _ => true,
    };
    if let Some(big_f) = cas::antiderivative(fx, x) {
        let (sa, sb) = if forward {
            (Side::Right, Side::Left)
        } else {
            (Side::Left, Side::Right)
        };
        let fa = cas::limit(&big_f, x, &point(a, sa));
        let fb = cas::limit(&big_f, x, &point(b, sb));
        if let (Ok(fa), Ok(fb)) = (fa, fb) {
            if is_inf(&fa) != 0 || is_inf(&fb) != 0 {
                return Err("integrate: the integral diverges".into());
            }
            let res = simplify(&sub(fb, fa));
            if numeric_ok {
                if let Ok(q) = quad() {
                    let v = numeric::value(&res);
                    if !v.is_finite() || (v - q).abs() > 1e-6 * (1.0 + q.abs()) {
                        return Ok(Expr::float(q));
                    }
                }
            }
            return Ok(res);
        }
    }
    if numeric_ok {
        return quad()
            .map(Expr::float)
            .map_err(|e| format!("integrate: {}", e));
    }
    if !free.is_empty() {
        return Err(format!(
            "integrate: can't integrate {} symbolically; give values to {} to get a number",
            show(fx),
            free.join(", ")
        ));
    }
    Err(format!(
        "integrate: can't integrate {} from {} to {}",
        show(fx),
        show(a),
        show(b)
    ))
}
