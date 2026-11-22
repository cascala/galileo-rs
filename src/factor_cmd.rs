//! Commands for algebraic factoring: factor() on polynomials over the
//! integers and on whole numbers of the form a^n ± b^n, cyclotomic(n),
//! algfactor(n) (the algebraic pieces of a number), factorsearch (hunt for
//! factorizations in a family of polynomials) and digits(n).

use crate::algfactor::{self as af, ZPoly};
use crate::build::*;
use crate::calc;
use crate::discover::Report;
use crate::eval::{R, Session};
use crate::expr::{Expr, Obj};
use crate::latex::latex;
use crate::num::Number;
use crate::parser::{Node, Op};
use crate::print::show;
use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use std::collections::BTreeMap;
use std::rc::Rc;

fn report(kind: &'static str, text: String, markdown: String, formulas: Vec<Expr>) -> Expr {
    Expr::Obj(Rc::new(Obj::Report(Report { kind, text, markdown, formulas, best: 0, vars: vec![], order: 0, target: 0 })))
}

fn int(n: i128) -> Expr {
    Expr::int(n)
}

fn bnum(n: BigInt) -> Expr {
    Expr::Num(Number::from_bigint(n))
}

// ---------- expressions <-> integer polynomials ----------

/// the terms of a polynomial in vars as exponent vectors with rational coefficients
fn terms_of(e: &Expr, vars: &[String]) -> Option<Vec<(Vec<usize>, BigRational)>> {
    let ex = calc::expand(e);
    let ts = match &ex {
        Expr::Add(t) => t.clone(),
        o => vec![o.clone()],
    };
    let mut out = vec![];
    for t in ts {
        let (c, rest) = t.coeff_and_rest();
        let c = c.big().filter(|_| c.is_exact())?;
        let mut ex = vec![0usize; vars.len()];
        let fs = match rest {
            Expr::Mul(f) => f,
            o => vec![o],
        };
        for f in fs {
            match &f {
                Expr::Num(n) if n.is_one() => {}
                Expr::Sym(s) => ex[vars.iter().position(|v| v == s)?] += 1,
                Expr::Pow(b, k) => {
                    let Expr::Sym(s) = &**b else { return None };
                    let k = k.num().filter(|k| k.is_exact() && k.is_integer() && !k.is_negative())?.as_int()?;
                    if k > 5000 {
                        return None;
                    }
                    ex[vars.iter().position(|v| v == s)?] += k as usize;
                }
                _ => return None,
            }
        }
        out.push((ex, c));
    }
    Some(out)
}

fn upoly_expr(p: &[BigInt], x: &Expr) -> Expr {
    add(p.iter().enumerate().filter(|(_, c)| !c.is_zero()).map(|(k, c)| mul2(bnum(c.clone()), pow(x.clone(), int(k as i128)))).collect())
}

/// homogeneous version of p: sum p_i x^i y^(d - i)
fn hpoly_expr(p: &[BigInt], x: &Expr, y: &Expr) -> Expr {
    let d = p.len() - 1;
    add(p
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.is_zero())
        .map(|(k, c)| mul(vec![bnum(c.clone()), pow(x.clone(), int(k as i128)), pow(y.clone(), int((d - k) as i128))]))
        .collect())
}

