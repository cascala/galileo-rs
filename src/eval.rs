//! Evaluating parsed statements against a session (variables, `ans`).

use crate::build::{self, *};
use crate::calc;
use crate::expr::{Const, Expr, Func};
use crate::linalg::{self, Mat};
use crate::num::Number;
use crate::parser::{parse, Node, Op, Stmt};
use crate::print::show;
use std::collections::BTreeMap;

pub type R<T> = Result<T, String>;

/// What a statement produced
#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    Value(Expr),
    /// assignments of several names, e.g. [L,U,P] = lu(A)
    Assigned(Vec<(String, Expr)>),
    Text(String),
    Nothing,
    Exit,
}

impl Output {
    pub fn render(&self) -> String {
        match self {
            Output::Value(e) => show(e),
            Output::Assigned(v) => {
                v.iter().map(|(n, e)| format!("{} =\n{}", n, show(e))).collect::<Vec<_>>().join("\n")
            }
            Output::Text(s) => s.clone(),
            Output::Nothing | Output::Exit => String::new(),
        }
    }
}

pub struct Session {
    pub(crate) vars: BTreeMap<String, Expr>,
    /// names that evaluate to themselves for now (plot variables, coordinates)
    pub(crate) hidden: Vec<String>,
    /// user-defined functions: name -> (parameters, body)
    pub(crate) funcs: BTreeMap<String, (Vec<String>, Node)>,
    /// how deeply user functions are calling each other
    pub(crate) depth: usize,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

fn builtins() -> BTreeMap<String, Expr> {
    let mut v = BTreeMap::new();
    v.insert("pi".into(), Expr::Const(Const::Pi));
    v.insert("e".into(), Expr::Const(Const::E));
    v.insert("i".into(), Expr::Const(Const::I));
    v.insert("j".into(), Expr::Const(Const::I));
    v.insert("true".into(), Expr::Bool(true));
    v.insert("false".into(), Expr::Bool(false));
    v
}

impl Session {
    pub fn new() -> Session {
        Session { vars: builtins(), hidden: vec![], funcs: BTreeMap::new(), depth: 0 }
    }

    /// names the user defined
    pub fn variable_names(&self) -> Vec<String> {
        let b = builtins();
        self.vars.iter().filter(|(k, v)| b.get(*k) != Some(v)).map(|(k, _)| k.clone()).collect()
    }

    pub fn get(&self, name: &str) -> Option<&Expr> {
        self.vars.get(name)
    }
    pub fn set(&mut self, name: &str, v: Expr) {
        self.vars.insert(name.to_string(), v);
    }

    /// Run source text; one output (or error) per statement
    pub fn run(&mut self, src: &str) -> Vec<R<Output>> {
        match parse(src) {
            Err(e) => vec![Err(format!("parse error: {}", e))],
            Ok(stmts) => {
                let mut out = vec![];
                for s in stmts {
                    let r = self.exec(&s);
                    let stop = matches!(r, Ok(Output::Exit));
                    out.push(r);
                    if stop {
                        break;
                    }
                }
                out
            }
        }
    }

    /// Run source text; each result comes with whether it should be hidden
    /// (the statement ended in `;`). Stops at the first error.
    pub fn run_quiet(&mut self, src: &str) -> Vec<(R<Output>, bool)> {
        match crate::parser::parse_quiet(src) {
            Err(e) => vec![(Err(format!("parse error: {}", e)), false)],
            Ok(stmts) => {
                let mut out = vec![];
                for (s, quiet) in stmts {
                    let r = self.exec(&s);
                    let stop = matches!(r, Ok(Output::Exit) | Err(_));
                    out.push((r, quiet));
                    if stop {
                        break;
                    }
                }
                out
            }
        }
    }

