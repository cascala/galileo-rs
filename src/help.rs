//! Built-in documentation: `help` lists commands by topic, `help(name)` describes one.

pub struct Entry {
    pub name: &'static str,
    pub topic: &'static str,
    pub usage: &'static str,
    pub description: &'static str,
    pub example: &'static str,
}

macro_rules! e {
    ($n:expr, $t:expr, $u:expr, $d:expr, $x:expr) => {
        Entry { name: $n, topic: $t, usage: $u, description: $d, example: $x }
    };
}

pub const ENTRIES: &[Entry] = &[
    e!("help", "Basics", "help or help(name)", "Lists all commands, or describes one command.", "help(deriv)"),
    e!("who", "Basics", "who", "Lists the variables you defined, with their types (whos does the same).", "who"),
    e!("clear", "Basics", "clear", "Removes all variables you defined.", "clear"),
    e!("eval", "Basics", "eval(expr)", "Evaluates an expression numerically, for example a fraction or a constant like pi.", "eval(5/4)"),
    e!("info", "Basics", "info(expr)", "Shows the internal structure of an expression.", "info(x^2+1)"),
    e!("load", "Basics", "load(file)", "Runs the Galileo statements in a file.", "load(examples/a.gg)"),
    e!("sin", "Functions", "sin(x), cos(x), tan(x)", "Trigonometric functions. Exact at multiples of pi/6 and pi/4.", "sin(pi/6)"),
    e!("asin", "Functions", "asin(x), acos(x), atan(x)", "Inverse trigonometric functions.", "asin(1)"),
    e!("exp", "Functions", "exp(x)", "Exponential function.", "exp(1)"),
    e!("log", "Functions", "log(x)", "Natural logarithm.", "log(e)"),
    e!("sqrt", "Functions", "sqrt(x)", "Square root; sqrt(-1) is the imaginary unit i.", "sqrt(16)"),
    e!("abs", "Functions", "abs(x)", "Absolute value.", "abs(-3)"),
    e!("sinh", "Functions", "sinh(x), cosh(x), tanh(x), asinh(x), acosh(x), atanh(x)", "Hyperbolic functions and their inverses.", "deriv(tanh(x), x)"),
    e!("atan2", "Functions", "atan2(y, x)", "Angle of the point (x, y), in (-pi, pi].", "atan2(1.0, -1.0)"),
    e!("functions", "Functions", "a(t), A(r), f(x, y)", "A call to an unknown one-letter name (optionally with digits or a subscript, like a_1) is an undefined function. deriv uses the chain rule: deriv(a(t)^2, t) is 2*a(t)*a'(t). Partial derivatives of functions of several variables are written f_,1 (first argument) and so on.", "deriv(a(t)^2, t)"),
    e!("max", "Functions", "max(a, b, ...), min(a, b, ...)", "Largest or smallest of numbers (matrix entries count individually).", "max(1, 2/3, pi)"),
    e!("simplify", "Algebra", "simplify(expr)", "Combines fractions, cancels common factors and applies sin^2 + cos^2 = 1, keeping the simplest form.", "simplify(sin(y)^2 + cos(y)^2)"),
    e!("expand", "Algebra", "expand(expr)", "Multiplies out products and powers.", "expand((a+b)^2)"),
    e!("factor", "Algebra", "factor(expr)", "Factors a polynomial in one variable over the rationals, or pulls common factors out of sums.", "factor(x^3 - 6*x^2 + 11*x - 6)"),
    e!("subs", "Algebra", "subs(expr, x, value)", "Replaces the variable x by a value.", "subs(x^2 + 1, x, 3)"),
    e!("solve", "Algebra", "solve(f, x) or solve([f1 f2], [x y])", "Solves f = 0 for x and returns all solutions in a row. Exact for polynomials up to degree 2 and for polynomials whose other roots are rational; numeric (complex) roots otherwise. Rational functions work too. With a list of equations and unknowns, solves a linear system. For other equations use fsolve.", "solve(x^2 - 5*x + 6, x)"),
    e!("fsolve", "Algebra", "fsolve(f, x, guess)", "A numeric solution of f = 0 near the guess (Newton's method).", "fsolve(cos(x) - x, x, 1)"),
    e!("coeff", "Algebra", "coeff(p, x, n), degree(p, x)", "Coefficient of x^n in the polynomial p, and the degree of p in x.", "coeff((x+1)^3, x, 2)"),
    e!("numer", "Algebra", "numer(f), denom(f)", "Numerator and denominator after bringing f over a common denominator.", "denom(1/x + 1/y)"),
    e!("factorial", "Functions", "factorial(n), binomial(n, k)", "n! and the binomial coefficient n choose k, exact.", "binomial(10, 3)"),
    e!("floor", "Functions", "floor(x), ceil(x), round(x)", "Round down, up, or to the nearest whole number.", "round(2.5)"),
    e!("mod", "Functions", "mod(a, b), gcd(a, b), lcm(a, b)", "Remainder of a / b (with the sign of b), greatest common divisor, least common multiple.", "mod(17, 5)"),
    e!("complexity", "Algebra", "complexity(expr)", "Size of an expression (number of nodes).", "complexity(a*b + a*c)"),
    e!("and", "Algebra", "and(a, b), or(a, b), xor(a, b), not(a); also a && b, a || b, !a", "Logic on true, false and numbers (nonzero is true).", "and(true, false)"),
    e!("deriv", "Calculus", "deriv(expr, x)", "Derivative of expr with respect to x.", "deriv(x^3 - 2*x, x)"),
    e!("integrate", "Calculus", "integrate(f, x) or integrate(f, x, a, b)", "Antiderivative of f with respect to x (without the constant; log(x) stands for log|x|), or the definite integral from a to b. a and b can be inf or -inf. If no formula is found, a definite integral with numeric limits is computed numerically. int is the same.", "integrate(x*exp(x), x)"),
    e!("nintegrate", "Calculus", "nintegrate(f, x, a, b)", "Numeric definite integral (adaptive Gauss-Kronrod); a and b can be inf or -inf.", "nintegrate(exp(-x^2), x, -inf, inf)"),
    e!("limit", "Calculus", "limit(f, x, a) or limit(f, x, a, left), limit(f, x, a, right)", "Limit of f as x approaches a, from both sides or one side. a can be inf or -inf. Uses l'Hopital's rule for 0/0 and inf/inf.", "limit(sin(x)/x, x, 0)"),
    e!("taylor", "Calculus", "taylor(f, x, a, n) or taylor(f, x, n)", "Taylor polynomial of f around x = a (default 0) up to the term (x-a)^n. series is the same.", "taylor(exp(x), x, 0, 4)"),
    e!("sum", "Calculus", "sum(f, k, a, b), prod(f, k, a, b), sum(A)", "Sum or product of f for k = a, a+1, ..., b (whole numbers); sum(A) adds the entries of a matrix.", "sum(1/k^2, k, 1, 10)"),
    e!("inv", "Matrices", "inv(A)", "Inverse of a matrix; works with symbolic entries. Matrices are written [a b; c d].", "A = [a b; c d]; inv(A)"),
    e!("det", "Matrices", "det(A)", "Determinant.", "det([a b; c d])"),
    e!("lu", "Matrices", "[L, U, P] = lu(A)", "LU factorization with pivoting, P*A = L*U (also [L, U] = lu(A) with A = L*U).", "[L, U, P] = lu([1 2; 3 4])"),
    e!("transpose", "Matrices", "transpose(A) or A'", "Transpose of a matrix.", "[1 2; 3 4]'"),
    e!("backslash", "Matrices", "A \\ b", "Solves the linear system A*x = b for the column x.", "[2 1; 1 3] \\ [3; 5]"),
    e!("trace", "Matrices", "trace(A), size(A)", "Sum of the diagonal entries; size gives [rows columns].", "trace([1 2; 3 4])"),
    e!("eig", "Matrices", "eig(A), charpoly(A, x)", "Eigenvalues of a square matrix (exact when the characteristic polynomial factors or has degree 2, numeric otherwise), and the characteristic polynomial det(A - x*I).", "eig([2 1; 1 2])"),
    e!("dot", "Matrices", "dot(u, v), cross(u, v)", "Dot product of two vectors, and cross product of two vectors of length 3.", "cross([1 0 0], [0 1 0])"),
    e!("norm", "Matrices", "norm(A), norm(A, 1), norm(A, inf), norm(A, fro)", "Matrix norms: 2-norm (largest singular value, the default), 1-norm (largest column sum), inf-norm (largest row sum), Frobenius.", "norm([1 -2; 3 4], 1)"),
    e!("eye", "Matrices", "eye(n) or eye(n, m)", "Identity matrix.", "eye(3)"),
    e!("ones", "Matrices", "ones(n) or ones(n, m), zeros(n) or zeros(n, m)", "Matrix of ones or zeros.", "ones(2, 3)"),
    e!("rand", "Matrices", "rand(n) or rand(n, m)", "Random matrix with entries uniform in [0, 1).", "rand(2)"),
    e!("range", "Matrices", "a:b or a:step:b", "Row vector from a to b. Exact for exact numbers.", "0:1/4:1"),
    e!("linspace", "Matrices", "linspace(a, b) or linspace(a, b, n)", "Row vector of n evenly spaced values from a to b (default 100).", "linspace(0, 1, 5)"),
    e!("logspace", "Matrices", "logspace(a, b) or logspace(a, b, n)", "Row vector of n values from 10^a to 10^b (default 50).", "logspace(0, 2, 3)"),
    e!("index", "Matrices", "A[i], A[i, j], g[a, b], christoffel(g)[a, b, c]", "Indexing starts at 0. Works on matrices, metrics and tensor components.", "christoffel(metric.generate(polar))[1, 0, 1]"),
    e!("odesolve", "Calculus", "odesolve(rhs, unknowns, t, start, t0, t1 (, steps))", "Solves the system of first-order equations dy/dt = rhs numerically (Runge-Kutta) from t = t0 to t = t1 and returns the unknowns at t1: a number for one unknown, a row otherwise. The arguments are the same as the first six of odeplot; see help(odeplot).", "odesolve([v, -x], [x v], t, [1 0], 0, pi)"),
    e!("odeplot", "Calculus", "odeplot(rhs, unknowns, t, start, t0, t1, [X Y] (, steps))", "Solves dy/dt = rhs numerically and plots the solution. The arguments: (1) rhs, a row with the right-hand side of each equation, dy1/dt, dy2/dt, ...; (2) unknowns, the names y1, y2, ... in the same order; (3) t, the name of the independent variable; (4) start, the values of y1, y2, ... at t0; (5, 6) t0 and t1, where the solution starts and stops; (7) [X Y], what to draw: two expressions in t and the unknowns for the horizontal and vertical axes ([t y1] plots y1 against t, [y1 y2] is a phase portrait, [r*cos(phi) r*sin(phi)] turns polar coordinates into a picture), or three for a 3D curve; (8) optionally the number of Runge-Kutta steps (default 20000). The equations must be first order: write y'' = f as two equations, y' = w and w' = f, with unknowns [y w].", "odeplot([v, -x], [x v], t, [1 0], 0, 2*pi, [t x])"),
    e!("plot", "Plotting", "plot(f1, f2, ..., x, from, to) or plot(f, x)", "Plots one or more expressions of x. The range defaults to -10 .. 10.", "plot(sin(x), cos(x), x, 0, 2*pi)"),
    e!("parametric", "Plotting", "parametric(fx, fy, t, from, to)", "Plots the curve (fx(t), fy(t)), with equal scales on both axes.", "parametric(cos(3*t), sin(2*t), t, 0, 2*pi)"),
    e!("plot3d", "Plotting", "plot3d(f, x, a, b, y, c, d)", "Plots the surface z = f(x, y) over a <= x <= b, c <= y <= d.", "plot3d(sin(x)*cos(y), x, -pi, pi, y, -pi, pi)"),
    e!("surface", "Plotting", "surface(fx, fy, fz, u, a, b, v, c, d)", "Plots a parametric surface.", "surface(sin(u)*cos(v), sin(u)*sin(v), cos(u), u, 0, pi, v, 0, 2*pi)"),
    e!("parametric3d", "Plotting", "parametric3d(fx, fy, fz, t, from, to)", "Plots a curve in space.", "parametric3d(cos(t), sin(t), t/(2*pi), t, 0, 6*pi)"),
    e!("polarplot", "Plotting", "polarplot(r, phi, from, to)", "Plots the curve at distance r(phi) from the origin, for angles phi from..to.", "polarplot(1 + cos(phi), phi, 0, 2*pi)"),
    e!("show", "Plotting", "show(p1, p2, ...)", "Overlays plots: all 2D, or all 3D. Plots are values, so p = plot(...) stores one.", "show(plot(sin(x), x, 0, pi), plot(cos(x), x, 0, pi))"),
    e!("title", "Plotting", "title(p, \"text\"), xlabel(p, \"text\"), ylabel(p, \"text\"), zlabel(p, \"text\"), labels(p, \"x\", \"y\" (, \"z\"))", "A copy of plot p with a title or axis labels.", "labels(plot(sin(t), t, 0, 10), \"time t\", \"position\")"),
    e!("legend", "Plotting", "legend(p, \"first\", \"second\", ...)", "A copy of plot p with new names for its curves in the legend.", "legend(show(plot(sin(x), x, 0, pi), plot(cos(x), x, 0, pi)), \"sine\", \"cosine\")"),
    e!("metric", "Differential geometry", "metric([x1 x2 ...], [g11 g12 ...; ...]) or metric.generate(name)", "A metric with the given coordinates and components g_ab. Coordinates stay symbolic even if a variable of that name has a value. metric.generate gives a ready-made one: minkowski, euclidean, polar, spherical, two-sphere, three-sphere, torus, hyperbolic, schwarzschild.", "g = metric([r phi], [1 0; 0 r^2])"),
    e!("inducedmetric", "Differential geometry", "inducedmetric([x y z], [u v])", "The metric induced by flat space on the surface (or curve, or hypersurface) x(u, v), y(u, v), z(u, v): the first fundamental form.", "inducedmetric([sin(u)*cos(v) sin(u)*sin(v) cos(u)], [u v])"),
    e!("inverse", "Differential geometry", "inverse(g)", "Inverse metric g^ab.", "inverse(metric.generate(polar))"),
    e!("christoffel", "Differential geometry", "christoffel(g), christoffelfirst(g)", "Christoffel symbols of the second kind Gamma^a_bc (christoffelsecond is the same), or of the first kind Gamma_abc. Pick one with christoffel(g)[a, b, c].", "christoffel(metric.generate(two-sphere))"),
    e!("riemannsecond", "Differential geometry", "riemannsecond(g), riemannfirst(g)", "Riemann curvature tensor R^a_bcd (second) or R_abcd (first).", "riemannsecond(metric.generate(two-sphere))"),
    e!("riccitensor", "Differential geometry", "riccitensor(g), ricciscalar(g)", "Ricci tensor R_ab and Ricci scalar R.", "ricciscalar(metric.generate(two-sphere))"),
    e!("einsteintensor", "Differential geometry", "einsteintensor(g), einsteinscalar(g)", "Einstein tensor G_ab = R_ab - R g_ab / 2, and its trace.", "einsteintensor(metric.generate(schwarzschild))"),
    e!("weyltensor", "Differential geometry", "weyltensor(g), schoutentensor(g)", "Weyl conformal tensor C_abcd and Schouten tensor P_ab (3 or more dimensions).", "weyltensor(metric.generate(three-sphere))"),
    e!("gausscurvature", "Differential geometry", "gausscurvature(g)", "Gaussian curvature of a 2D metric.", "gausscurvature(metric.generate(two-sphere))"),
    e!("kretschmann", "Differential geometry", "kretschmann(g)", "Kretschmann scalar R_abcd R^abcd.", "kretschmann(metric.generate(schwarzschild))"),
    e!("geodesics", "Differential geometry", "geodesics(g)", "The geodesic equations of a metric, with ' for d/ds.", "geodesics(metric.generate(two-sphere))"),
    e!("geodesic", "Differential geometry", "geodesic(g, x0, v0, s1 (, [X Y (Z)] (, steps)))", "Integrates geodesics from position x0 with velocity v0 (one row per geodesic) up to parameter s1 and plots them, mapped through X, Y (, Z), expressions in the coordinates. Other variables in the metric need values first.", "geodesic(metric.generate(two-sphere), [pi/2 0], [1 1], 6, [sin(theta)*cos(phi) sin(theta)*sin(phi) cos(theta)])"),
];

