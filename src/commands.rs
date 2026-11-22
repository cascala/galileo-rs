//! Built-in commands beyond algebra: plotting, differential geometry and ODEs.

use crate::eval::{R, Session};
use crate::expr::{Expr, Obj};
use crate::geometry::{self, Metric};
use crate::numeric::{self, compile, run};
use crate::ode;
use crate::parser::Node;
use crate::plot::{self, Curve, Figure, Figure3D, Series, Surface};
use crate::print::show;
use std::rc::Rc;

fn obj(o: Obj) -> Expr {
    Expr::Obj(Rc::new(o))
}

/// x[i, j, ...] on a figure, metric or set of components
pub fn index_obj(o: &Obj, ix: &[usize]) -> R<Expr> {
    match o {
        Obj::Components(c) => c.get(ix),
        Obj::Metric(m) => match ix {
            [i, j] => m.g.get(*i).and_then(|r| r.get(*j)).cloned().ok_or_else(|| "index out of bounds".into()),
            _ => Err("a metric needs two indices, e.g. g[0,0]".into()),
        },
        Obj::Equations(e) => match ix {
            [i] => e.eqs.get(*i).map(|x| x.1.clone()).ok_or_else(|| "index out of bounds".into()),
            _ => Err("use one index to pick an equation".into()),
        },
        _ => Err(format!("a {} can't be indexed", o.kind())),
    }
}

/// Names whose unknown calls are undefined functions: a(t), A(r), f(x), f2(x), a_1(t)
fn function_name(f: &str) -> bool {
    let mut c = f.chars();
    let first = c.next().is_some_and(|x| x.is_alphabetic());
    let rest: String = c.collect();
    first && (rest.is_empty() || rest.starts_with('_') || rest.chars().all(|x| x.is_ascii_digit()))
}

fn ident(n: &Node, what: &str) -> R<String> {
    match n {
        Node::Ident(s) => Ok(s.clone()),
        _ => Err(format!("{} must be a variable name", what)),
    }
}

/// [r phi] or [r, phi]: a list of names, taken literally (not evaluated)
fn names(n: &Node, what: &str) -> R<Vec<String>> {
    match n {
        Node::Matrix(rows) => rows.iter().flatten().map(|x| ident(x, what)).collect(),
        Node::Ident(s) => Ok(vec![s.clone()]),
        _ => Err(format!("{} must be a list of names such as [r phi]", what)),
    }
}

fn flat(e: Expr) -> Vec<Expr> {
    match e {
        Expr::Matrix(m) => m.into_iter().flatten().collect(),
        other => vec![other],
    }
}

/// plot labels are the expressions as typed
fn label(n: &Node, _v: &Expr) -> String {
    crate::parser::source(n)
}

fn give_values(free: &[String]) -> String {
    format!("give values to {} first, e.g. {} = 1", free.join(", "), free[0])
}

/// A labelled solution: (label, [(t, state)])
type Trajectory = (String, Vec<(f64, Vec<f64>)>);

