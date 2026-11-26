//! Commands for groups as values ([`crate::fgroup`]): constructors, subgroups,
//! cosets, quotients, conjugacy classes, Sylow subgroups.

use crate::eval::{R, Session};
use crate::expr::{Expr, Obj};
use crate::fgroup::{self as fg, Group, Perm};
use crate::parser::Node;
use std::rc::Rc;

fn gval(g: Group) -> Expr {
    Expr::Obj(Rc::new(Obj::Group(g)))
}

fn group_of(e: &Expr, f: &str) -> R<Group> {
    match e {
        Expr::Obj(o) => match &**o {
            Obj::Group(g) => Ok(g.clone()),
            _ => Err(format!(
                "{}: expected a group such as sym(4), got {}",
                f,
                o.kind()
            )),
        },
        _ => Err(format!(
            "{}: expected a group such as sym(4) or group([2, 1, 3], [2, 3, 1])",
            f
        )),
    }
}

fn perm_in(e: &Expr, n: usize, f: &str) -> R<Perm> {
    let p = crate::extra_cmd::perm_of(e, f)?;
    if p.len() > n {
        return Err(format!(
            "{}: {} moves more than {} points",
            f,
            crate::print::show(e),
            n
        ));
    }
    let mut seen = vec![false; p.len()];
    for &k in &p {
        if k == 0 || k > p.len() || seen[k - 1] {
            return Err(format!(
                "{}: {} is not a permutation",
                f,
                crate::print::show(e)
            ));
        }
        seen[k - 1] = true;
    }
    Ok((0..n)
        .map(|i| {
            if i < p.len() {
                (p[i] - 1) as u16
            } else {
                i as u16
            }
        })
        .collect())
}

fn perm_expr(p: &Perm) -> Vec<Expr> {
    p.iter().map(|&k| Expr::int(k as i128 + 1)).collect()
}

fn rows(ps: &[Perm]) -> Expr {
    Expr::Matrix(ps.iter().map(perm_expr).collect())
}

fn small(e: &Expr, f: &str, lo: i128, hi: i128) -> R<usize> {
    let n = crate::extra_cmd::int_of(e, f)?;
    if n < lo || n > hi {
        return Err(format!("{}: n from {} to {}", f, lo, hi));
    }
    Ok(n as usize)
}