/// Factorization of a polynomial with rational coefficients in one variable,
/// or homogeneous in two: (constant, [(factor, multiplicity)]).
pub fn factor_parts(e: &Expr) -> Option<(Number, Vec<(Expr, u32)>)> {
    let vars = e.symbols();
    if vars.is_empty() || vars.len() > 2 {
        return None;
    }
    let ts = terms_of(e, &vars)?;
    let den = ts.iter().fold(BigInt::one(), |acc, (_, c)| acc.lcm(c.denom()));
    let total: Vec<usize> = ts.iter().filter(|(_, c)| !c.is_zero()).map(|(ex, _)| ex.iter().sum()).collect();
    let dtot = *total.first()?;
    if vars.len() == 2 && total.iter().any(|&d| d != dtot) {
        return None;
    }
    // coefficients in the first variable (the second is set to 1)
    let dmax = ts.iter().map(|(ex, _)| ex[0]).max()?;
    let mut p: ZPoly = vec![BigInt::zero(); dmax + 1];
    for (ex, c) in &ts {
        p[ex[0]] += (c * BigRational::from_integer(den.clone())).to_integer();
    }
    let (c, fs) = af::factor_zpoly(&p)?;
    let x = Expr::sym(&vars[0]);
    let mut out: Vec<(Expr, u32)> = vec![];
    if vars.len() == 2 {
        let y = Expr::sym(&vars[1]);
        let degf = af::trim(p.clone()).len().saturating_sub(1);
        if dtot > degf {
            out.push((y.clone(), (dtot - degf) as u32));
        }
        for (f, m) in fs {
            out.push((hpoly_expr(&f, &x, &y), m));
        }
    } else {
        for (f, m) in fs {
            out.push((upoly_expr(&f, &x), m));
        }
    }
    let k = Number::from_big(BigRational::new(c, den));
    Some((k, out))
}

fn assemble(k: Number, fs: &[(Expr, u32)]) -> Expr {
    let mut v = vec![Expr::Num(k)];
    for (f, m) in fs {
        v.push(pow(f.clone(), int(*m as i128)));
    }
    mul(v)
}

/// factor() on a polynomial: Some when it is a polynomial we can handle
pub fn factor_poly(e: &Expr) -> Option<Expr> {
    let (k, fs) = factor_parts(e)?;
    let nontrivial = fs.iter().map(|(_, m)| *m as usize).sum::<usize>() >= 2 || !k.is_one();
    if !nontrivial {
        return Some(e.clone());
    }
    Some(assemble(k, &fs))
}

// ---------- whole numbers ----------

fn digits(n: &BigInt) -> usize {
    n.abs().to_string().len()
}

struct IntFactors {
    sign: bool,
    primes: BTreeMap<BigInt, u32>,
    composites: Vec<BigInt>,
}

impl IntFactors {
    fn text(&self) -> String {
        let mut parts = vec![];
        if self.sign {
            parts.push("-1".to_string());
        }
        for (p, e) in &self.primes {
            parts.push(if *e == 1 { p.to_string() } else { format!("{}^{}", p, e) });
        }
        let labels = self.labels();
        parts.extend(labels.iter().cloned());
        let mut s = parts.join(" * ");
        for (l, c) in labels.iter().zip(&self.composites) {
            s.push_str(&format!("\n{} = {} (composite: no factor found in reasonable time)", l, c));
        }
        s
    }
    fn labels(&self) -> Vec<String> {
        let mut seen: BTreeMap<usize, usize> = BTreeMap::new();
        self.composites
            .iter()
            .map(|c| {
                let d = digits(c);
                let k = seen.entry(d).or_insert(0);
                *k += 1;
                if *k == 1 { format!("C{}", d) } else { format!("C{}.{}", d, k) }
            })
            .collect()
    }
}

/// a^n ± b^n with a, b >= 1, read from the expression as typed: (a, b, n, plus, negative)
fn binomial_from_node(s: &mut Session, node: &Node) -> Option<(BigInt, BigInt, u64, bool, bool)> {
    let Node::Bin(op, l, r) = node else { return None };
    let plus = match op {
        Op::Add => true,
        Op::Sub => false,
        _ => return None,
    };
    let mut side = |n: &Node| -> Option<(BigInt, u64)> {
        match n {
            Node::Bin(Op::Pow, b, e) => {
                let b = s.eval(b).ok()?.num()?.as_bigint()?;
                let e = s.eval(e).ok()?.num()?.as_bigint()?.to_u64()?;
                Some((b, e))
            }
            _ => Some((s.eval(n).ok()?.num()?.as_bigint()?, 1)),
        }
    };
    let (a, n1) = side(l)?;
    let (b, n2) = side(r)?;
    if a.is_negative() || b.is_negative() || a.is_zero() || b.is_zero() {
        return None;
    }
    let (n1, n2) = match (a.is_one(), b.is_one()) {
        (true, false) => (n2, n2),
        (false, true) => (n1, n1),
        _ => (n1, n2),
    };
    let k = n1.gcd(&n2);
    if k < 2 {
        return None;
    }
    let a = num_traits::pow(a, (n1 / k) as usize);
    let b = num_traits::pow(b, (n2 / k) as usize);
    if !plus && a < b {
        return Some((b, a, k, false, true));
    }
    Some((a, b, k, plus, false))
}