impl Session {
    pub(crate) fn command_call(&mut self, f: &str, args: &[Node]) -> R<Expr> {
        if let Some(r) = self.cas_call(f, args) {
            return r;
        }
        match f {
            "plot" => self.plot(args),
            "parametric" => self.parametric(args),
            "plot3d" => self.plot3d(args),
            "surface" => self.surface(args),
            "parametric3d" => self.parametric3d(args),
            "show" => self.show(args),
            "metric" => self.metric(args),
            "metric.generate" => {
                let name = match args {
                    [Node::Str(s)] | [Node::Ident(s)] => s.clone(),
                    _ => return Err("usage: metric.generate(name)".into()),
                };
                geometry::template(&name).map(|m| obj(Obj::Metric(m))).ok_or_else(|| {
                    format!("unknown metric {}; known: {}", name, geometry::TEMPLATES.join(", "))
                })
            }
            "inducedmetric" => {
                if args.len() != 2 {
                    return Err("usage: inducedmetric([x(u,v) y(u,v) z(u,v)], [u v])".into());
                }
                let coords = names(&args[1], "the coordinates")?;
                let emb = flat(self.eval_hidden(&args[0], &coords)?);
                if emb.iter().any(|e| !e.is_scalar()) {
                    return Err("inducedmetric: the embedding must be a row of expressions".into());
                }
                Ok(obj(Obj::Metric(geometry::induced(&emb, &coords))))
            }
            "odeplot" | "odesolve" => self.ode(f == "odeplot", args),
            "geodesic" => self.geodesic(args),
            "atan2" => {
                if args.len() != 2 {
                    return Err("atan2 takes 2 arguments".into());
                }
                let y = self.eval(&args[0])?;
                let x = self.eval(&args[1])?;
                if let (Some(a), Some(b)) = (y.num(), x.num()) {
                    if !a.is_exact() || !b.is_exact() {
                        return Ok(Expr::float(a.to_f64().atan2(b.to_f64())));
                    }
                }
                Ok(Expr::Apply("atan2".into(), vec![y, x]))
            }
            _ if geometry::NAMED.contains(&f) => {
                if args.len() != 1 {
                    return Err(format!("{} takes a metric", f));
                }
                let m = self.metric_arg(&args[0], f)?;
                geometry::named(f, &m).map_err(|e| format!("{}: {}", f, e))
            }
            _ if function_name(f) && !self.vars.contains_key(f) => {
                let mut vals = vec![];
                for a in args {
                    let v = self.eval(a)?;
                    if !v.is_scalar() {
                        return Err(format!("{}: arguments must be expressions", f));
                    }
                    vals.push(v);
                }
                Ok(Expr::Apply(f.to_string(), vals))
            }
            _ => Err(format!("unknown function {}", f)),
        }
    }

    fn metric_arg(&mut self, n: &Node, f: &str) -> R<Metric> {
        match self.eval(n)? {
            Expr::Obj(o) => match &*o {
                Obj::Metric(m) => Ok(m.clone()),
                other => Err(format!("{} needs a metric, got a {}", f, other.kind())),
            },
            other => Err(format!(
                "{} needs a metric (e.g. g = metric([r phi], [1 0; 0 r^2])), got {}",
                f,
                show(&other)
            )),
        }
    }

    /// A number, or an error naming the argument
    fn number(&mut self, n: &Node, what: &str) -> R<f64> {
        let v = self.eval(n)?;
        let x = numeric::value(&v);
        if x.is_finite() { Ok(x) } else { Err(format!("{} must be a number, got {}", what, show(&v))) }
    }

    fn numbers(&mut self, n: &Node, what: &str) -> R<Vec<f64>> {
        let v = self.eval(n)?;
        let xs: Vec<f64> = flat(v.clone()).iter().map(numeric::value).collect();
        if xs.iter().all(|x| x.is_finite()) { Ok(xs) } else { Err(format!("{} must be numbers, got {}", what, show(&v))) }
    }

    fn range(&mut self, cmd: &str, a: &Node, b: &Node) -> R<(f64, f64)> {
        let (x, y) = (self.number(a, "the range")?, self.number(b, "the range")?);
        if x == y {
            return Err(format!("{}: the range must be two different numbers", cmd));
        }
        Ok((x, y))
    }

    /// Evaluate `n` with `vars` unbound and compile it as a function of them
    fn function(&mut self, n: &Node, vars: &[String]) -> R<(numeric::Code, Expr)> {
        let v = self.eval_hidden(n, vars)?;
        if !v.is_scalar() {
            return Err(format!("can't plot {}", show(&v)));
        }
        let names: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
        Ok((compile(&v, &names), v))
    }

    // ---------- plots ----------

