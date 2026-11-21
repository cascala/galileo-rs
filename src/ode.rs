//! Numerical integration of dy/dt = f(t, y) with the classical Runge-Kutta method.

pub const DEFAULT_STEPS: usize = 20000;
pub const MAX_PLOT_POINTS: usize = 1500;

/// Samples (t, y), including the start. Stops early when the solution blows up
/// or a single step suddenly jumps (a singularity such as r = 0 or a horizon).
pub fn rk4(f: &dyn Fn(f64, &[f64]) -> Vec<f64>, y0: &[f64], t0: f64, t1: f64, steps: usize) -> Vec<(f64, Vec<f64>)> {
    let n = y0.len();
    let h = (t1 - t0) / steps as f64;
    let mut out = vec![(t0, y0.to_vec())];
    let mut y = y0.to_vec();
    let mut t = t0;
    let axpy = |a: &[f64], s: f64, b: &[f64]| -> Vec<f64> { (0..n).map(|i| a[i] + s * b[i]).collect() };
    for _ in 0..steps {
        let k1 = f(t, &y);
        let k2 = f(t + h / 2.0, &axpy(&y, h / 2.0, &k1));
        let k3 = f(t + h / 2.0, &axpy(&y, h / 2.0, &k2));
        let k4 = f(t + h, &axpy(&y, h, &k3));
        let next: Vec<f64> = (0..n).map(|j| y[j] + h / 6.0 * (k1[j] + 2.0 * k2[j] + 2.0 * k3[j] + k4[j])).collect();
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
pub fn thin<T: Clone>(v: &[T], max: usize) -> Vec<T> {
    if v.len() <= max {
        return v.to_vec();
    }
    let k = v.len().div_ceil(max);
    let mut out: Vec<T> = v.iter().step_by(k).cloned().collect();
    out.push(v[v.len() - 1].clone());
    out
}