/// the algebraic form of n: from the expression typed, or recognized as b^k ± 1
fn int_form(s: &mut Session, node: Option<&Node>, n: &BigInt, detect: bool) -> Option<(BigInt, BigInt, u64, bool, bool)> {
    let check = |a: &BigInt, b: &BigInt, k: u64, plus: bool, neg: bool| {
        let v = if plus { num_traits::pow(a.clone(), k as usize) + num_traits::pow(b.clone(), k as usize) } else { num_traits::pow(a.clone(), k as usize) - num_traits::pow(b.clone(), k as usize) };
        if neg { -v == *n } else { v == *n }
    };
    if let Some(f) = node.and_then(|nd| binomial_from_node(s, nd)) {
        if check(&f.0, &f.1, f.2, f.3, f.4) {
            return Some(f);
        }
    }
    if detect {
        let (b, k, plus) = af::detect_binomial(&n.abs())?;
        return Some((b, BigInt::one(), k, plus, n.is_negative()));
    }
    None
}

fn form_latex(a: &BigInt, b: &BigInt, k: u64, plus: bool) -> String {
    let op = if plus { "+" } else { "-" };
    if b.is_one() { format!("{}^{{{}}} {} 1", a, k, op) } else { format!("{}^{{{}}} {} {}^{{{}}}", a, k, op, b, k) }
}

impl Session {
    /// factor(n) for a whole number: split algebraically first when n = a^n ± b^n
    pub(crate) fn factor_integer(&mut self, node: &Node, n: &BigInt) -> R<String> {
        let detect = n.bits() > 100;
        if let Some((a, b, k, plus, neg)) = int_form(self, Some(node), n, detect) {
            if let Some(pieces) = af::algebraic_pieces(&a, &b, k, plus) {
                let mut f = IntFactors { sign: neg, primes: BTreeMap::new(), composites: vec![] };
                for p in &pieces {
                    let (ps, cs) = af::factor_partial_known(&p.value, p.d);
                    for q in ps {
                        *f.primes.entry(q).or_insert(0) += 1;
                    }
                    f.composites.extend(cs);
                }
                return Ok(f.text());
            }
        }
        if n.bits() <= 120 {
            return crate::extra_cmd::factored_text(n);
        }
        let (ps, cs) = af::factor_partial(n);
        if ps.is_empty() {
            return Err(format!("could not find a factor of this {}-digit number in reasonable time", digits(n)));
        }
        let mut f = IntFactors { sign: n.is_negative(), primes: BTreeMap::new(), composites: cs };
        for q in ps {
            *f.primes.entry(q).or_insert(0) += 1;
        }
        Ok(f.text())
    }

    /// Some(result) if f is one of the commands here
    pub(crate) fn factor_call(&mut self, f: &str, args: &[Node]) -> Option<R<Expr>> {
        Some(match f {
            "cyclotomic" => self.cyclotomic_cmd(args),
            "algfactor" => self.algfactor_cmd(args),
            "factorsearch" => self.factorsearch_cmd(args),
            "digits" => (|| {
                let [a] = args else { return Err("digits takes 1 argument: digits(n)".into()) };
                let v = self.eval(a)?;
                let n = v.num().and_then(|n| n.as_bigint()).ok_or_else(|| format!("digits needs a whole number, got {}", show(&v)))?;
                Ok(int(digits(&n) as i128))
            })(),
            _ => return None,
        })
    }