    fn plot(&mut self, args: &[Node]) -> R<Expr> {
        let usage = "usage: plot(f, x) or plot(f1, f2, ..., x, from, to)";
        let k = args.len();
        // plot(f..., x, from, to) when the last two are numbers, else plot(f..., x) on -10..10
        let explicit = k >= 4 && matches!(args[k - 3], Node::Ident(_)) && {
            let a = self.eval(&args[k - 2]).map(|v| numeric::value(&v));
            let b = self.eval(&args[k - 1]).map(|v| numeric::value(&v));
            matches!((a, b), (Ok(a), Ok(b)) if a.is_finite() && b.is_finite())
        };
        let (fs, var, a, b) = if explicit {
            let (a, b) = self.range("plot", &args[k - 2], &args[k - 1])?;
            (&args[..k - 3], ident(&args[k - 3], "the plot variable")?, a, b)
        } else if k >= 2 && matches!(args[k - 1], Node::Ident(_)) {
            (&args[..k - 1], ident(&args[k - 1], "the plot variable")?, -10.0, 10.0)
        } else {
            return Err(usage.into());
        };
        let xs = plot::steps(a.min(b), a.max(b), plot::SAMPLES);
        let mut series = vec![];
        for n in fs {
            let (code, v) = self.function(n, std::slice::from_ref(&var))?;
            series.push(Series { label: label(n, &v), points: xs.iter().map(|&x| (x, run(&code, &[x]))).collect() });
        }
        Ok(obj(Obj::Figure(Figure::new(series, &var, "", false))))
    }

    fn parametric(&mut self, args: &[Node]) -> R<Expr> {
        let [fx, fy, t, a, b] = args else { return Err("usage: parametric(fx, fy, t, from, to)".into()) };
        let t = ident(t, "the parameter")?;
        let (a, b) = self.range("parametric", a, b)?;
        let (gx, vx) = self.function(fx, std::slice::from_ref(&t))?;
        let (gy, vy) = self.function(fy, std::slice::from_ref(&t))?;
        let pts = plot::steps(a, b, plot::SAMPLES).iter().map(|&s| (run(&gx, &[s]), run(&gy, &[s]))).collect();
        let s = Series { label: format!("({}, {})", label(fx, &vx), label(fy, &vy)), points: pts };
        Ok(obj(Obj::Figure(Figure::new(vec![s], "", "", true))))
    }

    fn grid(&mut self, f: &Node, u: &str, v: &str, us: &[f64], vs: &[f64]) -> R<(Vec<Vec<f64>>, Expr)> {
        let (g, e) = self.function(f, &[u.to_string(), v.to_string()])?;
        Ok((us.iter().map(|&a| vs.iter().map(|&b| run(&g, &[a, b])).collect()).collect(), e))
    }

    fn plot3d(&mut self, args: &[Node]) -> R<Expr> {
        let [f, x, a, b, y, c, d] = args else { return Err("usage: plot3d(f, x, from, to, y, from, to)".into()) };
        let (x, y) = (ident(x, "x")?, ident(y, "y")?);
        let (x0, x1) = self.range("plot3d", a, b)?;
        let (y0, y1) = self.range("plot3d", c, d)?;
        let xs = plot::steps(x0, x1, plot::GRID - 1);
        let ys = plot::steps(y0, y1, plot::GRID - 1);
        let (gz, e) = self.grid(f, &x, &y, &xs, &ys)?;
        let gx = xs.iter().map(|&a| ys.iter().map(|_| a).collect()).collect();
        let gy = xs.iter().map(|_| ys.clone()).collect();
        let s = Surface { label: label(f, &e), x: gx, y: gy, z: gz };
        Ok(obj(Obj::Figure3D(Figure3D::new(vec![s], vec![], [&x, &y, "z"], false))))
    }

    fn surface(&mut self, args: &[Node]) -> R<Expr> {
        let [fx, fy, fz, u, a, b, v, c, d] = args else {
            return Err("usage: surface(fx, fy, fz, u, from, to, v, from, to)".into());
        };
        let (u, v) = (ident(u, "u")?, ident(v, "v")?);
        let (u0, u1) = self.range("surface", a, b)?;
        let (v0, v1) = self.range("surface", c, d)?;
        let us = plot::steps(u0, u1, plot::GRID - 1);
        let vs = plot::steps(v0, v1, plot::GRID - 1);
        let (x, ex) = self.grid(fx, &u, &v, &us, &vs)?;
        let (y, ey) = self.grid(fy, &u, &v, &us, &vs)?;
        let (z, ez) = self.grid(fz, &u, &v, &us, &vs)?;
        let s = Surface { label: format!("({}, {}, {})", label(fx, &ex), label(fy, &ey), label(fz, &ez)), x, y, z };
        Ok(obj(Obj::Figure3D(Figure3D::new(vec![s], vec![], ["x", "y", "z"], true))))
    }