impl Session {
    /// Some(result) if f is a group command and the arguments fit it
    pub(crate) fn algebra_call(&mut self, f: &str, args: &[Node]) -> Option<R<Expr>> {
        const NAMES: &[&str] = &[
            "sym",
            "alt",
            "cyclic",
            "dihedral",
            "units",
            "group",
            "order",
            "elements",
            "gens",
            "isabelian",
            "center",
            "subgroup",
            "cosets",
            "isnormal",
            "quotient",
            "classes",
            "derived",
            "issolvable",
            "issimple",
            "sylow",
            "orders",
            "dprod",
            "cayley",
            "kernel",
            "image",
            "ishom",
            "isomorphic",
            "abelianinv",
            "cayleygraph",
        ];
        if !NAMES.contains(&f) {
            return None;
        }
        let v = match args.iter().map(|a| self.eval(a)).collect::<R<Vec<Expr>>>() {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        let is_group = |e: &Expr| matches!(e, Expr::Obj(o) if matches!(&**o, Obj::Group(_)));
        // cayley of a list of permutations is the older command
        if f == "cayley" && !v.first().is_some_and(is_group) {
            return None;
        }
        Some(self.algebra(f, &v))
    }

    fn algebra(&mut self, f: &str, v: &[Expr]) -> R<Expr> {
        let one = |f: &str| -> R<Group> {
            match v {
                [g] => group_of(g, f),
                _ => Err(format!("usage: {}(G) for a group G", f)),
            }
        };
        match f {
            "sym" | "alt" | "cyclic" | "dihedral" | "units" => {
                let [n] = v else {
                    return Err(format!("usage: {}(n)", f));
                };
                let n = small(n, f, 1, 1000)?;
                let g = match f {
                    "sym" => fg::sym(n),
                    "alt" => fg::alt(n),
                    "cyclic" => fg::cyclic(n),
                    "dihedral" => fg::dihedral(n),
                    _ => fg::units(n),
                }?;
                self.note(format!("`{}`: a group of order {}.", g.name, g.order()));
                Ok(gval(g))
            }
            "group" => {
                let g =
                    match v {
                        [Expr::Str(s)] if s.trim_start().starts_with('<') => fg::presentation(s)?,
                        [Expr::Str(s)] => fg::named(s)?,
                        // the rows of generate(...)
                        [Expr::Matrix(m)] if m.len() > 1 => {
                            let n = m.iter().map(|r| r.len()).max().unwrap_or(1);
                            let ps = m
                                .iter()
                                .map(|r| perm_in(&Expr::Matrix(vec![r.clone()]), n, "group"))
                                .collect::<R<Vec<_>>>()?;
                            Group::from_gens(n, ps)?
                        }
                        [] => return Err(
                            "usage: group(p1, p2, ...) generated by permutations, or group(\"Q8\")"
                                .into(),
                        ),
                        ps => {
                            let n = ps
                                .iter()
                                .map(|p| crate::extra_cmd::perm_of(p, "group").map(|q| q.len()))
                                .collect::<R<Vec<_>>>()?
                                .into_iter()
                                .max()
                                .unwrap();
                            let ps = ps
                                .iter()
                                .map(|p| perm_in(p, n, "group"))
                                .collect::<R<Vec<_>>>()?;
                            Group::from_gens(n, ps)?
                        }
                    };
                self.note(format!("A group of order {}.", g.order()));
                Ok(gval(g))
            }
            "order" => Ok(Expr::int(one(f)?.order() as i128)),
            "elements" => {
                let g = one(f)?;
                if g.order() > 5040 {
                    return Err(format!(
                        "elements: {} has {} elements, too many to show",
                        g.name,
                        g.order()
                    ));
                }
                Ok(rows(g.elements()))
            }
            "gens" => Ok(rows(&one(f)?.gens)),
            "isabelian" => Ok(Expr::Bool(one(f)?.is_abelian())),
            "issolvable" => {
                let g = one(f)?;
                let s = g.derived_series()?;
                let ok = s.last().unwrap().order() == 1;
                self.note(format!(
                    "Derived series G ⊇ G' ⊇ G'' ⊇ …, orders {}: {}.",
                    s.iter()
                        .map(|h| h.order().to_string())
                        .collect::<Vec<_>>()
                        .join(" ⊇ "),
                    if ok {
                        "it reaches the trivial group, so G is solvable"
                    } else {
                        "it stops before the trivial group, so G is not solvable"
                    }
                ));
                Ok(Expr::Bool(ok))
            }
            "issimple" => Ok(Expr::Bool(one(f)?.is_simple()?)),
            "center" => {
                let g = one(f)?;
                let z = g.center()?;
                self.note(format!(
                    "The center (elements that commute with everything) has order {}.",
                    z.order()
                ));
                Ok(gval(z))
            }
            "derived" => {
                let g = one(f)?;
                let d = g.derived()?;
                self.note(format!(
                    "The commutator subgroup [G, G], generated by all a⁻¹b⁻¹ab, has order {} (index {}); G/[G, G] is the largest abelian quotient.",
                    d.order(),
                    g.order() / d.order()
                ));
                Ok(gval(d))
            }
            "subgroup" => {
                let [g, ps @ ..] = v else {
                    return Err("usage: subgroup(G, p1, p2, ...)".into());
                };
                let g = group_of(g, f)?;
                let ps = ps
                    .iter()
                    .map(|p| perm_in(p, g.degree, f))
                    .collect::<R<Vec<_>>>()?;
                if let Some(p) = ps.iter().find(|p| !g.contains(p)) {
                    return Err(format!(
                        "subgroup: {} is not in {}",
                        fg::show_perm(p),
                        g.name
                    ));
                }
                let h = g.sub(ps)?;
                self.note(format!(
                    "A subgroup of order {}; by Lagrange's theorem it divides {} = |G| (index {}).",
                    h.order(),
                    g.order(),
                    g.order() / h.order()
                ));
                Ok(gval(h))
            }
            "cosets" | "isnormal" | "quotient" => {
                let [a, b] = v else {
                    return Err(format!("usage: {}(G, H) for a subgroup H of G", f));
                };
                let (mut g, mut h) = (group_of(a, f)?, group_of(b, f)?);
                if f == "isnormal" && g.is_subgroup_of(&h) && !h.is_subgroup_of(&g) {
                    std::mem::swap(&mut g, &mut h);
                }
                if !h.is_subgroup_of(&g) {
                    return Err(format!("{}: {} is not a subgroup of {}", f, h.name, g.name));
                }
                match f {
                    "cosets" => {
                        let c = g.cosets(&h);
                        self.note(format!(
                            "{} left cosets gH of {} elements each: {} = {} · {} (Lagrange). One representative per row.",
                            c.len(),
                            h.order(),
                            g.order(),
                            c.len(),
                            h.order()
                        ));
                        Ok(rows(&c.iter().map(|c| c[0].clone()).collect::<Vec<_>>()))
                    }
                    "isnormal" => Ok(Expr::Bool(g.is_normal(&h))),
                    _ => {
                        if !g.is_normal(&h) {
                            return Err(format!(
                                "quotient: {} is not normal in {} (g H g⁻¹ ≠ H for some g), so the cosets do not form a group",
                                h.name, g.name
                            ));
                        }
                        let q = g.quotient(&h)?;
                        self.note(format!(
                            "G/N has order {} / {} = {}; its elements are the cosets of N, acting here as permutations of the cosets.",
                            g.order(),
                            h.order(),
                            q.order()
                        ));
                        Ok(gval(q))
                    }
                }
            }
            "classes" => {
                let g = one(f)?;
                let cls = g.classes();
                let sizes: Vec<String> = cls.iter().map(|c| c.len().to_string()).collect();
                self.note(format!(
                    "{} conjugacy classes (one representative per row, in cycle notation {}). Class equation: {} = {}.",
                    cls.len(),
                    cls.iter().map(|c| fg::cycles(&c[0])).collect::<Vec<_>>().join(", "),
                    g.order(),
                    sizes.join(" + ")
                ));
                Ok(rows(&cls.iter().map(|c| c[0].clone()).collect::<Vec<_>>()))
            }
            "orders" => {
                let g = one(f)?;
                Ok(Expr::Matrix(
                    g.order_counts()
                        .into_iter()
                        .map(|(o, c)| vec![Expr::int(o as i128), Expr::int(c as i128)])
                        .collect(),
                ))
            }
            "sylow" => {
                let [a, p] = v else {
                    return Err("usage: sylow(G, p) for a prime p".into());
                };
                let g = group_of(a, f)?;
                let p = small(p, f, 2, 1_000_000)?;
                if !crate::numtheory::is_prime(&num_bigint::BigInt::from(p)) {
                    return Err(format!("sylow: {} is not prime", p));
                }
                let s = g.sylow(p)?;
                let m = g.order() / s.order();
                let np: Vec<String> = (1..=m)
                    .filter(|d| m % d == 0 && d % p == 1 % p)
                    .map(|d| d.to_string())
                    .collect();
                self.note(format!(
                    "A Sylow {}-subgroup, of order {} (the largest power of {} dividing {}). Sylow's theorems: they are all conjugate, and their number divides {} and is 1 mod {}, so it is one of {}.",
                    p, s.order(), p, g.order(), m, p, np.join(", ")
                ));
                Ok(gval(s))
            }
            "dprod" => {
                let [a, b] = v else {
                    return Err("usage: dprod(G, H), the direct product".into());
                };
                Ok(gval(group_of(a, f)?.product(&group_of(b, f)?)?))
            }
            "cayley" => {
                let g = one(f)?;
                if g.order() > 200 {
                    return Err(format!(
                        "cayley: {} has {} elements, too many for a table",
                        g.name,
                        g.order()
                    ));
                }
                self.note("Entry (i, j) is the number of the element e_i·e_j, numbering as in elements(G).");
                Ok(Expr::Matrix(
                    g.table()
                        .into_iter()
                        .map(|r| r.into_iter().map(|k| Expr::int(k as i128)).collect())
                        .collect(),
                ))
            }
            "kernel" | "image" | "ishom" => {
                let [a, b, imgs @ ..] = v else {
                    return Err(format!(
                        "usage: {}(G, H, f1, f2, ...) where f_i is the image in H of the i-th generator of G (see gens(G))",
                        f
                    ));
                };
                let (g, h) = (group_of(a, f)?, group_of(b, f)?);
                let imgs = imgs
                    .iter()
                    .map(|p| perm_in(p, h.degree, f))
                    .collect::<R<Vec<_>>>()?;
                if let Some(p) = imgs.iter().find(|p| !h.contains(p)) {
                    return Err(format!("{}: {} is not in {}", f, fg::show_perm(p), h.name));
                }
                match f {
                    "ishom" => Ok(Expr::Bool(fg::hom_table(&g, &h, &imgs).is_ok())),
                    "kernel" => {
                        let k = fg::kernel(&g, &h, &imgs)?;
                        let im = fg::image(&g, &h, &imgs)?;
                        self.note(format!(
                            "The kernel (what goes to the identity) has order {}, the image order {}: |G| = {} = {} · {} (first isomorphism theorem, G/ker ≅ image).",
                            k.order(), im.order(), g.order(), k.order(), im.order()
                        ));
                        Ok(gval(k))
                    }
                    _ => Ok(gval(fg::image(&g, &h, &imgs)?)),
                }
            }
            "isomorphic" => {
                let [a, b] = v else {
                    return Err("usage: isomorphic(G, H)".into());
                };
                let (g, h) = (group_of(a, f)?, group_of(b, f)?);
                if g.order() > 2000 || h.order() > 2000 {
                    return Err("isomorphic: groups up to order 2000".into());
                }
                match fg::isomorphism(&g, &h) {
                    Some((gs, imgs)) => {
                        self.note(format!(
                            "An isomorphism: {}.",
                            gs.iter()
                                .zip(&imgs)
                                .map(|(x, y)| format!(
                                    "{} ↦ {}",
                                    fg::show_perm(x),
                                    fg::show_perm(y)
                                ))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                        Ok(Expr::Bool(true))
                    }
                    None => {
                        self.note("No isomorphism: the orders, element orders or class sizes differ, or no assignment of generators works.");
                        Ok(Expr::Bool(false))
                    }
                }
            }
            "abelianinv" => {
                let g = one(f)?;
                let d = g.derived()?;
                let q = if d.order() == 1 {
                    g.clone()
                } else {
                    g.quotient(&d)?
                };
                let inv = fg::abelian_invariants(&q);
                let text = if inv.is_empty() {
                    "the trivial group".to_string()
                } else {
                    inv.iter()
                        .map(|d| format!("Z/{}", d))
                        .collect::<Vec<_>>()
                        .join(" × ")
                };
                self.note(format!(
                    "{} ≅ {}, with each number dividing the next (invariant factors){}.",
                    if d.order() == 1 { "G" } else { "G/[G, G]" },
                    text,
                    if d.order() == 1 {
                        ""
                    } else {
                        "; G is not abelian, so this is its largest abelian quotient"
                    }
                ));
                Ok(Expr::Matrix(vec![
                    inv.into_iter().map(|d| Expr::int(d as i128)).collect(),
                ]))
            }
            "cayleygraph" => {
                let g = one(f)?;
                if g.order() > 400 {
                    return Err(format!(
                        "cayleygraph: {} has {} elements, too many to draw",
                        g.name,
                        g.order()
                    ));
                }
                Ok(cayley_graph(&g))
            }
            _ => unreachable!(),
        }
    }
}

/// vertices on a circle, one colored set of arrows per generator
fn cayley_graph(g: &Group) -> Expr {
    use crate::plot::{Figure, Series};
    let n = g.order();
    // lay out the cosets x, s x, s^2 x, ... of the first generator s as polygons:
    // concentric for up to 3 of them, otherwise around a circle
    let els = g.elements();
    let idx = |p: &Perm| els.iter().position(|e| e == p).unwrap();
    let mut pos = vec![(0.0f64, 0.0f64); n];
    let mut placed = vec![false; n];
    let s0 = g.gens.first().cloned().unwrap_or_else(|| g.identity());
    let m = fg::order_of(&s0);
    let ncos = n / m;
    // breadth-first, so neighbouring cosets come out next to each other
    let mut order = vec![g.identity()];
    let mut seen = vec![false; n];
    seen[idx(&g.identity())] = true;
    let mut k = 0;
    while k < order.len() {
        for s in &g.gens {
            let y = fg::mul(s, &order[k]);
            let j = idx(&y);
            if !seen[j] {
                seen[j] = true;
                order.push(y);
            }
        }
        k += 1;
    }
    let tau = std::f64::consts::TAU;
    let mut c = 0;
    for x in &order {
        if placed[idx(x)] {
            continue;
        }
        let mut y = x.clone();
        for k in 0..m {
            let j = idx(&y);
            placed[j] = true;
            let t = tau * k as f64 / m as f64 + tau / 4.0;
            pos[j] = if ncos <= 3 {
                let r = 1.0 - 0.4 * c as f64;
                (r * t.cos(), r * t.sin())
            } else {
                let ct = tau * c as f64 / ncos as f64;
                let r = (0.8 * std::f64::consts::PI / ncos as f64).min(0.35);
                (ct.cos() + r * t.cos(), ct.sin() + r * t.sin())
            };
            y = fg::mul(&s0, &y);
        }
        c += 1;
    }
    let at = |i: usize| pos[i];
    let mut series = vec![];
    for s in &g.gens {
        let mut pts = vec![];
        for (i, x) in g.elements().iter().enumerate() {
            let y = fg::mul(s, x);
            let j = idx(&y);
            if i == j {
                continue;
            }
            let (a, b) = (at(i), at(j));
            pts.push(a);
            pts.push(b);
            // an arrowhead
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let l = (dx * dx + dy * dy).sqrt().max(1e-9);
            let (ux, uy) = (dx / l * 0.06, dy / l * 0.06);
            pts.push((b.0 - ux - uy * 0.6, b.1 - uy + ux * 0.6));
            pts.push(b);
            pts.push((b.0 - ux + uy * 0.6, b.1 - uy - ux * 0.6));
            pts.push((f64::NAN, f64::NAN));
        }
        series.push(Series {
            label: format!("× {}", fg::cycles(s)),
            points: pts,
            thin: true,
            ..Default::default()
        });
    }
    series.push(Series {
        label: "elements".into(),
        points: (0..n).map(at).collect(),
        markers: true,
        color: Some("#333333".into()),
        ..Default::default()
    });
    let mut fig = Figure::new(series, "", "", true);
    fig.title = format!("Cayley graph of {}", g.name);
    Expr::Obj(Rc::new(Obj::Figure(fig)))
}