    fn cyclotomic_cmd(&mut self, args: &[Node]) -> R<Expr> {
        let usage = "usage: cyclotomic(n) is the polynomial Phi_n(x); cyclotomic(n, v) puts v (a number or an expression) in place of x";
        let (n, v) = match args {
            [n] => (self.eval(n)?, Expr::sym("x")),
            [n, v] => (self.eval(n)?, self.eval(v)?),
            _ => return Err(usage.into()),
        };
        let n = n.num().and_then(|n| n.as_int()).filter(|&n| (1..=20_000).contains(&n)).ok_or("cyclotomic: n must be a whole number from 1 to 20000")? as u64;
        if let Some(b) = v.num().and_then(|x| x.as_bigint()) {
            return Ok(bnum(af::cyclotomic_value(n, &b, &BigInt::one())));
        }
        let p = af::cyclotomic(n);
        let x = Expr::sym("x");
        let e = upoly_expr(&p, &x);
        Ok(if v == x { e } else { calc::subst(&e, "x", &v) })
    }

    fn algfactor_cmd(&mut self, args: &[Node]) -> R<Expr> {
        let [a] = args else { return Err("algfactor takes 1 argument: algfactor(2^58 + 1)".into()) };
        let v = self.eval(a)?;
        let n = v.num().and_then(|n| n.as_bigint()).filter(|n| n.abs() > BigInt::one()).ok_or("algfactor needs a whole number of 2 or more")?;
        let form = int_form(self, Some(a), &n, true);
        let (pieces, head_md, head_tx) = match &form {
            Some((a, b, k, plus, _)) => match af::algebraic_pieces(a, b, *k, *plus) {
                Some(p) => {
                    let f = form_latex(a, b, *k, *plus);
                    let n_au = p.iter().filter(|x| x.text.starts_with('L')).count();
                    let extra = if n_au > 0 { format!(", {} of them Aurifeuillean pairs $L \\cdot M$", n_au) } else { String::new() };
                    (
                        p.clone(),
                        format!("$${}$$ splits algebraically into {} pieces{}:", f, p.len(), extra),
                        format!("{} splits algebraically into {} pieces:", f.replace(['{', '}'], ""), p.len()),
                    )
                }
                None => (vec![], String::new(), String::new()),
            },
            None => (vec![], String::new(), String::new()),
        };
        let pieces = if pieces.is_empty() {
            vec![af::Piece { text: n.to_string(), latex: n.to_string(), value: n.abs(), d: 0 }]
        } else {
            pieces
        };
        let (head_md, head_tx) = if head_md.is_empty() {
            let m = "This number has no algebraic form $a^n \\pm b^n$ that splits, so it is one piece:".to_string();
            (m.clone(), m.replace('$', "").replace("\\pm", "±"))
        } else {
            (head_md, head_tx)
        };
        let mut md = vec![head_md, String::new(), "| piece | digits | factors |".into(), "|---|---|---|".into()];
        let mut tx = vec![head_tx];
        let (mut nprime, mut ncomp, mut compdig) = (0, 0, 0);
        let mut formulas = vec![];
        for p in &pieces {
            let (ps, cs) = af::factor_partial_known(&p.value, p.d);
            nprime += ps.len();
            ncomp += cs.len();
            compdig += cs.iter().map(digits).sum::<usize>();
            let mut grouped: BTreeMap<BigInt, u32> = BTreeMap::new();
            for q in ps {
                *grouped.entry(q).or_insert(0) += 1;
            }
            let mut fac: Vec<String> = grouped.iter().map(|(q, e)| if *e == 1 { q.to_string() } else { format!("{}^{}", q, e) }).collect();
            let mut fac_md: Vec<String> = grouped.iter().map(|(q, e)| if *e == 1 { q.to_string() } else { format!("{}<sup>{}</sup>", q, e) }).collect();
            for c in &cs {
                fac.push(format!("C{} (composite, no factor found)", digits(c)));
                fac_md.push(format!("**C{}** (composite, no factor found)", digits(c)));
            }
            let single_prime = grouped.len() == 1 && cs.is_empty() && grouped.values().all(|e| *e == 1);
            let cell = if single_prime { "prime".to_string() } else { fac_md.join(" · ") };
            md.push(format!("| ${}$ | {} | {} |", p.latex, digits(&p.value), cell));
            let shown = if digits(&p.value) <= 40 { format!(" = {}", p.value) } else { format!(" ({} digits)", digits(&p.value)) };
            tx.push(format!("  {}{}: {}", p.text, shown, if single_prime { "prime".into() } else { fac.join(" * ") }));
            formulas.push(bnum(p.value.clone()));
        }
        let summary = if ncomp == 0 {
            format!("All {} prime factors found.", nprime)
        } else {
            format!(
                "{} prime factors found; {} composite part{} with {} digits in all did not split in reasonable time.",
                nprime,
                ncomp,
                if ncomp == 1 { "" } else { "s" },
                compdig
            )
        };
        md.push(String::new());
        md.push(summary.clone());
        tx.push(summary);
        Ok(report("Algebraic factors", tx.join("\n"), md.join("\n"), formulas))
    }