const ALIASES: &[(&str, &str)] = &[
    ("whos", "who"),
    ("cos", "sin"),
    ("tan", "sin"),
    ("acos", "asin"),
    ("atan", "asin"),
    ("min", "max"),
    ("or", "and"),
    ("xor", "and"),
    ("not", "and"),
    ("zeros", "ones"),
    ("diff", "deriv"),
    ("int", "integrate"),
    ("series", "taylor"),
    ("prod", "sum"),
    ("product", "sum"),
    ("degree", "coeff"),
    ("denom", "numer"),
    ("binomial", "factorial"),
    ("ceil", "floor"),
    ("round", "floor"),
    ("gcd", "mod"),
    ("lcm", "mod"),
    ("size", "trace"),
    ("charpoly", "eig"),
    ("cross", "dot"),
    ("xlabel", "title"),
    ("ylabel", "title"),
    ("zlabel", "title"),
    ("labels", "title"),
    ("cosh", "sinh"),
    ("tanh", "sinh"),
    ("asinh", "sinh"),
    ("acosh", "sinh"),
    ("atanh", "sinh"),
    ("christoffelfirst", "christoffel"),
    ("christoffelsecond", "christoffel"),
    ("riemannfirst", "riemannsecond"),
    ("riemann", "riemannsecond"),
    ("ricciscalar", "riccitensor"),
    ("einsteinscalar", "einsteintensor"),
    ("schoutentensor", "weyltensor"),
    ("metric.generate", "metric"),
];