    /// Run source text and render it like the REPL does
    pub fn run_to_string(&mut self, src: &str) -> String {
        self.run(src)
            .into_iter()
            .map(|r| match r {
                Ok(o) => o.render(),
                Err(e) => format!("Error: {}", e),
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn exec(&mut self, s: &Stmt) -> R<Output> {
        match s {
            Stmt::Expr(n) => {
                let v = self.eval(n)?;
                self.set("ans", v.clone());
                Ok(Output::Value(v))
            }
            Stmt::Assign(name, n) => {
                let v = self.eval(n)?;
                self.set(name, v.clone());
                Ok(Output::Value(v))
            }
            Stmt::MultiAssign(names, n) => {
                let vals = match n {
                    Node::Call(f, args) if f == "lu" && args.len() == 1 => {
                        let m = self.matrix_arg(&args[0], "lu")?;
                        let (l, u, p) = linalg::lu(&m)?;
                        match names.len() {
                            // two outputs: A = L*U with L = P^T * L
                            2 => vec![Expr::Matrix(linalg::matmul(&linalg::transpose(&p), &l)?), Expr::Matrix(u)],
                            3 => vec![Expr::Matrix(l), Expr::Matrix(u), Expr::Matrix(p)],
                            _ => return Err("lu returns 2 or 3 values".into()),
                        }
                    }
                    Node::Call(f, args) if f == "eig" && args.len() == 1 && names.len() == 2 => {
                        let m = self.matrix_arg(&args[0], "eig")?;
                        let (v, vals) = crate::cas_cmd::eigvec(&m)?;
                        let n = vals.len();
                        let d: Vec<Vec<Expr>> = (0..n).map(|i| (0..n).map(|j| if i == j { vals[i].clone() } else { Expr::zero() }).collect()).collect();
                        vec![Expr::Matrix(v), Expr::Matrix(d)]
                    }
                    _ => return Err("only lu(...) and eig(...) return several values".into()),
                };
                let pairs: Vec<(String, Expr)> = names.iter().cloned().zip(vals).collect();
                for (n, v) in &pairs {
                    self.set(n, v.clone());
                }
                Ok(Output::Assigned(pairs))
            }
            Stmt::Command(c, arg) => self.command(c, arg.as_deref()),
            Stmt::Define(name, params, body) => {
                if Func::from_name(name).is_some() || ["exp", "sqrt"].contains(&name.as_str()) {
                    return Err(format!("{} is a built-in function; pick another name", name));
                }
                // `ans` means the last result at the time of the definition, not when
                // the function is called: d(x) := ans after deriv(f(x), x) is the derivative
                let body = match self.vars.get("ans").cloned() {
                    Some(v) if !params.iter().any(|p| p == "ans") => replace_ident(body, "ans", &expr_node(&v)?),
                    _ => body.clone(),
                };
                let text = format!("{}({}) := {}", name, params.join(", "), crate::parser::source(&body));
                self.funcs.insert(name.clone(), (params.clone(), body));
                Ok(Output::Text(text))
            }
        }
    }

    fn command(&mut self, c: &str, arg: Option<&str>) -> R<Output> {
        match c {
            "who" | "whos" => {
                let b = builtins();
                let lines: Vec<String> = self
                    .vars
                    .iter()
                    .filter(|(k, v)| b.get(*k) != Some(v))
                    .map(|(k, v)| format!("{}:\t{}", k, type_name(v)))
                    .collect();
                let mut text = format!("Defined variables:\n{}", lines.join("\n"));
                if !self.funcs.is_empty() {
                    let fs: Vec<String> = self
                        .funcs
                        .iter()
                        .map(|(k, (p, b))| format!("{}({}) := {}", k, p.join(", "), crate::parser::source(b)))
                        .collect();
                    text = format!("{}\nDefined functions:\n{}", text, fs.join("\n"));
                }
                Ok(Output::Text(text))
            }
            "clear" => {
                self.vars = builtins();
                self.funcs.clear();
                Ok(Output::Nothing)
            }
            "exit" | "quit" => Ok(Output::Exit),
            "pwd" => Ok(Output::Text(std::env::current_dir().map_err(|e| e.to_string())?.display().to_string())),
            "ls" => {
                let mut names: Vec<String> = std::fs::read_dir(".")
                    .map_err(|e| e.to_string())?
                    .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
                    .collect();
                names.sort();
                Ok(Output::Text(names.join("\n")))
            }
            "cd" => {
                let dir = arg.ok_or("cd needs a directory")?;
                std::env::set_current_dir(dir).map_err(|e| format!("cd {}: {}", dir, e))?;
                Ok(Output::Nothing)
            }
            "load" => {
                let file = arg.ok_or("load needs a file name")?;
                let src = std::fs::read_to_string(file).map_err(|e| format!("load {}: {}", file, e))?;
                let mut last = Output::Nothing;
                for r in self.run(&src) {
                    last = r?;
                }
                Ok(last)
            }
            "help" => Ok(Output::Text(crate::help::help(arg))),
            _ => Err(format!("unknown command {}", c)),
        }
    }

    fn matrix_arg(&mut self, n: &Node, f: &str) -> R<Mat> {
        match self.eval(n)? {
            Expr::Matrix(m) => Ok(m),
            other => Err(format!("{} needs a matrix, got {}", f, show(&other))),
        }
    }

    pub fn eval(&mut self, n: &Node) -> R<Expr> {
        let v = match n {
            Node::Num(s) => Expr::Num(Number::parse(s).ok_or_else(|| format!("bad number {}", s))?),
            Node::Str(s) => Expr::Str(s.clone()),
            Node::Ident(name) => self.lookup(name),
            Node::Neg(a) => {
                let a = self.eval(a)?;
                arith(Op::Mul, Expr::int(-1), a)?
            }
            Node::Bin(op, a, b) => {
                let a = self.eval(a)?;
                // short-circuit boolean operators
                match op {
                    Op::And if !truthy(&a)? => return Ok(Expr::Bool(false)),
                    Op::Or if truthy(&a)? => return Ok(Expr::Bool(true)),
                    Op::And | Op::Or => return Ok(Expr::Bool(truthy(&self.eval(b)?)?)),
                    _ => {}
                }
                let b = self.eval(b)?;
                match op {
                    Op::Eq | Op::Ne | Op::Lt | Op::Gt | Op::Le | Op::Ge => compare(*op, a, b)?,
                    _ => arith(*op, a, b)?,
                }
            }
            Node::Range(a, step, b) => {
                let a = self.eval(a)?;
                let s = match step {
                    Some(s) => self.eval(s)?,
                    None => Expr::one(),
                };
                let b = self.eval(b)?;
                range(&a, &s, &b)?
            }
            Node::Matrix(rows) => {
                let mut out: Vec<Vec<Expr>> = vec![];
                for r in rows {
                    // a row may contain row vectors / matrices that are concatenated
                    let mut parts: Vec<Mat> = vec![];
                    for x in r {
                        match self.eval(x)? {
                            Expr::Matrix(m) => parts.push(m),
                            v if v.is_scalar() || matches!(v, Expr::Bool(_) | Expr::Str(_)) => parts.push(vec![vec![v]]),
                            v => return Err(format!("{} can't be a matrix element", show(&v))),
                        }
                    }
                    // empty blocks ([]) vanish, so [a, rest] works when rest is []
                    parts.retain(|p| !p.is_empty() && !p[0].is_empty());
                    let Some(h) = parts.first().map(|p| p.len()) else { continue };
                    if parts.iter().any(|p| p.len() != h) {
                        return Err("matrix blocks in a row must have the same height".into());
                    }
                    for k in 0..h {
                        out.push(parts.iter().flat_map(|p| p[k].clone()).collect());
                    }
                }
                if out.iter().any(|r| r.len() != out[0].len()) {
                    return Err("not all rows have the same size".into());
                }
                // [x] is just x
                if out.len() == 1 && out[0].len() == 1 {
                    out.pop().unwrap().pop().unwrap()
                } else {
                    Expr::Matrix(out)
                }
            }
            Node::Transpose(a) => match self.eval(a)? {
                Expr::Matrix(m) => Expr::Matrix(linalg::transpose(&m)),
                v => v,
            },
            Node::Index(a, idx) => {
                let target = self.eval(a)?;
                let mut ix = vec![];
                for i in idx {
                    let v = self.eval(i)?;
                    let v = if v.num().is_some_and(|n| !n.is_exact() && n.is_integer()) { Expr::int(v.num().unwrap().to_f64() as i128) } else { v };
                    ix.push(v.num().and_then(|n| n.as_int()).filter(|&k| k >= 0).ok_or("indices must be non-negative integers")? as usize);
                }
                let m = match target {
                    Expr::Matrix(m) => m,
                    Expr::Obj(o) => return crate::commands::index_obj(&o, &ix),
                    // a scalar is a 1x1 matrix: x[0] and x[0,0] are x
                    v if v.is_scalar() && ix.iter().all(|&k| k == 0) && ix.len() <= 2 => return Ok(v),
                    v => return Err(format!("{} can't be indexed", show(&v))),
                };
                let (r, c) = linalg::dims(&m);
                let oob = || "index out of bounds".to_string();
                match ix.as_slice() {
                    [i] if r == 1 => m[0].get(*i).cloned().ok_or_else(oob)?,
                    [i] if c == 1 => m.get(*i).map(|row| row[0].clone()).ok_or_else(oob)?,
                    [i] => Expr::Matrix(vec![m.get(*i).cloned().ok_or_else(oob)?]),
                    [i, j] => m.get(*i).and_then(|row| row.get(*j)).cloned().ok_or_else(oob)?,
                    _ => return Err("use one or two indices".into()),
                }
            }
            Node::Call(f, args) => self.call(f, args)?,
        };
        check(&v)?;
        Ok(v)
    }

    fn call(&mut self, f: &str, args: &[Node]) -> R<Expr> {
        let arity = |n: usize| -> R<()> {
            if args.len() == n { Ok(()) } else { Err(format!("{} takes {} argument{}", f, n, if n == 1 { "" } else { "s" })) }
        };
        if self.funcs.contains_key(f) {
            return self.call_user(f, args);
        }
        if let Some(Expr::Matrix(m)) = self.vars.get(f).filter(|_| !self.hidden.iter().any(|h| h == f)).cloned() {
            return self.index(f, &m, args);
        }
        if f.ends_with('\'') {
            return self.call_primed(f, args);
        }
        if f == "if" {
            if args.len() != 3 {
                return Err("usage: if(condition, value if true, value if false)".into());
            }
            let c = self.eval(&args[0])?;
            return match c {
                Expr::Bool(b) => self.eval(&args[if b { 1 } else { 2 }]),
                Expr::Num(n) => self.eval(&args[if n.is_zero() { 2 } else { 1 }]),
                other => Err(format!("if: can't tell whether {} is true or false", show(&other))),
            };
        }
        if let Some(func) = Func::from_name(f) {
            arity(1)?;
            let a = self.eval(&args[0])?;
            return elementwise(&a, &|x| build::func(func, x.clone()));
        }
        match f {
            "exp" | "sqrt" => {
                arity(1)?;
                let a = self.eval(&args[0])?;
                let g = if f == "exp" { build::exp } else { build::sqrt };
                elementwise(&a, &|x| g(x.clone()))
            }
            "deriv" | "diff" => {
                arity(2)?;
                let x = match &args[1] {
                    Node::Ident(n) => n.clone(),
                    _ => return Err("deriv(y, x): x must be a variable name".into()),
                };
                let y = self.eval_hidden(&args[0], std::slice::from_ref(&x))?;
                Ok(calc::deriv(&y, &x))
            }
            "simplify" | "expand" | "factor" | "eval" | "complexity" | "info" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(format!("{} takes 1 argument", f));
                }
                let a = self.eval(&args[0])?;
                Ok(match f {
                    "simplify" => calc::simplify(&a),
                    "expand" => calc::expand(&a),
                    "factor" => match a.num().and_then(|n| n.as_bigint()) {
                        // factor(360) = 2^3 * 3^2 * 5
                        Some(n) if n.bits() > 1 => Expr::Str(crate::extra_cmd::factored_text(&n).map_err(|e| format!("factor: {}", e))?),
                        _ if crate::extra_cmd::is_huge(&a) => return Err(crate::extra_cmd::not_whole(&a, "factor")),
                        _ => crate::cas::factor_poly(&a).unwrap_or_else(|| calc::factor(&a)),
                    },
                    "eval" => calc::evalf(&a),
                    "complexity" => Expr::int(a.size() as i128),
                    _ => Expr::Str(structure(&a)),
                })
            }
            "subs" => {
                arity(3)?;
                let e = self.eval(&args[0])?;
                let x = match &args[1] {
                    Node::Ident(n) => n.clone(),
                    _ => return Err("subs(expr, x, value): x must be a variable name".into()),
                };
                let v = self.eval(&args[2])?;
                Ok(calc::subst(&e, &x, &v))
            }
            "and" | "or" | "xor" => {
                arity(2)?;
                let a = truthy(&self.eval(&args[0])?)?;
                let b = truthy(&self.eval(&args[1])?)?;
                Ok(Expr::Bool(match f {
                    "and" => a && b,
                    "or" => a || b,
                    _ => a ^ b,
                }))
            }
            "not" => {
                arity(1)?;
                Ok(Expr::Bool(!truthy(&self.eval(&args[0])?)?))
            }
            "inv" => {
                arity(1)?;
                match self.eval(&args[0])? {
                    Expr::Matrix(m) => {
                        let inv = linalg::inverse(&m)?;
                        Ok(Expr::Matrix(inv.iter().map(|r| r.iter().map(crate::ratfun::tidy).collect()).collect()))
                    }
                    v if v.is_scalar() => arith(Op::Pow, v, Expr::int(-1)),
                    _ => Err("inv needs a matrix".into()),
                }
            }
            "det" => {
                arity(1)?;
                let m = self.matrix_arg(&args[0], "det")?;
                linalg::det(&m)
            }
            "transpose" => {
                arity(1)?;
                let m = self.matrix_arg(&args[0], "transpose")?;
                Ok(Expr::Matrix(linalg::transpose(&m)))
            }
            "lu" => Err("use [L,U] = lu(A) or [L,U,P] = lu(A)".into()),
            "norm" => {
                let m = self.matrix_arg(args.first().ok_or("norm takes a matrix")?, "norm")?;
                let p = match args.get(1) {
                    None => "2".to_string(),
                    Some(Node::Ident(s)) => s.clone(),
                    Some(Node::Str(s)) => s.clone(),
                    Some(n) => show(&self.eval(n)?),
                };
                // a vector's length, exact: norm([1 1]) = sqrt(2)
                let (r, c) = linalg::dims(&m);
                if (r == 1 || c == 1) && p == "2" {
                    let sq = m.iter().flatten().map(|x| build::pow(x.clone(), Expr::int(2))).collect();
                    return Ok(calc::simplify(&build::sqrt(calc::simplify(&build::add(sq)))));
                }
                linalg::norm(&m, &p)
            }
            "eye" | "ones" | "zeros" | "rand" => {
                if args.is_empty() {
                    return Ok(match f {
                        "zeros" => Expr::zero(),
                        "rand" => Expr::float(rand_f64()),
                        _ => Expr::one(),
                    });
                }
                let n = self.size_arg(&args[0])?;
                let m = match args.get(1) {
                    Some(a) => self.size_arg(a)?,
                    None => n,
                };
                Ok(Expr::Matrix(match f {
                    "eye" => linalg::identity(n, m),
                    "ones" => linalg::filled(n, m, Expr::one()),
                    "zeros" => linalg::filled(n, m, Expr::zero()),
                    _ => (0..n).map(|_| (0..m).map(|_| Expr::float(rand_f64())).collect()).collect(),
                }))
            }
            "linspace" | "logspace" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(format!("{}(a, b[, n])", f));
                }
                let a = self.eval(&args[0])?;
                let b = self.eval(&args[1])?;
                let n = match args.get(2) {
                    Some(x) => self.size_arg(x)?,
                    None => if f == "linspace" { 100 } else { 50 },
                };
                let (a, b) = (calc::evalf(&a), calc::evalf(&b));
                let (Some(a), Some(b)) = (a.num(), b.num()) else { return Err(format!("{} needs numbers", f)) };
                let (a, b) = (a.to_f64(), b.to_f64());
                let row: Vec<Expr> = (0..n)
                    .map(|k| {
                        let t = if n > 1 { a + (b - a) * k as f64 / (n - 1) as f64 } else { a };
                        Expr::float(if f == "linspace" { t } else { 10f64.powf(t) })
                    })
                    .collect();
                Ok(Expr::Matrix(vec![row]))
            }
            "max" | "min" => {
                let mut vals = vec![];
                for a in args {
                    match self.eval(a)? {
                        Expr::Matrix(m) => vals.extend(m.into_iter().flatten()),
                        v => vals.push(v),
                    }
                }
                let mut best: Option<(f64, Expr)> = None;
                for v in vals {
                    let x = calc::evalf(&v).num().ok_or_else(|| format!("{} needs numbers", f))?.to_f64();
                    if best.as_ref().is_none_or(|(b, _)| if f == "max" { x > *b } else { x < *b }) {
                        best = Some((x, v));
                    }
                }
                best.map(|b| b.1).ok_or_else(|| format!("{} needs arguments", f))
            }
            _ => self.command_call(f, args),
        }
    }