    fn parametric3d(&mut self, args: &[Node]) -> R<Expr> {
        let [fx, fy, fz, t, a, b] = args else { return Err("usage: parametric3d(fx, fy, fz, t, from, to)".into()) };
        let t = ident(t, "the parameter")?;
        let (a, b) = self.range("parametric3d", a, b)?;
        let ts = std::slice::from_ref(&t);
        let (gx, ex) = self.function(fx, ts)?;
        let (gy, ey) = self.function(fy, ts)?;
        let (gz, ez) = self.function(fz, ts)?;
        let pts = plot::steps(a, b, plot::CURVE3D_SAMPLES).iter().map(|&s| (run(&gx, &[s]), run(&gy, &[s]), run(&gz, &[s]))).collect();
        let c = Curve { label: format!("({}, {}, {})", label(fx, &ex), label(fy, &ey), label(fz, &ez)), points: pts };
        Ok(obj(Obj::Figure3D(Figure3D::new(vec![], vec![c], ["x", "y", "z"], true))))
    }

    fn show(&mut self, args: &[Node]) -> R<Expr> {
        let mut figs = vec![];
        for a in args {
            match self.eval(a)? {
                Expr::Obj(o) if matches!(*o, Obj::Figure(_) | Obj::Figure3D(_)) => figs.push(o),
                other => return Err(format!("show: {} is not a plot", show(&other))),
            }
        }
        let Some(first) = figs.first().cloned() else { return Err("usage: show(figure1, figure2, ...)".into()) };
        match &*first {
            Obj::Figure(f) => {
                let mut acc = f.clone();
                for g in &figs[1..] {
                    match &**g {
                        Obj::Figure(g) => acc = acc.overlay(g),
                        _ => return Err("show: all arguments must be 2D plots, or all 3D plots".into()),
                    }
                }
                Ok(obj(Obj::Figure(acc)))
            }
            Obj::Figure3D(f) => {
                let mut acc = f.clone();
                for g in &figs[1..] {
                    match &**g {
                        Obj::Figure3D(g) => acc = acc.overlay(g),
                        _ => return Err("show: all arguments must be 2D plots, or all 3D plots".into()),
                    }
                }
                Ok(obj(Obj::Figure3D(acc)))
            }
            _ => unreachable!(),
        }
    }

    // ---------- geometry ----------

    fn metric(&mut self, args: &[Node]) -> R<Expr> {
        let [c, m] = args else { return Err("usage: metric([x1 x2 ...], [g11 g12 ...; g21 g22 ...])".into()) };
        let coords = names(c, "metric: the first argument")?;
        let g = match self.eval_hidden(m, &coords)? {
            Expr::Matrix(g) => g,
            v if coords.len() == 1 && v.is_scalar() => vec![vec![v]],
            _ => return Err("metric: the second argument must be a square matrix with one row per coordinate".into()),
        };
        Ok(obj(Obj::Metric(Metric::new(coords, g)?)))
    }

    /// Substitute session values into e, leaving the given names alone
    fn values(&mut self, e: &Expr, keep: &[String]) -> Expr {
        let k = self.hidden.len();
        self.hidden.extend(keep.iter().cloned());
        let r = self.resolve(e, &mut vec![]);
        self.hidden.truncate(k);
        r
    }