/// all names help knows about, for completion
pub fn names() -> Vec<&'static str> {
    let mut v: Vec<&str> = ENTRIES.iter().map(|e| e.name).chain(ALIASES.iter().map(|a| a.0)).collect();
    v.retain(|n| !matches!(*n, "functions" | "index" | "range" | "backslash"));
    v.sort();
    v.dedup();
    v
}

pub fn plain(e: &Entry) -> String {
    format!("{}\n  {}\n  Example: {}", e.usage, e.description, e.example)
}

pub fn markdown(e: &Entry) -> String {
    format!("**`{}`**\n\n{}\n\nExample: `{}`", e.usage, e.description, e.example)
}

pub fn entry(name: &str) -> Option<&'static Entry> {
    let n = name.trim();
    let key = ALIASES.iter().find(|(a, _)| *a == n).map_or(n, |(_, k)| *k);
    ENTRIES.iter().find(|e| e.name == key)
}

pub fn topics() -> Vec<(&'static str, Vec<&'static Entry>)> {
    let mut out: Vec<(&str, Vec<&Entry>)> = vec![];
    for e in ENTRIES {
        match out.iter_mut().find(|(t, _)| *t == e.topic) {
            Some(t) => t.1.push(e),
            None => out.push((e.topic, vec![e])),
        }
    }
    out
}

pub fn help(name: Option<&str>) -> String {
    match name {
        None | Some("") => {
            let mut s = String::from("Galileo commands (help(name) for details):\n");
            for (t, es) in topics() {
                s += &format!("  {}: {}\n", t, es.iter().map(|e| e.name).collect::<Vec<_>>().join(", "));
            }
            s + "Operators: + - * / ^ \\ && ||. Constants: pi, e, i (or j). End a line with ; to hide its result."
        }
        Some(n) => match entry(n) {
            Some(e) => plain(e),
            None => {
                let close: Vec<&str> =
                    ENTRIES.iter().map(|e| e.name).filter(|c| c.starts_with(&n[..n.len().min(2)])).collect();
                if close.is_empty() {
                    format!("help: no command called '{}'. Type help for a list.", n)
                } else {
                    format!("help: no command called '{}'. Did you mean: {}?", n, close.join(", "))
                }
            }
        },
    }
}