    /// f(a, b) for a user-defined f: the body evaluated with the parameters bound
    fn call_user(&mut self, f: &str, args: &[Node]) -> R<Expr> {
        let mut vals = vec![];
        for a in args {
            vals.push(self.eval(a)?);
        }
        if self.depth >= 2000 {
            return Err(format!("{}: too many nested calls (does the recursion stop?)", f));
        }
        self.depth += 1;
        let r = self.run_user(f.to_string(), vals);
        self.depth -= 1;
        r
    }

    /// A(k): entry k of a vector, or row k of a matrix; A(i, j): one entry. Counting from 1.
    fn index(&mut self, f: &str, m: &[Vec<Expr>], args: &[Node]) -> R<Expr> {
        let mut ks = vec![];
        for a in args {
            let v = self.eval(a)?;
            let k = v.num().filter(|n| n.is_exact()).and_then(|n| n.as_int()).ok_or_else(|| format!("{}(...): positions are whole numbers, got {}", f, show(&v)))?;
            ks.push(k);
        }
        let (rows, cols) = (m.len() as i128, m.first().map_or(0, |r| r.len()) as i128);
        let bad = |k: i128, n: i128| -> R<usize> {
            if k < 1 || k > n { Err(format!("{}: position {} is outside 1..{}", f, k, n)) } else { Ok(k as usize - 1) }
        };
        match ks[..] {
            [k] if rows == 1 => Ok(m[0][bad(k, cols)?].clone()),
            [k] if cols == 1 => Ok(m[bad(k, rows)?][0].clone()),
            [k] => Ok(Expr::Matrix(vec![m[bad(k, rows)?].clone()])),
            [i, j] => Ok(m[bad(i, rows)?][bad(j, cols)?].clone()),
            _ => Err(format!("{}(...) takes one or two positions", f)),
        }
    }