    /// map expressions for odeplot / geodesic: 2 entries give a 2D plot, 3 a 3D plot
    fn figure(
        &mut self,
        trajs: Vec<Trajectory>,
        names: &[String],
        map: Vec<Expr>,
        equal: bool,
    ) -> R<Expr> {
        let nm: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let fs: Vec<numeric::Code> = map.iter().map(|m| compile(m, &nm)).collect();
        let labels: Vec<String> = map.iter().map(show).collect();
        let at = |t: f64, y: &[f64]| -> Vec<f64> {
            let mut a = y.to_vec();
            a.push(t);
            a
        };
        match map.len() {
            2 => {
                let series = trajs
                    .into_iter()
                    .map(|(l, pts)| Series {
                        label: l,
                        points: ode::thin(&pts, ode::MAX_PLOT_POINTS)
                            .iter()
                            .map(|(t, y)| {
                                let a = at(*t, y);
                                (run(&fs[0], &a), run(&fs[1], &a))
                            })
                            .collect(),
                    })
                    .collect();
                Ok(obj(Obj::Figure(Figure::new(series, &labels[0], &labels[1], equal))))
            }
            3 => {
                let curves = trajs
                    .into_iter()
                    .map(|(l, pts)| Curve {
                        label: l,
                        points: ode::thin(&pts, ode::MAX_PLOT_POINTS)
                            .iter()
                            .map(|(t, y)| {
                                let a = at(*t, y);
                                (run(&fs[0], &a), run(&fs[1], &a), run(&fs[2], &a))
                            })
                            .collect(),
                    })
                    .collect();
                Ok(obj(Obj::Figure3D(Figure3D::new(vec![], curves, [&labels[0], &labels[1], &labels[2]], equal))))
            }
            _ => Err("the plot map must have 2 entries (2D) or 3 entries (3D), e.g. [x y] or [x y z]".into()),
        }
    }

    /// odeplot([f1 f2], [y1 y2], t, [y1_0 y2_0], t0, t1, [X Y] (, steps))
    /// odesolve([f1 f2], [y1 y2], t, [y1_0 y2_0], t0, t1 (, steps))
    fn ode(&mut self, plot: bool, args: &[Node]) -> R<Expr> {
        let usage = if plot {
            "usage: odeplot([f1 f2], [y1 y2], t, [y1_0 y2_0], t0, t1, [X Y]) where X, Y (and Z for 3D) are expressions in t, y1, y2"
        } else {
            "usage: odesolve([f1 f2], [y1 y2], t, [y1_0 y2_0], t0, t1)"
        };
        let (map, steps) = match (plot, args.len()) {
            (true, 7) => (Some(&args[6]), None),
            (true, 8) => (Some(&args[6]), Some(&args[7])),
            (false, 6) => (None, None),
            (false, 7) => (None, Some(&args[6])),
            _ => return Err(usage.into()),
        };
        let vars = names(&args[1], "the unknowns")?;
        let t = ident(&args[2], "the independent variable")?;
        let mut all = vars.clone();
        all.push(t.clone());
        let rhs = flat(self.eval_hidden(&args[0], &all)?);
        let y0 = self.numbers(&args[3], "the start values")?;
        if rhs.len() != vars.len() || y0.len() != vars.len() {
            return Err(format!("{}: give one equation and one start value per unknown", if plot { "odeplot" } else { "odesolve" }));
        }
        let t0 = self.number(&args[4], "t0")?;
        let t1 = self.number(&args[5], "t1")?;
        let steps = match steps {
            Some(s) => self.number(s, "the number of steps")?.max(1.0) as usize,
            None => ode::DEFAULT_STEPS,
        };
        let nm: Vec<&str> = all.iter().map(|s| s.as_str()).collect();
        let mut free = vec![];
        for r in &rhs {
            for s in numeric::free(r, &nm) {
                if !free.contains(&s) {
                    free.push(s);
                }
            }
        }
        if !free.is_empty() {
            return Err(give_values(&free));
        }
        let fs: Vec<numeric::Code> = rhs.iter().map(|r| compile(r, &nm)).collect();
        let n = vars.len();
        let f = |tt: f64, y: &[f64]| -> Vec<f64> {
            let mut a = y.to_vec();
            a.push(tt);
            fs.iter().map(|g| run(g, &a)).collect()
        };
        let traj = ode::rk4(&f, &y0, t0, t1, steps);
        match map {
            None => {
                let last = &traj[traj.len() - 1];
                if (last.0 - t1).abs() > 1e-9 * t1.abs().max(1.0) {
                    return Err(format!("odesolve: the solution breaks down at t = {}", last.0));
                }
                let row: Vec<Expr> = last.1.iter().map(|v| Expr::float(*v)).collect();
                Ok(if n == 1 { row[0].clone() } else { Expr::Matrix(vec![row]) })
            }
            Some(m) => {
                let ms = flat(self.eval_hidden(m, &all)?);
                let polar = ms.len() == 2 && show(&ms[0]).contains("cos(") && show(&ms[1]).contains("sin(");
                // name the curve after what is drawn vertically (or the whole map in 3D)
                let name = match ms.len() {
                    2 => show(&ms[1]),
                    _ => "solution".into(),
                };
                self.figure(vec![(name, traj)], &all, ms, polar)
            }
        }
    }

