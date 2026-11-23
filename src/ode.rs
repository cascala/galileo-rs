//! Numerical integration of dy/dt = f(t, y) with the classical Runge-Kutta method.

pub const DEFAULT_STEPS: usize = 20000;
pub const MAX_PLOT_POINTS: usize = 1500;

/// Samples (t, y), including the start. Stops early when the solution blows up
/// or a single step suddenly jumps (a singularity such as r = 0 or a horizon).
pub fn rk4(
    f: &dyn Fn(f64, &[f64]) -> Vec<f64>,
    y0: &[f64],
    t0: f64,
    t1: f64,
    steps: usize,
) -> Vec<(f64, Vec<f64>)> {
    let n = y0.len();
    let h = (t1 - t0) / steps as f64;
    let mut out = vec![(t0, y0.to_vec())];
    let mut y = y0.to_vec();
    let mut t = t0;
    let axpy =
        |a: &[f64], s: f64, b: &[f64]| -> Vec<f64> { (0..n).map(|i| a[i] + s * b[i]).collect() };
    for _ in 0..steps {
        let k1 = f(t, &y);
        let k2 = f(t + h / 2.0, &axpy(&y, h / 2.0, &k1));
        let k3 = f(t + h / 2.0, &axpy(&y, h / 2.0, &k2));
        let k4 = f(t + h, &axpy(&y, h, &k3));
        let next: Vec<f64> = (0..n)
            .map(|j| y[j] + h / 6.0 * (k1[j] + 2.0 * k2[j] + 2.0 * k3[j] + k4[j]))
            .collect();
        if !next.iter().all(|v| v.is_finite() && v.abs() < 1e12) {
            break;
        }
        let jump = (0..n).map(|j| (next[j] - y[j]).abs()).fold(0.0, f64::max);
        let size = y.iter().map(|v| v.abs()).fold(1.0, f64::max);
        if jump >= 0.25 * size {
            break;
        }
        y = next;
        t += h;
        out.push((t, y.clone()));
    }
    out
}

/// Keep at most `max` evenly spaced samples (and always the last)
/// Thin a plotted path to about `max` points, spaced evenly along the drawn
/// curve (not in time), so fast loops stay smooth and slow stretches cheap.
pub fn thin_path(v: &[(f64, f64)], max: usize) -> Vec<(f64, f64)> {
    let ok: Vec<(f64, f64)> = v
        .iter()
        .copied()
        .filter(|p| p.0.is_finite() && p.1.is_finite())
        .collect();
    if v.len() <= max || ok.len() != v.len() {
        return thin(v, max);
    }
    let (x0, x1) = ok.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |a, p| {
        (a.0.min(p.0), a.1.max(p.0))
    });
    let (y0, y1) = ok.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |a, p| {
        (a.0.min(p.1), a.1.max(p.1))
    });
    let (sx, sy) = ((x1 - x0).max(1e-300), (y1 - y0).max(1e-300));
    let d = |a: (f64, f64), b: (f64, f64)| ((a.0 - b.0) / sx).hypot((a.1 - b.1) / sy);
    let total: f64 = v.windows(2).map(|w| d(w[0], w[1])).sum();
    let step = total / max as f64;
    let mut out = vec![v[0]];
    let mut acc = 0.0;
    for w in v.windows(2) {
        acc += d(w[0], w[1]);
        if acc >= step {
            out.push(w[1]);
            acc = 0.0;
        }
    }
    if out.last() != v.last() {
        out.push(v[v.len() - 1]);
    }
    out
}

pub fn thin<T: Clone>(v: &[T], max: usize) -> Vec<T> {
    if v.len() <= max {
        return v.to_vec();
    }
    let k = v.len().div_ceil(max);
    let mut out: Vec<T> = v.iter().step_by(k).cloned().collect();
    out.push(v[v.len() - 1].clone());
    out
}
