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
    e!("sin", "Functions", "sin(x), cos(x), tan(x)", "Trigonometric functions, in radians. Exact at multiples of pi/12 (15 degrees).", "sin(pi/6)"),
    e!("asin", "Functions", "asin(x), acos(x), atan(x)", "Inverse trigonometric functions.", "asin(1)"),
    e!("exp", "Functions", "exp(x)", "Exponential function.", "exp(1)"),
    e!("log", "Functions", "log(x)", "Natural logarithm.", "log(e)"),
    e!("sqrt", "Functions", "sqrt(x)", "Square root; sqrt(-1) is the imaginary unit i.", "sqrt(16)"),
    e!("abs", "Functions", "abs(x)", "Absolute value; for a complex number a + b*i the modulus sqrt(a^2 + b^2).", "abs(-3)"),
    e!("sinh", "Functions", "sinh(x), cosh(x), tanh(x), asinh(x), acosh(x), atanh(x)", "Hyperbolic functions and their inverses.", "deriv(tanh(x), x)"),
    e!("atan2", "Functions", "atan2(y, x)", "Angle of the point (x, y), in (-pi, pi].", "atan2(1.0, -1.0)"),
    e!("functions", "Functions", "a(t), A(r), f(x, y)", "A call to an unknown one-letter name (optionally with digits or a subscript, like a_1) is an undefined function. deriv uses the chain rule: deriv(a(t)^2, t) is 2*a(t)*a'(t). Partial derivatives of functions of several variables are written f_,1 (first argument) and so on.", "deriv(a(t)^2, t)"),
    e!("max", "Functions", "max(a, b, ...), min(a, b, ...)", "Largest or smallest of numbers (matrix entries count individually).", "max(1, 2/3, pi)"),
    e!("simplify", "Algebra", "simplify(expr)", "Combines fractions, cancels common factors and applies sin^2 + cos^2 = 1, keeping the simplest form.", "simplify(sin(y)^2 + cos(y)^2)"),
    e!("expand", "Algebra", "expand(expr)", "Multiplies out products and powers.", "expand((a+b)^2)"),
    e!("factor", "Algebra", "factor(expr)", "Factors a polynomial in one variable over the rationals, or pulls common factors out of sums. A whole number is factored into primes.", "factor(x^3 - 6*x^2 + 11*x - 6)"),
    e!("subs", "Algebra", "subs(expr, x, value)", "Replaces the variable x by a value.", "subs(x^2 + 1, x, 3)"),
    e!("solve", "Algebra", "solve(f, x) or solve([f1 f2], [x y])", "Solves f = 0 (or an equation lhs == rhs) for x and returns all solutions in a row. An equation in one of sin(x), cos(x), tan(x) gives the solutions with 0 <= x < 2*pi. Exact for polynomials up to degree 2 and for polynomials whose other roots are rational; numeric (complex) roots otherwise. Rational functions work too. With a list of equations and unknowns, solves a linear system. For other equations use fsolve.", "solve(x^2 - 5*x + 6, x)"),
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
    e!("sum", "Calculus", "sum(f, k, a, b), prod(f, k, a, b), sum(A)", "Sum or product of f for k = a, a+1, ..., b; sum(A) adds the entries of a matrix. With a symbolic or infinite upper limit, sum finds closed forms for polynomials (sum(k^2, k, 1, n)), geometric series and 1/k^2, 1/k^4, 1/k^6.", "sum(1/k^2, k, 1, 10)"),
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
    e!("define", "Basics", "f(x) := expr, f(x, y) := expr (or f(x) = expr)", "Defines your own function. Call it like a built-in one: f(3), f(t + 1). f'(x) and f''(x) are its derivatives. Functions can call themselves together with if(...).", "f(x) := x^2 + 1; f(3)"),
    e!("if", "Basics", "if(condition, a, b)", "a when the condition is true, b otherwise; only the chosen branch is computed, so recursive functions stop. When the recursive call is the whole branch, as in count(n, k) := if(k == n, k, count(n, k + 1)), it is run as a loop and may take millions of steps. Conditions use == ~= < > <= >= and && || !.", "fact(n) := if(n == 0, 1, n*fact(n-1)); fact(10)"),
    e!("equations", "Algebra", "lhs == rhs", "Compares two values: true or false when that can be decided, otherwise an equation, which solve, fsolve, implicitplot and dsolve accept.", "solve(x^2 == 2*x + 3, x)"),
    e!("polydiv", "Algebra", "polydiv(p, q, x)", "Long division of polynomials in x: [quotient, remainder] with p = quotient*q + remainder and the remainder of lower degree than q.", "polydiv(x^3 - 1, x - 1, x)"),
    e!("trigexpand", "Algebra", "trigexpand(expr), trigreduce(expr)", "trigexpand applies the addition formulas: sin(a+b), cos(2x), tan(a+b), ... in terms of sines and cosines of the single angles. trigreduce goes the other way: products and powers of sin and cos become sums of sines and cosines of multiple angles.", "trigexpand(cos(2*x))"),
    e!("real", "Complex numbers", "real(z), imag(z), conj(z), abs(z), arg(z)", "Real part, imaginary part, complex conjugate, modulus |z| and argument (angle) of z. Variables count as real.", "arg(1 + i)"),
    e!("rect", "Complex numbers", "rect(z), polar(z)", "rect writes z as a + b*i; polar gives [r, phi] with z = r*exp(i*phi).", "polar(1 + i)"),
    e!("isprime", "Number theory", "isprime(n), nextprime(n), prevprime(n)", "Whether n is prime (Miller-Rabin), the next prime after n and the last one before it. Whole numbers up to about 1.7e38.", "isprime(2^61 - 1)"),
    e!("ifactor", "Number theory", "ifactor(n), factor(n), factors(n)", "Prime factorization of a whole number: as text 2^3 * 3^2 * 5, or as rows [prime exponent]. Uses trial division, then Pollard's rho.", "ifactor(2^64 + 1)"),
    e!("primes", "Number theory", "primes(n), primes(a, b), prime(k), primepi(n)", "The primes up to n (or from a to b) by the sieve of Eratosthenes, the k-th prime, and the number of primes up to n.", "primes(30)"),
    e!("divisors", "Number theory", "divisors(n), sigma(n), totient(n)", "All divisors of n, their sum, and Euler's phi: how many of 1..n share no factor with n (eulerphi is the same).", "divisors(28)"),
    e!("powmod", "Number theory", "powmod(a, b, n), modinv(a, n)", "a^b mod n computed by repeated squaring (fast for huge b), and the inverse of a mod n (the x with a*x = 1 mod n).", "powmod(2, 1000, 1001)"),
    e!("multorder", "Number theory", "multorder(a, n), primroot(n)", "The smallest k > 0 with a^k = 1 mod n, and the smallest primitive root mod n (a number whose powers give every unit).", "multorder(2, 13)"),
    e!("crt", "Number theory", "crt([r1, r2, ...], [m1, m2, ...])", "Chinese remainder theorem: the smallest x >= 0 with x = r_i mod m_i (moduli without common factors).", "crt([2, 3, 2], [3, 5, 7])"),
    e!("pollard", "Number theory", "pollard(n), isqrt(n)", "One factor of n found by Pollard's rho method, and the integer square root (largest k with k^2 <= n).", "pollard(8051)"),
    e!("letters", "Number theory", "letters(text), word(list)", "letters turns the letters of a text into numbers A = 0, B = 1, ..., Z = 25 (other characters are skipped); word turns numbers back into letters, taking them mod 26. For shift ciphers.", "word(mod(letters(\"HELLO\") + 3, 26))"),
    e!("encode", "Number theory", "encode(text), decode(n)", "encode turns a text of at most 15 characters into one whole number (its bytes in base 256); decode turns the number back into the text. For RSA examples.", "decode(encode(\"Hi Galileo\"))"),
    e!("perm", "Groups", "perm(\"(1 2 3)(4 5)\", n), cycles(p)", "A permutation of 1..n written in one-line notation, p = [p(1), p(2), ...], from cycle notation; cycles(p) goes back.", "cycles([2, 3, 1, 5, 4])"),
    e!("compose", "Groups", "compose(p, q), pinverse(p)", "p after q: first q, then p, so compose(p, q)[i] = p[q[i]]. pinverse undoes p.", "compose([2, 3, 1], [2, 1, 3])"),
    e!("porder", "Groups", "porder(p), psign(p)", "Order of a permutation (smallest k with p^k = identity: the lcm of its cycle lengths) and its sign (+1 even, -1 odd).", "porder([2, 3, 1, 5, 4])"),
    e!("generate", "Groups", "generate(g1, g2, ...), cayley(G)", "All permutations made by composing the generators, one per row (the identity first); cayley(G) is the group table: entry (i, j) is the row number of G_i after G_j.", "cayley(generate([2, 3, 1]))"),
    e!("table", "Calculus", "table(expr, k, a, b (, step)) or table(expr, k, [v1, v2, ...])", "A row with expr for each k; when expr is itself a row, the rows stack into a matrix.", "table(k^2, k, 1, 10)"),
    e!("iterate", "Calculus", "iterate(f, x, x0, n)", "x0, f(x0), f(f(x0)), ... : n steps of x -> f(x), where f is an expression in x.", "iterate(x/2 + 1/x, x, 1.0, 5)"),
    e!("dsolve", "Calculus", "dsolve(ode, y, x) or dsolve(ode, y, x, x0, [y(x0), y'(x0), ...])", "Exact solution of a linear differential equation written with y, y', y'', ...: constant coefficients of any order (forcing by polynomials, exponentials, sines and cosines), and first-order equations y' + p(x) y = q(x). Without start values the answer has constants C1, C2, ....", "dsolve(y'' + y == 0, y, x, 0, [1, 0])"),
    e!("implicitplot", "Plotting", "implicitplot(F, x, x0, x1, y, y0, y1)", "Draws the curve F(x, y) = 0, or an equation such as x^2 + y^2 == 1, in the window x0..x1, y0..y1 (equal scales). A list [F1, F2, ...] draws several curves.", "implicitplot(x^2/4 + y^2 == 1, x, -3, 3, y, -2, 2)"),
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
    e!("discover", "Discovery", "discover(data, [x y]) or discover(data, [x y], [term1 term2 ...]) (, sigma)", "Searches for the simplest formula that explains the last variable from the others (symbolic regression). (1) data: a matrix with one row per observation and one column per variable; (2) the names of the columns, in order; the quantity to explain comes last, or is the one written with primes: [t x''] explains the second derivative of x, estimated from neighbouring rows (the first column is then the independent variable). (3) optionally a row of candidate terms; discover then looks for the fewest terms whose weighted sum fits (the SINDy idea). Without terms it tries power laws with simple exponents (y = C*x^(3/2)) and short sums of powers of the inputs. (4) optionally one uncertainty per row: the fit then weighs each row by 1/sigma^2 and reports chi2/dof. The result lists candidates from simple to complex with their misfit (relative error, or chi2/dof with uncertainties) and marks the best one: the simplest formula that predicts each row from the others (leave-one-out) at most twice as badly as any, with at least two rows per fitted constant (with uncertainties: the simplest whose chi2/dof is close to the lowest). Constants close to simple numbers such as 1, 3/2 or 4*pi^2 are recognised. d[k] picks candidate k, best(d) the best one.", "discover([1 1; 2 2.83; 3 5.2; 4 8], [a T])"),
    e!("check", "Discovery", "check(law, data, [x y]) or check(law, data, [x y], sigma)", "Tests a formula for the last variable against observations. Names in the law that are not columns are constants to fit (k in k/p^2): they are fitted by least squares and reported with their uncertainty. The result shows each row's observed and predicted value and the difference; with uncertainties (one per row) the difference is in standard deviations and the law is called consistent when chi2/dof <= 3 and every point is within 3 sigma. Give a row of laws, [k/p, k/p^2], to compare several, or the result of discover to test its best formula on new data: check(d, newdata).", "check(a^(3/2), [1 1; 4 8; 9 27], [a T])"),
    e!("prove", "Discovery", "prove(lhs, rhs) or prove(lhs, rhs, [a value; b value])", "Shows step by step that lhs = rhs: it brings the difference over a common denominator, expands and simplifies it to 0, and lists what it had to assume (denominators nonzero, square roots of nonnegative numbers, logarithms of positive numbers). With a third argument it first substitutes the given values. If the difference is not 0 it looks for a counterexample at random points; if it finds none it says so, which is evidence but not a proof.", "prove(x^2/x, x)"),
    e!("best", "Discovery", "best(d)", "The formula discover marked as best, as an expression you can use further.", "best(discover([1 1; 2 4; 3 9], [x y]))"),
    e!("scatter", "Plotting", "scatter(data, [x y]) or scatter(data, [names...], [X Y]) (, sigma)", "Plots observations as points. data has one row per observation; the names label its columns. With two columns the first goes across and the second up; otherwise give [X Y], two expressions in the column names, such as [log(a) log(T)] for a log-log plot. A last argument with one uncertainty per row (for the last column) draws error bars. Combine with show(scatter(...), plot(...)) to draw a fitted curve through the data.", "scatter([1 1; 2 4; 3 9], [x y])"),
    e!("odedata", "Calculus", "odedata(rhs, unknowns, t, start, t0, t1, n)", "Like odesolve, but returns the whole solution as a table with n+1 rows [t y1 y2 ...], evenly spaced from t0 to t1, ready for scatter or discover.", "odedata([v, -x], [x v], t, [1 0], 0, 6, 60)"),
    e!("transform", "Matrices", "transform(data, [names], [expr1 expr2 ...])", "Makes a new table from a data table: the names label the columns of data, and each expression (in those names) gives one column of the result, computed row by row. Use it to change units, take logarithms or compute 1/r.", "transform([1 2; 2 4], [x y], [x 1/y])"),
    e!("indexing", "Matrices", "A(k), A(i, j)", "Picks from a stored table or list, counting from 1: A(k) is entry k of a list (row or column) or row k of a table, A(i, j) the entry in row i, column j.", "v = [3 5 7]; v(2)"),
    e!("rows", "Matrices", "rows(A), cols(A), length(v)", "The number of rows or columns of a table; length is the number of entries of a list (the larger of the two).", "rows(generate([2 3 1]))"),
    e!("column", "Matrices", "column(A, k)", "Column k of a matrix (counting from 1), as a column vector.", "column([1 2; 3 4], 2)"),
];

const ALIASES: &[(&str, &str)] = &[
    ("cols", "rows"),
    ("length", "rows"),
    ("word", "letters"),
    ("decode", "encode"),
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
    (":=", "define"),
    ("==", "equations"),
    ("imag", "real"),
    ("conj", "real"),
    ("arg", "real"),
    ("polar", "rect"),
    ("trigreduce", "trigexpand"),
    ("nextprime", "isprime"),
    ("prevprime", "isprime"),
    ("factors", "ifactor"),
    ("prime", "primes"),
    ("primepi", "primes"),
    ("sigma", "divisors"),
    ("totient", "divisors"),
    ("eulerphi", "divisors"),
    ("modinv", "powmod"),
    ("primroot", "multorder"),
    ("isqrt", "pollard"),
    ("cycles", "perm"),
    ("pinverse", "compose"),
    ("psign", "porder"),
    ("cayley", "generate"),
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
            s + "Operators: + - * / ^ \\ && || == ~= < > <= >=. Constants: pi, e, i (or j). End a line with ; to hide its result."
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