    /// geodesic(g, [x0 ...], [v0 ...] (one row per geodesic), s1 (, [X Y (Z)] (, steps)))
    fn geodesic(&mut self, args: &[Node]) -> R<Expr> {
        let usage = "usage: geodesic(g, [x0 y0], [vx vy], s1) or geodesic(g, x0, v0, s1, [X Y Z]) with X, Y, Z expressions in the coordinates";
        if args.len() < 4 || args.len() > 6 {
            return Err(usage.into());
        }
        let m = self.metric_arg(&args[0], "geodesic")?;
        let n = m.dim();
        let xs = m.coords.clone();
        let x0 = self.numbers(&args[1], "geodesic: the start point")?;
        if x0.len() != n {
            return Err(format!("geodesic: the start point needs {} numbers", n));
        }
        let v0s: Vec<Vec<f64>> = match self.eval(&args[2])? {
            Expr::Matrix(rows) if rows[0].len() == n => rows.iter().map(|r| r.iter().map(numeric::value).collect()).collect(),
            _ => return Err(format!("geodesic: the start velocity needs {} numbers (or one row of {} per geodesic)", n, n)),
        };
        if v0s.iter().flatten().any(|v| !v.is_finite()) {
            return Err("geodesic: the start velocity must be numbers".into());
        }
        let s1 = self.number(&args[3], "geodesic: the length s1")?;
        let steps = match args.get(5) {
            Some(s) => self.number(s, "the number of steps")?.max(1.0) as usize,
            None => ode::DEFAULT_STEPS / 4,
        };
        let cv = geometry::curvature(&m)?;
        let nm: Vec<&str> = xs.iter().map(|s| s.as_str()).collect();
        let mut terms = vec![];
        let mut free: Vec<String> = vec![];
        for a in 0..n {
            for b in 0..n {
                for c in b..n {
                    let g = cv.gamma(a, b, c).clone();
                    if g.is_zero() {
                        continue;
                    }
                    let g = self.values(&g, &xs);
                    for s in numeric::free(&g, &nm) {
                        if !free.contains(&s) {
                            free.push(s);
                        }
                    }
                    terms.push((a, b, c, compile(&g, &nm)));
                }
            }
        }
        if !free.is_empty() {
            return Err(format!("geodesic: {}", give_values(&free)));
        }
        let f = |_: f64, y: &[f64]| -> Vec<f64> {
            let mut out = y[n..2 * n].to_vec();
            out.resize(2 * n, 0.0);
            for (a, b, c, g) in &terms {
                let w = if b == c { 1.0 } else { 2.0 };
                out[n + a] -= w * run(g, &y[..n]) * y[n + b] * y[n + c];
            }
            out
        };
        let trajs: Vec<Trajectory> = v0s
            .iter()
            .enumerate()
            .map(|(i, v0)| {
                let mut y0 = x0.clone();
                y0.extend(v0);
                (format!("geodesic {}", i + 1), ode::rk4(&f, &y0, 0.0, s1, steps))
            })
            .collect();
        let mut all: Vec<String> = xs.clone();
        all.extend(xs.iter().map(|x| format!("{}'", x)));
        all.push("s".into());
        let map = match args.get(4) {
            Some(mp) => flat(self.eval_hidden(mp, &all)?),
            None if n == 2 || n == 3 => xs.iter().map(|x| Expr::sym(x)).collect(),
            None => {
                return Err(format!(
                    "geodesic: for a {}D metric, give a map to 2 or 3 plot coordinates, e.g. [r*cos(phi) r*sin(phi)]",
                    n
                ));
            }
        };
        let map: Vec<Expr> = map.iter().map(|e| self.values(e, &all)).collect();
        self.figure(trajs, &all, map, true)
    }
}