    /// Runs a user function. A call in tail position (the whole body, or a
    /// branch of an if that is the whole body) does not nest: the loop just
    /// rebinds the parameters, so f(n, k) := if(..., k, f(n, k + 1)) can run
    /// millions of steps.
    fn run_user(&mut self, mut f: String, mut vals: Vec<Expr>) -> R<Expr> {
        let mut steps: u64 = 0;
        loop {
            let (params, body) = self.funcs.get(&f).cloned().unwrap();
            if params.len() != vals.len() {
                return Err(format!("{} takes {} argument{}", f, params.len(), if params.len() == 1 { "" } else { "s" }));
            }
            let saved: Vec<Option<Expr>> = params.iter().map(|p| self.vars.get(p).cloned()).collect();
            let hidden = self.hidden.clone();
            self.hidden.retain(|h| !params.contains(h));
            for (p, v) in params.iter().zip(vals) {
                self.vars.insert(p.clone(), v);
            }
            let r = self.tail(&body);
            self.hidden = hidden;
            for (p, v) in params.iter().zip(saved) {
                match v {
                    Some(v) => self.vars.insert(p.clone(), v),
                    None => self.vars.remove(p),
                };
            }
            match r? {
                Tail::Value(e) => return Ok(e),
                Tail::Call(g, v) => {
                    steps += 1;
                    if steps > 10_000_000 {
                        return Err(format!("{}: still running after 10 million steps (does the recursion stop?)", g));
                    }
                    f = g;
                    vals = v;
                }
            }
        }
    }