    fn factorsearch_cmd(&mut self, args: &[Node]) -> R<Expr> {
        let usage = "usage: factorsearch(expr, k, a, b) factors expr over the integers for k = a, a+1, ..., b and lists the k for which it splits; expr must be a polynomial in one other variable, or homogeneous in two";
        let [body, kn, a, b, rest @ ..] = args else { return Err(usage.into()) };
        let Node::Ident(k) = kn else { return Err(usage.into()) };
        let get = |s: &mut Session, n: &Node| -> R<i128> { s.eval(n)?.num().and_then(|x| x.as_int()).ok_or_else(|| usage.to_string()) };
        let (a, b) = (get(self, a)?, get(self, b)?);
        let step = match rest {
            [s] => get(self, s)?,
            [] => 1,
            _ => return Err(usage.into()),
        };
        if step <= 0 || b < a || (b - a) / step > 5000 {
            return Err("factorsearch: give a <= b, a positive step and at most 5000 values".into());
        }
        let mut vals = vec![];
        let mut kv = a;
        while kv <= b {
            vals.push(int(kv));
            kv += step;
        }
        let polys = self.with_values(body, k, &vals)?;
        let mut md = vec![format!("| {} | polynomial | factors |", k), "|---|---|---|".into()];
        let mut tx = vec![];
        let mut formulas = vec![];
        let (total, mut hits) = (vals.len(), 0);
        for (kv, e) in vals.iter().zip(polys) {
            let ek = calc::expand(&e);
            if let Some((c, fs)) = factor_parts(&ek) {
                let count: u32 = fs.iter().map(|(_, m)| *m).sum();
                let monomial = fs.iter().all(|(f, _)| matches!(f, Expr::Sym(_)));
                if count >= 2 && !monomial {
                    hits += 1;
                    let fe = assemble(c, &fs);
                    md.push(format!("| {} | ${}$ | ${}$ |", show(kv), latex(&ek), latex(&fe)));
                    tx.push(format!("{} = {}: {} = {}", k, show(kv), show(&ek), show(&fe)));
                    formulas.push(fe);
                }
            }
        }
        let summary = format!("{} of {} values of {} give a polynomial that factors over the integers.", hits, total, k);
        if hits == 0 {
            md.clear();
        }
        md.push(String::new());
        md.push(summary.clone());
        tx.push(summary);
        Ok(report("Factor search", tx.join("\n"), md.join("\n"), formulas))
    }
}