    fn tail(&mut self, node: &Node) -> R<Tail> {
        if let Node::Call(name, args) = node {
            if self.funcs.contains_key(name) {
                let mut vals = vec![];
                for a in args {
                    vals.push(self.eval(a)?);
                }
                return Ok(Tail::Call(name.clone(), vals));
            }
            if name == "if" && args.len() == 3 {
                let c = self.eval(&args[0])?;
                let pick = match c {
                    Expr::Bool(b) => b,
                    Expr::Num(n) => !n.is_zero(),
                    other => return Err(format!("if: can't tell whether {} is true or false", show(&other))),
                };
                return self.tail(&args[if pick { 1 } else { 2 }]);
            }
        }
        Ok(Tail::Value(self.eval(node)?))
    }

    /// f'(x), y''(t): derivatives of a user function, or of an undefined one
    fn call_primed(&mut self, f: &str, args: &[Node]) -> R<Expr> {
        let base = f.trim_end_matches('\'');
        let order = f.len() - base.len();
        match self.funcs.get(base).cloned() {
            Some((params, body)) => {
                if params.len() != 1 || args.len() != 1 {
                    return Err(format!("{}: primes work for functions of one variable", f));
                }
                let p = &params[0];
                let mut d = self.eval_hidden(&body, std::slice::from_ref(p))?;
                for _ in 0..order {
                    d = calc::deriv(&d, p);
                }
                let v = self.eval(&args[0])?;
                Ok(calc::subst(&d, p, &v))
            }
            None => {
                let mut vals = vec![];
                for a in args {
                    vals.push(self.eval(a)?);
                }
                Ok(Expr::Apply(f.to_string(), vals))
            }
        }
    }

    /// Value of a variable; stored expressions pick up values assigned later
    /// (K = gausscurvature(t); R = 2; K uses R = 2)
    pub(crate) fn lookup(&self, name: &str) -> Expr {
        if self.hidden.iter().any(|h| h == name) {
            return Expr::Sym(name.to_string());
        }
        match self.vars.get(name) {
            None => Expr::Sym(name.to_string()),
            Some(v @ (Expr::Obj(_) | Expr::Bool(_) | Expr::Str(_))) => v.clone(),
            Some(v) => self.resolve(v, &mut vec![name.to_string()]),
        }
    }

    /// Substitute the values of bound symbols (but not hidden ones, nor the
    /// names in `stack`, which guards against x = x + 1 loops)
    pub(crate) fn resolve(&self, e: &Expr, stack: &mut Vec<String>) -> Expr {
        match e {
            Expr::Sym(s) => {
                if stack.contains(s) || self.hidden.contains(s) {
                    return e.clone();
                }
                match self.vars.get(s) {
                    Some(v) if v.is_scalar() && v != e => {
                        stack.push(s.clone());
                        let r = self.resolve(v, stack);
                        stack.pop();
                        r
                    }
                    _ => e.clone(),
                }
            }
            Expr::Num(_) | Expr::Const(_) | Expr::Bool(_) | Expr::Str(_) | Expr::Obj(_) => e.clone(),
            _ => {
                if e.symbols().iter().all(|s| !self.vars.contains_key(s) || self.hidden.contains(s) || stack.contains(s)) {
                    return e.clone();
                }
                e.map_children(&mut |c| self.resolve(c, stack))
            }
        }
    }

    /// Evaluate with some names temporarily unbound
    pub(crate) fn eval_hidden(&mut self, n: &Node, names: &[String]) -> R<Expr> {
        let k = self.hidden.len();
        self.hidden.extend(names.iter().cloned());
        let r = self.eval(n);
        self.hidden.truncate(k);
        r
    }

    fn size_arg(&mut self, n: &Node) -> R<usize> {
        let v = self.eval(n)?;
        v.num()
            .filter(|n| n.is_integer() && !n.is_negative())
            .map(|n| n.to_f64() as usize)
            .ok_or_else(|| format!("{} is not a valid size", show(&v)))
    }
}

fn type_name(v: &Expr) -> String {
    match v {
        Expr::Num(_) => "Scalar".into(),
        Expr::Bool(_) => "Boolean".into(),
        Expr::Matrix(m) => format!("Matrix:{}(r),{}(c)", m.len(), m.first().map_or(0, |r| r.len())),
        Expr::Sym(_) => "Variable".into(),
        Expr::Str(_) => "String".into(),
        Expr::Obj(o) => o.kind().into(),
        _ => "Expression".into(),
    }
}

thread_local!(static RAND_STATE: std::cell::Cell<u64> = const { std::cell::Cell::new(0) });

/// Seed the generator behind rand(); hosts without a clock (WebAssembly) call this
pub fn seed_random(seed: u64) {
    RAND_STATE.with(|s| s.set(seed | 1));
}

fn clock_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        0x9e37_79b9_7f4a_7c15
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64) | 1
    }
}

fn rand_f64() -> f64 {
    RAND_STATE.with(|s| {
        let mut x = s.get();
        if x == 0 {
            x = clock_seed();
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        (x >> 11) as f64 / (1u64 << 53) as f64
    })
}

fn truthy(e: &Expr) -> R<bool> {
    match e {
        Expr::Bool(b) => Ok(*b),
        Expr::Num(n) => Ok(!n.is_zero()),
        other => Err(format!("{} is not a boolean", show(other))),
    }
}

enum Tail {
    Value(Expr),
    Call(String, Vec<Expr>),
}

fn elementwise(a: &Expr, f: &dyn Fn(&Expr) -> Expr) -> R<Expr> {
    match a {
        Expr::Matrix(m) => Ok(Expr::Matrix(linalg::map(m, f))),
        v if v.is_scalar() => Ok(f(v)),
        v => Err(format!("can't apply a function to {}", show(v))),
    }
}

/// Arithmetic on scalars and matrices
pub fn arith(op: Op, a: Expr, b: Expr) -> R<Expr> {
    use Expr::Matrix as M;
    for v in [&a, &b] {
        if matches!(v, Expr::Bool(_) | Expr::Str(_)) {
            return Err(format!("can't do arithmetic with {}", show(v)));
        }
    }
    Ok(match (op, a, b) {
        (Op::Add, M(x), M(y)) => M(linalg::zip(&x, &y, "sum", |p, q| add2(p.clone(), q.clone()))?),
        (Op::Sub, M(x), M(y)) => M(linalg::zip(&x, &y, "difference", |p, q| sub(p.clone(), q.clone()))?),
        (Op::Add, M(x), s) | (Op::Add, s, M(x)) => M(linalg::map(&x, |p| add2(p.clone(), s.clone()))),
        (Op::Sub, M(x), s) => M(linalg::map(&x, |p| sub(p.clone(), s.clone()))),
        (Op::Sub, s, M(x)) => M(linalg::map(&x, |p| sub(s.clone(), p.clone()))),
        (Op::Mul, M(x), M(y)) => M(linalg::matmul(&x, &y)?),
        (Op::Mul, M(x), s) => M(linalg::map(&x, |p| mul2(p.clone(), s.clone()))),
        (Op::Mul, s, M(x)) => M(linalg::map(&x, |p| mul2(s.clone(), p.clone()))),
        (Op::Div, M(x), M(y)) => M(linalg::matmul(&x, &linalg::inverse(&y)?)?),
        (Op::Div, M(x), s) => M(linalg::map(&x, |p| div(p.clone(), s.clone()))),
        (Op::Div, _, M(_)) => return Err("can't divide by a matrix; use inv(A)".into()),
        (Op::Solve, M(x), M(y)) => M(linalg::solve(&x, &y)?),
        (Op::Solve, ..) => return Err("A\\b needs two matrices".into()),
        (Op::Pow, M(x), s) => {
            let n = s.num().and_then(|n| n.as_int()).ok_or("matrix powers must be integers")?;
            let base = if n < 0 { linalg::inverse(&x)? } else { x.clone() };
            let (r, c) = linalg::dims(&x);
            if r != c {
                return Err("only square matrices have powers".into());
            }
            let mut acc = linalg::identity(r, r);
            for _ in 0..n.abs() {
                acc = linalg::matmul(&acc, &base)?;
            }
            M(acc)
        }
        (Op::Pow, _, M(_)) => return Err("matrix exponents are not supported".into()),
        (Op::Add, x, y) => add2(x, y),
        (Op::Sub, x, y) => sub(x, y),
        (Op::Mul, x, y) => mul2(x, y),
        (Op::Div, x, y) => div(x, y),
        (Op::Pow, x, y) => pow(x, y),
        (Op::And | Op::Or | Op::Eq | Op::Ne | Op::Lt | Op::Gt | Op::Le | Op::Ge, ..) => {
            return Err("comparisons are not arithmetic".into());
        }
    })
}

/// a == b, a < b, ...: true or false when that can be decided; a symbolic
/// a == b stays an equation (for solve, dsolve and implicitplot)
pub fn compare(op: Op, a: Expr, b: Expr) -> R<Expr> {
    if let (Expr::Bool(_), _) | (_, Expr::Bool(_)) | (Expr::Str(_), _) | (_, Expr::Str(_)) = (&a, &b) {
        return match op {
            Op::Eq => Ok(Expr::Bool(a == b)),
            Op::Ne => Ok(Expr::Bool(a != b)),
            _ => Err("only numbers can be ordered".into()),
        };
    }
    if let (Expr::Matrix(_), _) | (_, Expr::Matrix(_)) = (&a, &b) {
        return match op {
            Op::Eq => Ok(Expr::Bool(a == b)),
            Op::Ne => Ok(Expr::Bool(a != b)),
            _ => Err("matrices can't be ordered".into()),
        };
    }
    let d = calc::simplify(&sub(a.clone(), b.clone()));
    let num = |e: &Expr| calc::evalf(e).num().map(|n| n.to_f64());
    match op {
        Op::Eq | Op::Ne => {
            let same = if d.is_zero() {
                Some(true)
            } else if d.symbols().is_empty() && !d.contains_sym("i") {
                num(&d).map(|v| v.abs() < 1e-12 * (1.0 + num(&a).unwrap_or(0.0).abs()))
            } else {
                None
            };
            match (same, op) {
                (Some(s), Op::Eq) => Ok(Expr::Bool(s)),
                (Some(s), _) => Ok(Expr::Bool(!s)),
                (None, Op::Eq) => Ok(Expr::Apply("==".into(), vec![a, b])),
                (None, _) => Err(format!("can't decide whether {} ~= {}", show(&a), show(&b))),
            }
        }
        _ => {
            let (Some(x), Some(y)) = (num(&a), num(&b)) else {
                return Err(format!("can't compare {} and {}: they must be numbers", show(&a), show(&b)));
            };
            Ok(Expr::Bool(match op {
                Op::Lt => x < y,
                Op::Gt => x > y,
                Op::Le => x <= y,
                _ => x >= y,
            }))
        }
    }
}

/// An equation a == b as the expression a - b (other expressions unchanged)
pub fn equation_lhs(e: &Expr) -> Expr {
    match e {
        Expr::Apply(f, v) if f == "==" && v.len() == 2 => sub(v[0].clone(), v[1].clone()),
        other => other.clone(),
    }
}

/// Reject division by an exact zero anywhere in the result
fn check(e: &Expr) -> R<()> {
    match e {
        Expr::Pow(b, p) if b.is_zero() && b.num().unwrap().is_exact() && p.num().is_some_and(|n| n.is_negative()) => {
            Err("division by zero".into())
        }
        Expr::Add(v) | Expr::Mul(v) => v.iter().try_for_each(check),
        Expr::Pow(a, b) => check(a).and(check(b)),
        Expr::Func(_, a) => check(a),
        Expr::Matrix(m) => m.iter().flatten().try_for_each(check),
        _ => Ok(()),
    }
}

/// an expression as source code again, so it can go into a function body
fn expr_node(v: &Expr) -> R<Node> {
    match v {
        Expr::Matrix(m) => Ok(Node::Matrix(m.iter().map(|r| r.iter().map(expr_node).collect::<R<_>>()).collect::<R<_>>()?)),
        Expr::Str(s) => Ok(Node::Str(s.clone())),
        e => match crate::parser::parse(&show(e)) {
            Ok(mut st) if st.len() == 1 => match st.pop() {
                Some(crate::parser::Stmt::Expr(n)) => Ok(n),
                _ => Err(format!("cannot use {} in a function body", show(e))),
            },
            _ => Err(format!("cannot use {} in a function body", show(e))),
        },
    }
}

/// body with every use of the name replaced by node
fn replace_ident(body: &Node, name: &str, node: &Node) -> Node {
    let r = |n: &Node| replace_ident(n, name, node);
    let rb = |n: &Box<Node>| Box::new(replace_ident(n, name, node));
    match body {
        Node::Ident(s) if s == name => node.clone(),
        Node::Neg(a) => Node::Neg(rb(a)),
        Node::Bin(op, a, b) => Node::Bin(*op, rb(a), rb(b)),
        Node::Range(a, s, b) => Node::Range(rb(a), s.as_ref().map(rb), rb(b)),
        Node::Call(f, args) => Node::Call(f.clone(), args.iter().map(r).collect()),
        Node::Index(a, args) => Node::Index(rb(a), args.iter().map(r).collect()),
        Node::Matrix(m) => Node::Matrix(m.iter().map(|row| row.iter().map(r).collect()).collect()),
        Node::Transpose(a) => Node::Transpose(rb(a)),
        other => other.clone(),
    }
}

/// a:step:b as a row vector; exact when the inputs are exact
fn range(a: &Expr, s: &Expr, b: &Expr) -> R<Expr> {
    let (Some(an), Some(sn), Some(bn)) = (a.num(), s.num(), b.num()) else {
        return Err("ranges need numbers".into());
    };
    if sn.is_zero() {
        return Err("range step can't be zero".into());
    }
    let count = (bn.sub(&an).to_f64() / sn.to_f64() + 1e-10).floor();
    if count < 0.0 {
        return Ok(Expr::Matrix(vec![vec![]]));
    }
    if count > 1e7 {
        return Err("range is too long".into());
    }
    // decimal steps like 0:0.1:0.5 give 0.3, not 0.30000000000000004
    let decimals = |n: &Number| -> Option<u32> {
        let s = format!("{}", n.to_f64());
        if s.contains('e') { None } else { Some(s.split('.').nth(1).map_or(0, |f| f.len()) as u32) }
    };
    let scale = match (an.is_exact() && sn.is_exact(), decimals(&an), decimals(&sn)) {
        (false, Some(x), Some(y)) if x.max(y) <= 12 => Some(10f64.powi(x.max(y) as i32)),
        _ => None,
    };
    let row = (0..=count as i128)
        .map(|k| match scale {
            Some(sc) => {
                let v = ((an.to_f64() * sc).round() + k as f64 * (sn.to_f64() * sc).round()) / sc;
                Expr::float(v)
            }
            None => Expr::Num(an.add(sn.mul(Number::int(k)))),
        })
        .collect();
    Ok(Expr::Matrix(vec![row]))
}

/// info(e): the expression tree
fn structure(e: &Expr) -> String {
    use Expr::*;
    let list = |v: &[Expr]| v.iter().map(structure).collect::<Vec<_>>().join(", ");
    match e {
        Num(n) => format!("Number({})", n),
        Sym(s) => format!("Symbol({})", s),
        Const(_) => format!("Constant({})", show(e)),
        Add(v) => format!("Add({})", list(v)),
        Mul(v) => format!("Mul({})", list(v)),
        Pow(a, b) => format!("Pow({}, {})", structure(a), structure(b)),
        Func(f, a) => format!("{}({})", f.name(), structure(a)),
        Matrix(m) => format!("Matrix({}x{})", m.len(), m.first().map_or(0, |r| r.len())),
        Bool(b) => format!("Bool({})", b),
        Str(s) => format!("String({})", s),
        Apply(f, v) => format!("Function {}({})", f, list(v)),
        Obj(o) => o.kind().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(s: &str) -> String {
        Session::new().run_to_string(s)
    }
    #[test]
    fn basics() {
        assert_eq!(run("1+10+u+1"), "u+12");
        assert_eq!(run("8*y*u*y*u*y^5*y*u*u^6*r*y*7*y"), "56*r*u^9*y^10");
        assert_eq!(run("4*va-5*va"), "-va");
        assert_eq!(run("1/3+2/6"), "2/3");
        assert_eq!(run("a=1;b=2;c=a+b"), "1\n2\n3");
        assert_eq!(run("j^3"), "-i");
        assert_eq!(run("log(e^5*e^4)"), "9");
        assert_eq!(run("1/0"), "Error: division by zero");
        assert_eq!(run("(1&&0)||true"), "true");
        assert_eq!(run("A=[4 5 6;6 5 4;7 8 7];norm(inv(A)*A-eye(3),1)").lines().last().unwrap(), "0");
        assert_eq!(run("deriv(x^3,x)"), "3*x^2");
        assert_eq!(run("0.1:0.1:0.3").split('\t').count(), 3);
        assert_eq!(run("1:3"), "1\t2\t3");
    }
    #[test]
    fn tail_calls_and_big_integers() {
        let out = run("count(n, k) := if(k == n, k, count(n, k + 1)); count(100000, 0)");
        assert_eq!(out.lines().last().unwrap(), "100000");
        assert_eq!(run("fact(n) := if(n == 0, 1, n*fact(n-1)); fact(20)").lines().last().unwrap(), "2432902008176640000");
        assert_eq!(run("gcd(2^60-1, 2^40-1)"), "1048575");
        assert_eq!(run("mod([3 6 9], 7)"), "3\t6\t2");
        assert_eq!(run("tf(n) := if(n == 1, [], [2, tf(n/2)]); tf(8)").lines().last().unwrap(), "2\t2\t2");
        assert_eq!(run("[1 true]"), "1\ttrue");
        assert_eq!(run("rows([1 2; 3 4; 5 6])"), "3");
        assert!(run("factor(10^300 + 1)").starts_with("Error: factor: found the factors 73, 137, "));
        assert!(run("isprime(2^(2^2281 - 1) - 1)").contains("too large to compute: it would have about 10^686 digits"));
        assert!(run("isprime(eval(2^(2^2281 - 1) - 1))").contains("decimal too large to represent"));
        assert!(run("factor(2^(10^9) + 1)").contains("about 301029996 digits"));
        assert!(run("mod(2^(10^7), 7)").contains("too large to compute"));
        let out = run("f(x) := 8 + 3*x^2 + 9*x; deriv(f(x), x); d(x) := ans; deriv(d(x), x); d(3)");
        assert_eq!(out.lines().last().unwrap(), "27");
        assert!(out.contains("d(x) := 6*x+9"));
        assert!(run("m = [1 2; 3 4]; m*2; g(k) := k*ans; g(2)").ends_with("4\t8\n12\t16"));
        assert_eq!(run("h(ans) := ans + 1; h(1)").lines().last().unwrap(), "2");
        assert_eq!(run("2^200"), "1606938044258990275541962092341162602522202993782792835301376");
        assert_eq!(run("mod(3^100, 7)"), "4");
        assert_eq!(run("isprime(2^521 - 1)"), "true");
        assert_eq!(run("factor(2^127 - 3)"), "5^2 * 213156431 * 31927947500766558008599290859");
        assert_eq!(run("decode(encode(\"a message longer than fifteen characters\"))"), "a message longer than fifteen characters");
        assert_eq!(run("floor(10^40/7)"), "1428571428571428571428571428571428571428");
        assert_eq!(run("1/(2^200) * 2^201"), "2");
        assert_eq!(run("170141183460469231731687303715884105728 - 1"), "170141183460469231731687303715884105727");
        assert_eq!(run("rank([1 2 3; 4 5 6; 7 8 9])"), "2");
        assert_eq!(run("null([1 2 3; 4 5 6; 7 8 9])"), "1\n-2\n1");
        assert_eq!(run("eigvec([2 1; 1 2])"), "-1\t1\n1\t1");
        assert_eq!(run("v = [3 5 7]; v(2)").lines().last().unwrap(), "5");
        assert_eq!(run("A = [1 2; 3 4]; A(2, 1)").lines().last().unwrap(), "3");
        assert_eq!(run("A = [1 2; 3 4]; A(2)").lines().last().unwrap(), "3\t4");
    }
}
