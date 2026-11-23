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
        Entry {
            name: $n,
            topic: $t,
            usage: $u,
            description: $d,
            example: $x,
        }
    };
}

pub const ENTRIES: &[Entry] = &[
    e!(
        "help",
        "Basics",
        "help or help(name)",
        "Lists all commands, or describes one command.",
        "help(deriv)"
    ),
    e!(
        "who",
        "Basics",
        "who",
        "Lists the variables you defined, with their types (whos does the same).",
        "who"
    ),
    e!(
        "clear",
        "Basics",
        "clear",
        "Removes all variables you defined.",
        "clear"
    ),
    e!(
        "eval",
        "Basics",
        "eval(expr)",
        "Evaluates an expression numerically, for example a fraction or a constant like pi.",
        "eval(5/4)"
    ),
    e!(
        "info",
        "Basics",
        "info(expr)",
        "Shows the internal structure of an expression.",
        "info(x^2+1)"
    ),
    e!(
        "load",
        "Basics",
        "load(file), load(RSA)",
        "Runs the Galileo statements in a file. load(RSA) instead defines the RSA Factoring Challenge numbers: RSA100, RSA110, ... RSA617 by decimal digits and RSA576, RSA640, RSA704, RSA768, RSA896, RSA1024, RSA1536, RSA2048 by bits, with the two primes of each factored one as RSA100p and RSA100q and so on. Factors are included up to RSA250, plus RSA270 and RSA896 (RSA-896, factored in September 2026, is the largest so far); RSA260 was factored in September 2026 too, but its factors are not included yet.",
        "load(RSA)"
    ),
    e!(
        "sin",
        "Functions",
        "sin(x), cos(x), tan(x)",
        "Trigonometric functions, in radians. Exact at multiples of pi/12 (15 degrees).",
        "sin(pi/6)"
    ),
    e!(
        "asin",
        "Functions",
        "asin(x), acos(x), atan(x)",
        "Inverse trigonometric functions.",
        "asin(1)"
    ),
    e!(
        "exp",
        "Functions",
        "exp(x)",
        "Exponential function.",
        "exp(1)"
    ),
    e!("log", "Functions", "log(x)", "Natural logarithm.", "log(e)"),
    e!(
        "sqrt",
        "Functions",
        "sqrt(x)",
        "Square root; sqrt(-1) is the imaginary unit i.",
        "sqrt(16)"
    ),
    e!(
        "abs",
        "Functions",
        "abs(x)",
        "Absolute value; for a complex number a + b*i the modulus sqrt(a^2 + b^2).",
        "abs(-3)"
    ),
    e!(
        "sinh",
        "Functions",
        "sinh(x), cosh(x), tanh(x), asinh(x), acosh(x), atanh(x)",
        "Hyperbolic functions and their inverses.",
        "deriv(tanh(x), x)"
    ),
    e!(
        "atan2",
        "Functions",
        "atan2(y, x)",
        "Angle of the point (x, y), in (-pi, pi].",
        "atan2(1.0, -1.0)"
    ),
    e!(
        "functions",
        "Functions",
        "a(t), A(r), f(x, y)",
        "A call to an unknown one-letter name (optionally with digits or a subscript, like a_1) is an undefined function. deriv uses the chain rule: deriv(a(t)^2, t) is 2*a(t)*a'(t). Partial derivatives of functions of several variables are written f_,1 (first argument) and so on.",
        "deriv(a(t)^2, t)"
    ),
    e!(
        "max",
        "Functions",
        "max(a, b, ...), min(a, b, ...)",
        "Largest or smallest of numbers (matrix entries count individually).",
        "max(1, 2/3, pi)"
    ),
    e!(
        "simplify",
        "Algebra",
        "simplify(expr)",
        "Combines fractions, cancels common factors and applies sin^2 + cos^2 = 1, keeping the simplest form.",
        "simplify(sin(y)^2 + cos(y)^2)"
    ),
    e!(
        "expand",
        "Algebra",
        "expand(expr)",
        "Multiplies out products and powers.",
        "expand((a+b)^2)"
    ),
    e!(
        "factor",
        "Algebra",
        "factor(expr), factor(expr, field)",
        "Factors a polynomial with rational coefficients over the integers, in any variables: (a*x + b)*(c*x + d) comes back from its expanded form. factor(expr, n) factors modulo a prime n (factor(x^4 + 1, 17)); factor(expr, sqrt(d)) over the numbers a + b*sqrt(d), with i for the Gaussian numbers (factor(x^2 + y^2, i)); these two take one variable, or a homogeneous polynomial in two. Irreducible factors of any degree are found (Zassenhaus: factor mod a prime, lift, recombine); x^n - 1 and x^n + 1 split into cyclotomic polynomials. Other expressions get their common factors pulled out. A whole number of any size is factored into primes (trial division, Pollard p - 1 and rho). A number a^n + b^n or a^n - b^n (typed that way, or recognized as b^n + 1 or b^n - 1) is first cut into its algebraic pieces, values of cyclotomic polynomials split again by Aurifeuillean factors, and then each piece is factored. Pieces that do not split in about a second are shown as C followed by their number of digits, with the full number on a line below.",
        "factor(x^4 + 4)"
    ),
    e!(
        "subs",
        "Algebra",
        "subs(expr, x, value)",
        "Replaces the variable x by a value.",
        "subs(x^2 + 1, x, 3)"
    ),
    e!(
        "solve",
        "Algebra",
        "solve(f, x) or solve([f1 f2], [x y])",
        "Solves f = 0 (or an equation lhs == rhs) for x and returns all solutions in a row. An equation in one of sin(x), cos(x), tan(x) gives the solutions with 0 <= x < 2*pi. Exact for polynomials up to degree 2 and for polynomials whose other roots are rational; numeric (complex) roots otherwise. Rational functions work too. With a list of equations and unknowns, solves a linear system. For other equations use fsolve.",
        "solve(x^2 - 5*x + 6, x)"
    ),
    e!(
        "fsolve",
        "Algebra",
        "fsolve(f, x, guess)",
        "A numeric solution of f = 0 near the guess (Newton's method).",
        "fsolve(cos(x) - x, x, 1)"
    ),
    e!(
        "coeff",
        "Algebra",
        "coeff(p, x, n), degree(p, x)",
        "Coefficient of x^n in the polynomial p, and the degree of p in x.",
        "coeff((x+1)^3, x, 2)"
    ),
    e!(
        "numer",
        "Algebra",
        "numer(f), denom(f)",
        "Numerator and denominator after bringing f over a common denominator.",
        "denom(1/x + 1/y)"
    ),
    e!(
        "factorial",
        "Functions",
        "factorial(n), binomial(n, k)",
        "n! and the binomial coefficient n choose k, exact.",
        "binomial(10, 3)"
    ),
    e!(
        "floor",
        "Functions",
        "floor(x), ceil(x), round(x)",
        "Round down, up, or to the nearest whole number.",
        "round(2.5)"
    ),
    e!(
        "mod",
        "Functions",
        "mod(a, b), gcd(a, b), lcm(a, b)",
        "Remainder of a / b (with the sign of b), greatest common divisor, least common multiple.",
        "mod(17, 5)"
    ),
    e!(
        "complexity",
        "Algebra",
        "complexity(expr)",
        "Size of an expression (number of nodes).",
        "complexity(a*b + a*c)"
    ),
    e!(
        "and",
        "Algebra",
        "and(a, b), or(a, b), xor(a, b), not(a); also a && b, a || b, !a",
        "Logic on true, false and numbers (nonzero is true).",
        "and(true, false)"
    ),
    e!(
        "deriv",
        "Calculus",
        "deriv(expr, x)",
        "Derivative of expr with respect to x.",
        "deriv(x^3 - 2*x, x)"
    ),
    e!(
        "integrate",
        "Calculus",
        "integrate(f, x) or integrate(f, x, a, b)",
        "Antiderivative of f with respect to x (without the constant; log(x) stands for log|x|), or the definite integral from a to b. a and b can be inf or -inf. If no formula is found, a definite integral with numeric limits is computed numerically. int is the same.",
        "integrate(x*exp(x), x)"
    ),
    e!(
        "nintegrate",
        "Calculus",
        "nintegrate(f, x, a, b)",
        "Numeric definite integral (adaptive Gauss-Kronrod); a and b can be inf or -inf.",
        "nintegrate(exp(-x^2), x, -inf, inf)"
    ),
    e!(
        "limit",
        "Calculus",
        "limit(f, x, a) or limit(f, x, a, left), limit(f, x, a, right)",
        "Limit of f as x approaches a, from both sides or one side. a can be inf or -inf. Uses l'Hopital's rule for 0/0 and inf/inf.",
        "limit(sin(x)/x, x, 0)"
    ),
    e!(
        "taylor",
        "Calculus",
        "taylor(f, x, a, n) or taylor(f, x, n)",
        "Taylor polynomial of f around x = a (default 0) up to the term (x-a)^n. series is the same.",
        "taylor(exp(x), x, 0, 4)"
    ),
    e!(
        "sum",
        "Calculus",
        "sum(f, k, a, b), prod(f, k, a, b), sum(A)",
        "Sum or product of f for k = a, a+1, ..., b; sum(A) adds the entries of a matrix. With a symbolic or infinite upper limit, sum finds closed forms for polynomials (sum(k^2, k, 1, n)), geometric series and 1/k^2, 1/k^4, 1/k^6.",
        "sum(1/k^2, k, 1, 10)"
    ),
    e!(
        "inv",
        "Matrices",
        "inv(A)",
        "Inverse of a matrix; works with symbolic entries. Matrices are written [a b; c d].",
        "A = [a b; c d]; inv(A)"
    ),
    e!(
        "det",
        "Matrices",
        "det(A)",
        "Determinant.",
        "det([a b; c d])"
    ),
    e!(
        "lu",
        "Matrices",
        "[L, U, P] = lu(A)",
        "LU factorization with pivoting, P*A = L*U (also [L, U] = lu(A) with A = L*U).",
        "[L, U, P] = lu([1 2; 3 4])"
    ),
    e!(
        "transpose",
        "Matrices",
        "transpose(A) or A'",
        "Transpose of a matrix.",
        "[1 2; 3 4]'"
    ),
    e!(
        "backslash",
        "Matrices",
        "A \\ b",
        "Solves the linear system A*x = b for the column x.",
        "[2 1; 1 3] \\ [3; 5]"
    ),
    e!(
        "trace",
        "Matrices",
        "trace(A), size(A)",
        "Sum of the diagonal entries; size gives [rows columns].",
        "trace([1 2; 3 4])"
    ),
    e!(
        "eig",
        "Matrices",
        "eig(A), charpoly(A, x)",
        "Eigenvalues of a square matrix (exact when the characteristic polynomial factors or has degree 2, numeric otherwise), and the characteristic polynomial det(A - x*I).",
        "eig([2 1; 1 2])"
    ),
    e!(
        "dot",
        "Matrices",
        "dot(u, v), cross(u, v)",
        "Dot product of two vectors, and cross product of two vectors of length 3.",
        "cross([1 0 0], [0 1 0])"
    ),
    e!(
        "norm",
        "Matrices",
        "norm(A), norm(A, 1), norm(A, inf), norm(A, fro)",
        "Matrix norms: 2-norm (largest singular value, the default), 1-norm (largest column sum), inf-norm (largest row sum), Frobenius.",
        "norm([1 -2; 3 4], 1)"
    ),
    e!(
        "eye",
        "Matrices",
        "eye(n) or eye(n, m)",
        "Identity matrix.",
        "eye(3)"
    ),
    e!(
        "ones",
        "Matrices",
        "ones(n) or ones(n, m), zeros(n) or zeros(n, m)",
        "Matrix of ones or zeros.",
        "ones(2, 3)"
    ),
    e!(
        "rand",
        "Matrices",
        "rand(n) or rand(n, m)",
        "Random matrix with entries uniform in [0, 1).",
        "rand(2)"
    ),
    e!(
        "range",
        "Matrices",
        "a:b or a:step:b",
        "Row vector from a to b. Exact for exact numbers.",
        "0:1/4:1"
    ),
    e!(
        "linspace",
        "Matrices",
        "linspace(a, b) or linspace(a, b, n)",
        "Row vector of n evenly spaced values from a to b (default 100).",
        "linspace(0, 1, 5)"
    ),
    e!(
        "logspace",
        "Matrices",
        "logspace(a, b) or logspace(a, b, n)",
        "Row vector of n values from 10^a to 10^b (default 50).",
        "logspace(0, 2, 3)"
    ),
    e!(
        "index",
        "Matrices",
        "A[i], A[i, j], g[a, b], christoffel(g)[a, b, c]",
        "Indexing starts at 0. Works on matrices, metrics and tensor components.",
        "christoffel(metric.generate(polar))[1, 0, 1]"
    ),
    e!(
        "odesolve",
        "Calculus",
        "odesolve(rhs, unknowns, t, start, t0, t1 (, steps))",
        "Solves the system of first-order equations dy/dt = rhs numerically (Runge-Kutta) from t = t0 to t = t1 and returns the unknowns at t1: a number for one unknown, a row otherwise. The arguments are the same as the first six of odeplot; see help(odeplot).",
        "odesolve([v, -x], [x v], t, [1 0], 0, pi)"
    ),
    e!(
        "odeplot",
        "Calculus",
        "odeplot(rhs, unknowns, t, start, t0, t1, [X Y] (, steps))",
        "Solves dy/dt = rhs numerically and plots the solution. The arguments: (1) rhs, a row with the right-hand side of each equation, dy1/dt, dy2/dt, ...; (2) unknowns, the names y1, y2, ... in the same order; (3) t, the name of the independent variable; (4) start, the values of y1, y2, ... at t0; (5, 6) t0 and t1, where the solution starts and stops; (7) [X Y], what to draw: two expressions in t and the unknowns for the horizontal and vertical axes ([t y1] plots y1 against t, [y1 y2] is a phase portrait, [r*cos(phi) r*sin(phi)] turns polar coordinates into a picture), or three for a 3D curve; (8) optionally the number of Runge-Kutta steps (default 20000). The equations must be first order: write y'' = f as two equations, y' = w and w' = f, with unknowns [y w].",
        "odeplot([v, -x], [x v], t, [1 0], 0, 2*pi, [t x])"
    ),
    e!(
        "plot",
        "Plotting",
        "plot(f1, f2, ..., x, from, to) or plot(f, x)",
        "Plots one or more expressions of x. The range defaults to -10 .. 10.",
        "plot(sin(x), cos(x), x, 0, 2*pi)"
    ),
    e!(
        "parametric",
        "Plotting",
        "parametric(fx, fy, t, from, to)",
        "Plots the curve (fx(t), fy(t)), with equal scales on both axes.",
        "parametric(cos(3*t), sin(2*t), t, 0, 2*pi)"
    ),
    e!(
        "plot3d",
        "Plotting",
        "plot3d(f, x, a, b, y, c, d)",
        "Plots the surface z = f(x, y) over a <= x <= b, c <= y <= d.",
        "plot3d(sin(x)*cos(y), x, -pi, pi, y, -pi, pi)"
    ),
    e!(
        "surface",
        "Plotting",
        "surface(fx, fy, fz, u, a, b, v, c, d)",
        "Plots a parametric surface.",
        "surface(sin(u)*cos(v), sin(u)*sin(v), cos(u), u, 0, pi, v, 0, 2*pi)"
    ),
    e!(
        "parametric3d",
        "Plotting",
        "parametric3d(fx, fy, fz, t, from, to)",
        "Plots a curve in space.",
        "parametric3d(cos(t), sin(t), t/(2*pi), t, 0, 6*pi)"
    ),
    e!(
        "polarplot",
        "Plotting",
        "polarplot(r, phi, from, to)",
        "Plots the curve at distance r(phi) from the origin, for angles phi from..to.",
        "polarplot(1 + cos(phi), phi, 0, 2*pi)"
    ),
    e!(
        "show",
        "Plotting",
        "show(p1, p2, ...)",
        "Overlays plots: all 2D, or all 3D. Plots are values, so p = plot(...) stores one.",
        "show(plot(sin(x), x, 0, pi), plot(cos(x), x, 0, pi))"
    ),
    e!(
        "title",
        "Plotting",
        "title(p, \"text\"), xlabel(p, \"text\"), ylabel(p, \"text\"), zlabel(p, \"text\"), labels(p, \"x\", \"y\" (, \"z\"))",
        "A copy of plot p with a title or axis labels.",
        "labels(plot(sin(t), t, 0, 10), \"time t\", \"position\")"
    ),
    e!(
        "legend",
        "Plotting",
        "legend(p, \"first\", \"second\", ...)",
        "A copy of plot p with new names for its curves in the legend.",
        "legend(show(plot(sin(x), x, 0, pi), plot(cos(x), x, 0, pi)), \"sine\", \"cosine\")"
    ),
    e!(
        "define",
        "Basics",
        "f(x) := expr, f(x, y) := expr (or f(x) = expr)",
        "Defines your own function. Call it like a built-in one: f(3), f(t + 1). f'(x) and f''(x) are its derivatives. Functions can call themselves together with if(...). ans in the body is replaced by the last result when the function is defined, so deriv(f(x), x) followed by d(x) := ans makes d the derivative.",
        "f(x) := x^2 + 1; f(3)"
    ),
    e!(
        "if",
        "Basics",
        "if(condition, a, b)",
        "a when the condition is true, b otherwise; only the chosen branch is computed, so recursive functions stop. When the recursive call is the whole branch, as in count(n, k) := if(k == n, k, count(n, k + 1)), it is run as a loop and may take millions of steps. Conditions use == ~= < > <= >= and && || !.",
        "fact(n) := if(n == 0, 1, n*fact(n-1)); fact(10)"
    ),
    e!(
        "equations",
        "Algebra",
        "lhs == rhs",
        "Compares two values: true or false when that can be decided, otherwise an equation, which solve, fsolve, implicitplot and dsolve accept.",
        "solve(x^2 == 2*x + 3, x)"
    ),
    e!(
        "polydiv",
        "Algebra",
        "polydiv(p, q, x)",
        "Long division of polynomials in x: [quotient, remainder] with p = quotient*q + remainder and the remainder of lower degree than q.",
        "polydiv(x^3 - 1, x - 1, x)"
    ),
    e!(
        "trigexpand",
        "Algebra",
        "trigexpand(expr), trigreduce(expr)",
        "trigexpand applies the addition formulas: sin(a+b), cos(2x), tan(a+b), ... in terms of sines and cosines of the single angles. trigreduce goes the other way: products and powers of sin and cos become sums of sines and cosines of multiple angles.",
        "trigexpand(cos(2*x))"
    ),
    e!(
        "real",
        "Complex numbers",
        "real(z), imag(z), conj(z), abs(z), arg(z)",
        "Real part, imaginary part, complex conjugate, modulus |z| and argument (angle) of z. Variables count as real.",
        "arg(1 + i)"
    ),
    e!(
        "rect",
        "Complex numbers",
        "rect(z), polar(z)",
        "rect writes z as a + b*i; polar gives [r, phi] with z = r*exp(i*phi).",
        "polar(1 + i)"
    ),
    e!(
        "isprime",
        "Number theory",
        "isprime(n), nextprime(n), prevprime(n)",
        "Whether n is prime (Miller-Rabin: certain below 3.3e24, a 20-base probable-prime test above), the next prime after n and the last one before it. Any size.",
        "isprime(2^521 - 1)"
    ),
    e!(
        "ifactor",
        "Number theory",
        "ifactor(n), factor(n), factors(n)",
        "Prime factorization of a whole number: as text 2^3 * 3^2 * 5, or as rows [prime exponent]. Uses trial division, then Pollard's p - 1 and rho. factor(n) also splits numbers a^n +- b^n algebraically first (see help(factor) and algfactor).",
        "ifactor(2^64 + 1)"
    ),
    e!(
        "semiprime",
        "Factoring lab",
        "semiprime(digits), semiprime(digits, seed), semifactors(digits, seed)",
        "A balanced semiprime N = p q with exactly that many digits, p and q primes of the same size drawn at random from the seed (default 0); the same seed always gives the same number. semifactors gives [p q] with p < q. These are the test numbers of the factoring lab.",
        "semiprime(30, 7)"
    ),
    e!(
        "labrun",
        "Factoring lab",
        "labrun(method, N), labrun(method, N, seconds)",
        "Runs one factoring method on N with a time limit (default 5 s) and reports the factor and the work it took. Work is counted in multiplications modulo N, the same on every computer. A method is a built-in name in quotes: \"trial\", \"fermat\", \"hart\", \"rho\", \"pm1\", \"squfof\", \"ecm\", \"qs\"; a lab program like \"iter(x^2 + 1, x - y, 2)\" or \"rel(rti, 64)\" (see help(algosearch)); or the name of your own function f(n) := ... that returns a factor (its work is counted in evaluation steps, so compare user functions by time). r[0] is the factor.",
        "labrun(\"qs\", semiprime(40, 1))"
    ),
    e!(
        "scoreboard",
        "Factoring lab",
        "scoreboard(methods, digits, trials, seconds)",
        "Runs methods (one, or a row like [\"rho\" \"qs\"]) on balanced semiprimes of the given sizes (a row like 12:4:40, the default), trials numbers per size (default 3), at most seconds per number (default 1). A method stops at the first size where it solves nothing. Shows a table of solved counts, median work and time, and fits how the cost grows: N^a (exponential in the digits), L(1/2) (quadratic sieve) or L(1/3) (number field sieve). Natively the fit uses time, in the browser work units. scoreplot(s) draws it.",
        "scoreboard([\"rho\" \"squfof\" \"qs\"], 16:4:32)"
    ),
    e!(
        "scoreplot",
        "Factoring lab",
        "scoreplot(s), scoreplot(s1, s2, ...)",
        "Plots scoreboard results: log10 of the cost against the number of digits, one series of points per method, plus reference curves for N^(1/2), N^(1/4), L(1/2, 1) and L(1/3, 1.92) drawn through the first point (only where they stay near the data). The fitted growth laws are listed in the scoreboard table.",
        "scoreplot(scoreboard(\"rho\", 12:2:20))"
    ),
    e!(
        "algosearch",
        "Factoring lab",
        "algosearch(seconds), algosearch(seconds, mode), algosearch(seconds, mode, seed), algosearch(seconds, mode, seed, digits)",
        "Searches for new factoring procedures for about that many seconds (default 5). Programs are written in a small language: iter(f, h, x0) iterates x -> f(x, y, i) from x0, with y the value of x at the last power-of-two step, and checks gcd(h, N) (Pollard rho is iter(x^2 + 1, x - y, 2), p - 1 is iter(pow(x, i), x - 1, 2)); rel(s, fb) makes s at step i, checks whether s^2 mod N is a square, and with fb factor-base primes combines smooth values by linear algebra mod 2 (Fermat is rel(rt + i, 0), Hart rel(rti, 0), Dixon rel(rt + i, 64)). Names: x, y, i, rt = ceil(sqrt N), rti = ceil(sqrt(i N)), B = floor(N^(1/3)), d0, d1 (digits of N in base B), small integers; operations + - * ^k, pow(a, i), luc(a) (Lucas V_i, Williams p + 1), inv(a) (a failed inverse mod N reveals a factor). A third kind, grp(world, e, r), raises an element of a group mod N to the power e(i) at step i and checks gcd with N: worlds mul (order p - 1, so grp(mul, i, 0) is Pollard p - 1), quad(d) (a + b sqrt(d), p + 1), cubic(c) (polynomials mod t^3 - c, Bach-Shallit), mat (2 x 2 matrices) and ec(a) (elliptic curves); with r > 0 it starts over with a new element and parameter every r steps, so grp(ec(3), i, 100) is ECM. mode \"evolve\" (default) mutates and recombines the best programs, \"random\" samples at random, \"exhaustive\" tries every small program in order of size. Programs are scored by total work on 12 semiprimes of 12 to 18 digits (or of the sizes in digits, e.g. [20 24 28], after a 12-digit screening; slow), then the best are re-run on other numbers two digits larger next to Pollard rho, with a paired p-value: gains that only show on the training numbers are luck. r[k] is program k as text.",
        "algosearch(10)"
    ),
    e!(
        "leaderboard",
        "Factoring lab",
        "leaderboard()",
        "The best programs found by every algosearch in this session, with how they compare to Pollard rho on numbers no search trained on.",
        "leaderboard()"
    ),
    e!(
        "gnfs",
        "Factoring lab",
        "gnfs(N), gnfs(N, seconds), gnfs(N, f, m), gnfs(N, f, m, seconds)",
        "Factors N with the number field sieve and prints every step: the polynomial f with f(m) divisible by N (base m, or a special form when N divides b^k +- 1: that is the special number field sieve), the factor bases, the line sieve over pairs (a, b) where both a - b m and the norm b^d f(a/b) are smooth, the linear algebra mod 2 (with quadratic characters), the square root in the number field (Hensel lifting at a prime where f splits) and gcd(X - Y, N). Give your own f (a polynomial in x, made monic if needed) and m to try a polynomial. Default limit 60 s. This is a teaching version: below about 60 digits the quadratic sieve is faster. r[0] is the factor.",
        "gnfs(2^128 + 1)"
    ),
    e!(
        "nfspoly",
        "Factoring lab",
        "nfspoly(N), nfspoly(N, degree)",
        "Polynomials for the number field sieve on N with their scores: the estimated size in bits of the two norms over a typical sieve region plus Murphy's alpha (how often values of f are divisible by small primes). Lower scores sieve faster. Lists the special forms found for N and the best base-m polynomials of degree 3 to 5 (or of the given degree). r[j] is the coefficient vector of the j-th polynomial, constant term first.",
        "nfspoly(2^128 + 1)"
    ),
    e!(
        "leakhunt",
        "Factoring lab",
        "leakhunt(digits, samples, seed)",
        "Looks for information about p hidden in cheap features of N = p q: residues of N, Jacobi symbols, leading digits, digits in other bases, the continued fraction of sqrt(N) and, up to 6 digits, class numbers h(-4N). Each feature is tested against properties of the smaller prime p (p mod 4, p mod 3, a middle bit, ...) with a chi-squared test on half of the samples (default 16 digits, 4000 samples), corrected for the number of tests (Benjamini-Hochberg) and re-tested on the other half; a naive Bayes predictor tries all features together. Control properties (p = q mod 3, the second bit of p) are known to leak and show that the test works.",
        "leakhunt(12, 2000)"
    ),
    e!(
        "cyclotomic",
        "Number theory",
        "cyclotomic(n), cyclotomic(n, v)",
        "The cyclotomic polynomial Phi_n(x): the product of x - z over the primitive n-th roots of unity z. It has integer coefficients, is irreducible, and x^n - 1 is the product of Phi_d(x) over the divisors d of n. cyclotomic(n, v) puts v in place of x: a number (cyclotomic(8, 10) = 10001) or an expression (cyclotomic(4, 2*x^2)).",
        "cyclotomic(12)"
    ),
    e!(
        "algfactor",
        "Number theory",
        "algfactor(n), algfactor(n, effort), algfactor(n, f), algfactor(n, f, effort)",
        "Searches for an algebraic form of the whole number n and factors along it, one row per piece with its size, its prime factors and the composite parts that did not split. It tests, in order: a form typed as a^k +- b^k; n = a^k + 1 or a^k - 1 for every base a (by k-th roots); n = a^k +- b^k for b up to 12*effort; Sophie Germain's a^4 + 4b^4; n dividing b^k +- 1 (a cofactor from a factor table) for b up to 12*effort and k up to 3000*effort; and finally n = f(x) for every polynomial family in families() (built in, plus any added with addfamily); then the same families with a second variable, n = y^d f(x/y) for y up to 50*effort (so x^6 + 27 also finds x^6 + 27y^6); the list includes 81 Aurifeuillean-type families g(c x^2) and g(c x^3) found by search (like x^6 + 8x^2 - 1 = (x^3 + 2x^2 + 2x - 1)(x^3 - 2x^2 + 2x + 1)), and every family is first checked modulo small numbers so most are ruled out without big arithmetic; then Fermat's method (n = a^2 - b^2 with a near sqrt(n), 20000*effort steps), which splits n at once when two factors are close; and last, for every degree d and every base x near the d-th root of n (and of n/c for small leading digits c), it writes n in base x with balanced digits, and when only a few digits are nonzero and all are at most 8*effort, it factors that polynomial. With a polynomial f in one variable, only n = f(x) is tested. Pieces are values of cyclotomic polynomials (split again by Aurifeuillean factors) or of the factors of f. effort (default 1, about a second; up to 10000) also gives Pollard rho and p - 1 that many times more work on each piece. r[k] is the value of piece k.",
        "algfactor((2^1062 + 1)/5)"
    ),
    e!(
        "addfamily",
        "Number theory",
        "addfamily(f), families()",
        "addfamily adds a polynomial family f(x) with whole-number coefficients (for example one found with factorsearch) to the list that algfactor tries on every number, for the rest of the session. families() shows the list: the built-in families and the added ones.",
        "addfamily(x^8 + x^4 + 1)"
    ),
    e!(
        "factorsearch",
        "Number theory",
        "factorsearch(expr, k, a, b) (, step)",
        "Hunts for factorizations in a family of polynomials: for k = a, a+1, ..., b, puts k into expr, factors the result over the integers and lists the k for which it splits, with the factors. expr must become a polynomial in one variable, or a homogeneous one in two. r[j] is the j-th factorization found.",
        "factorsearch(x^4 + k, k, 1, 100)"
    ),
    e!(
        "pm1",
        "Number theory",
        "pm1(n), pm1(n, B)",
        "Pollard's p - 1 method with bound B (default 100000): computes 3^E mod n for E = lcm(1, ..., B). A prime p of n with every prime power of p - 1 at most B has 3^E = 1 mod p, because (Z/p)* has order p - 1, so gcd(3^E - 1, n) finds p. It says which case happened and shows p - 1.",
        "pm1(2^67 - 1)"
    ),
    e!(
        "ecm",
        "Number theory",
        "ecm(n), ecm(n, curves), ecm(n, curves, B1)",
        "Lenstra's elliptic curve method: like p - 1, but in the group of points of a random elliptic curve mod p, whose order varies between p + 1 - 2 sqrt(p) and p + 1 + 2 sqrt(p) from curve to curve. Each curve is a new chance for a smooth group order. Defaults: 50 curves, B1 = 11000 (good for factors of about 20 digits; 50000 for 25, 250000 for 30). algfactor and factor also try a few curves after rho, more with a larger effort.",
        "ecm(nextprime(7*10^15)*nextprime(10^45))"
    ),
    e!(
        "squfof",
        "Number theory",
        "squfof(n)",
        "Shanks' square forms factorization (SQUFOF): walks the continued fraction of sqrt(k n) until a square appears among the denominators, then walks back from the square form to a factor. It takes about n^(1/4) steps, so it is for n up to about 30 digits.",
        "squfof(1000000000039 * 10000000000037)"
    ),
    e!(
        "classgroup",
        "Number theory",
        "classgroup(D)",
        "The class group of the negative discriminant D: one reduced form (a, b, c), meaning a x^2 + b x y + c y^2 with b^2 - 4ac = D and |b| <= a <= c, for each class, with its order under composition. Forms of order 2 are called ambiguous; for D = -4N they are exactly the factorizations of N.",
        "classgroup(-4*391)"
    ),
    e!(
        "reduceform",
        "Number theory",
        "reduceform([a, b, c]), composeforms(f, g), formpow(f, k)",
        "Binary quadratic forms a x^2 + b x y + c y^2 written [a, b, c]: the reduced form in the same class, Gauss composition of two forms of the same discriminant (the group law of the class group), and the k-th power of a form.",
        "composeforms([2, 1, 3], [2, 1, 3])"
    ),
    e!(
        "classfactor",
        "Number theory",
        "classfactor(n), classfactor(n, B)",
        "Factoring with the class group (after Schnorr and Lenstra): raises a form of discriminant -4kn to the product of the odd prime powers up to B (default 50000), then squares until the identity appears; the form just before has order 2, so it is ambiguous and splits n. Works when the odd part of the class number is B-smooth; for n up to about 25 digits.",
        "classfactor(nextprime(10^8)*nextprime(3*10^8))"
    ),
    e!(
        "polyselect",
        "Number theory",
        "polyselect(n), polyselect(n, d), polyselect(n, d, generations)",
        "Explores polynomial selection for the number field sieve: pairs f(x) of degree d (default by size: 3 below 60 digits, 4 below 110, 5 below 220, else 6) and g(x) = x - m with f(m) = n. Starts from the base-m expansion of n, runs a root sieve over rotations f + (j x + k)(x - m) to lower Murphy's alpha (more roots mod small primes), then an evolutionary search over the leading coefficient, the shift of m and the rotation, scored by Murphy's E (Dickman's rho of both sides over the sieve region). Shows lognorm, alpha and E for each step and the best pair, with the check f(m) - n = 0. A teaching tool: real selectors (Kleinjung's algorithm in CADO-NFS and msieve) search far more leading coefficients and also translate f.",
        "polyselect(nextprime(10^59)*nextprime(10^60))"
    ),
    e!(
        "digits",
        "Number theory",
        "digits(n)",
        "The number of decimal digits of the whole number n.",
        "digits(2^521 - 1)"
    ),
    e!(
        "primes",
        "Number theory",
        "primes(n), primes(a, b), prime(k), primepi(n)",
        "The primes up to n (or from a to b) by the sieve of Eratosthenes, the k-th prime, and the number of primes up to n.",
        "primes(30)"
    ),
    e!(
        "divisors",
        "Number theory",
        "divisors(n), sigma(n), totient(n)",
        "All divisors of n, their sum, and Euler's phi: how many of 1..n share no factor with n (eulerphi is the same).",
        "divisors(28)"
    ),
    e!(
        "powmod",
        "Number theory",
        "powmod(a, b, n), modinv(a, n)",
        "a^b mod n computed by repeated squaring (fast for huge b), and the inverse of a mod n (the x with a*x = 1 mod n).",
        "powmod(2, 1000, 1001)"
    ),
    e!(
        "multorder",
        "Number theory",
        "multorder(a, n), primroot(n)",
        "The smallest k > 0 with a^k = 1 mod n, and the smallest primitive root mod n (a number whose powers give every unit).",
        "multorder(2, 13)"
    ),
    e!(
        "crt",
        "Number theory",
        "crt([r1, r2, ...], [m1, m2, ...])",
        "Chinese remainder theorem: the smallest x >= 0 with x = r_i mod m_i (moduli without common factors).",
        "crt([2, 3, 2], [3, 5, 7])"
    ),
    e!(
        "pollard",
        "Number theory",
        "pollard(n), isqrt(n)",
        "One factor of n found by Pollard's rho method, and the integer square root (largest k with k^2 <= n).",
        "pollard(8051)"
    ),
    e!(
        "letters",
        "Number theory",
        "letters(text), word(list)",
        "letters turns the letters of a text into numbers A = 0, B = 1, ..., Z = 25 (other characters are skipped); word turns numbers back into letters, taking them mod 26. For shift ciphers.",
        "word(mod(letters(\"HELLO\") + 3, 26))"
    ),
    e!(
        "encode",
        "Number theory",
        "encode(text), decode(n)",
        "encode turns a text into one whole number (its bytes in base 256); decode turns the number back into the text. For RSA examples.",
        "decode(encode(\"Hi Galileo\"))"
    ),
    e!(
        "perm",
        "Groups",
        "perm(\"(1 2 3)(4 5)\", n), cycles(p)",
        "A permutation of 1..n written in one-line notation, p = [p(1), p(2), ...], from cycle notation; cycles(p) goes back.",
        "cycles([2, 3, 1, 5, 4])"
    ),
    e!(
        "compose",
        "Groups",
        "compose(p, q), pinverse(p)",
        "p after q: first q, then p, so compose(p, q)[i] = p[q[i]]. pinverse undoes p.",
        "compose([2, 3, 1], [2, 1, 3])"
    ),
    e!(
        "porder",
        "Groups",
        "porder(p), psign(p)",
        "Order of a permutation (smallest k with p^k = identity: the lcm of its cycle lengths) and its sign (+1 even, -1 odd).",
        "porder([2, 3, 1, 5, 4])"
    ),
    e!(
        "generate",
        "Groups",
        "generate(g1, g2, ...), cayley(G)",
        "All permutations made by composing the generators, one per row (the identity first); cayley(G) is the group table: entry (i, j) is the row number of G_i after G_j.",
        "cayley(generate([2, 3, 1]))"
    ),
    e!(
        "table",
        "Calculus",
        "table(expr, k, a, b (, step)) or table(expr, k, [v1, v2, ...])",
        "A row with expr for each k; when expr is itself a row, the rows stack into a matrix.",
        "table(k^2, k, 1, 10)"
    ),
    e!(
        "iterate",
        "Calculus",
        "iterate(f, x, x0, n)",
        "x0, f(x0), f(f(x0)), ... : n steps of x -> f(x), where f is an expression in x.",
        "iterate(x/2 + 1/x, x, 1.0, 5)"
    ),
    e!(
        "dsolve",
        "Calculus",
        "dsolve(ode, y, x) or dsolve(ode, y, x, x0, [y(x0), y'(x0), ...])",
        "Exact solution of a linear differential equation written with y, y', y'', ...: constant coefficients of any order (forcing by polynomials, exponentials, sines and cosines), and first-order equations y' + p(x) y = q(x). Without start values the answer has constants C1, C2, ....",
        "dsolve(y'' + y == 0, y, x, 0, [1, 0])"
    ),
    e!(
        "implicitplot",
        "Plotting",
        "implicitplot(F, x, x0, x1, y, y0, y1)",
        "Draws the curve F(x, y) = 0, or an equation such as x^2 + y^2 == 1, in the window x0..x1, y0..y1 (equal scales). A list [F1, F2, ...] draws several curves.",
        "implicitplot(x^2/4 + y^2 == 1, x, -3, 3, y, -2, 2)"
    ),
    e!(
        "metric",
        "Differential geometry",
        "metric([x1 x2 ...], [g11 g12 ...; ...]) or metric.generate(name)",
        "A metric with the given coordinates and components g_ab. Coordinates stay symbolic even if a variable of that name has a value. metric.generate gives a ready-made one: minkowski, euclidean, polar, spherical, two-sphere, three-sphere, torus, hyperbolic, schwarzschild.",
        "g = metric([r phi], [1 0; 0 r^2])"
    ),
    e!(
        "inducedmetric",
        "Differential geometry",
        "inducedmetric([x y z], [u v])",
        "The metric induced by flat space on the surface (or curve, or hypersurface) x(u, v), y(u, v), z(u, v): the first fundamental form.",
        "inducedmetric([sin(u)*cos(v) sin(u)*sin(v) cos(u)], [u v])"
    ),
    e!(
        "inverse",
        "Differential geometry",
        "inverse(g)",
        "Inverse metric g^ab.",
        "inverse(metric.generate(polar))"
    ),
    e!(
        "christoffel",
        "Differential geometry",
        "christoffel(g), christoffelfirst(g)",
        "Christoffel symbols of the second kind Gamma^a_bc (christoffelsecond is the same), or of the first kind Gamma_abc. Pick one with christoffel(g)[a, b, c].",
        "christoffel(metric.generate(two-sphere))"
    ),
    e!(
        "riemannsecond",
        "Differential geometry",
        "riemannsecond(g), riemannfirst(g)",
        "Riemann curvature tensor R^a_bcd (second) or R_abcd (first).",
        "riemannsecond(metric.generate(two-sphere))"
    ),
    e!(
        "riccitensor",
        "Differential geometry",
        "riccitensor(g), ricciscalar(g)",
        "Ricci tensor R_ab and Ricci scalar R.",
        "ricciscalar(metric.generate(two-sphere))"
    ),
    e!(
        "einsteintensor",
        "Differential geometry",
        "einsteintensor(g), einsteinscalar(g)",
        "Einstein tensor G_ab = R_ab - R g_ab / 2, and its trace.",
        "einsteintensor(metric.generate(schwarzschild))"
    ),
    e!(
        "weyltensor",
        "Differential geometry",
        "weyltensor(g), schoutentensor(g)",
        "Weyl conformal tensor C_abcd and Schouten tensor P_ab (3 or more dimensions).",
        "weyltensor(metric.generate(three-sphere))"
    ),
    e!(
        "gausscurvature",
        "Differential geometry",
        "gausscurvature(g)",
        "Gaussian curvature of a 2D metric.",
        "gausscurvature(metric.generate(two-sphere))"
    ),
    e!(
        "kretschmann",
        "Differential geometry",
        "kretschmann(g)",
        "Kretschmann scalar R_abcd R^abcd.",
        "kretschmann(metric.generate(schwarzschild))"
    ),
    e!(
        "geodesics",
        "Differential geometry",
        "geodesics(g)",
        "The geodesic equations of a metric, with ' for d/ds.",
        "geodesics(metric.generate(two-sphere))"
    ),
    e!(
        "geodesic",
        "Differential geometry",
        "geodesic(g, x0, v0, s1 (, [X Y (Z)] (, steps)))",
        "Integrates geodesics from position x0 with velocity v0 (one row per geodesic) up to parameter s1 and plots them, mapped through X, Y (, Z), expressions in the coordinates. Other variables in the metric need values first.",
        "geodesic(metric.generate(two-sphere), [pi/2 0], [1 1], 6, [sin(theta)*cos(phi) sin(theta)*sin(phi) cos(theta)])"
    ),
    e!(
        "discover",
        "Discovery",
        "discover(data, [x y]) or discover(data, [x y], [term1 term2 ...]) (, sigma)",
        "Searches for the simplest formula that explains the last variable from the others (symbolic regression). (1) data: a matrix with one row per observation and one column per variable; (2) the names of the columns, in order; the quantity to explain comes last, or is the one written with primes: [t x''] explains the second derivative of x, estimated from neighbouring rows (the first column is then the independent variable). (3) optionally a row of candidate terms; discover then looks for the fewest terms whose weighted sum fits (the SINDy idea). Without terms it tries power laws with simple exponents (y = C*x^(3/2)) and short sums of powers of the inputs. (4) optionally one uncertainty per row: the fit then weighs each row by 1/sigma^2 and reports chi2/dof. The result lists candidates from simple to complex with their misfit (relative error, or chi2/dof with uncertainties) and marks the best one: the simplest formula that predicts each row from the others (leave-one-out) at most twice as badly as any, with at least two rows per fitted constant (with uncertainties: the simplest whose chi2/dof is close to the lowest). Constants close to simple numbers such as 1, 3/2 or 4*pi^2 are recognised. d[k] picks candidate k, best(d) the best one.",
        "discover([1 1; 2 2.83; 3 5.2; 4 8], [a T])"
    ),
    e!(
        "check",
        "Discovery",
        "check(law, data, [x y]) or check(law, data, [x y], sigma)",
        "Tests a formula for the last variable against observations. Names in the law that are not columns are constants to fit (k in k/p^2): they are fitted by least squares and reported with their uncertainty. The result shows each row's observed and predicted value and the difference; with uncertainties (one per row) the difference is in standard deviations and the law is called consistent when chi2/dof <= 3 and every point is within 3 sigma. Give a row of laws, [k/p, k/p^2], to compare several, or the result of discover to test its best formula on new data: check(d, newdata).",
        "check(a^(3/2), [1 1; 4 8; 9 27], [a T])"
    ),
    e!(
        "prove",
        "Discovery",
        "prove(lhs, rhs) or prove(lhs, rhs, [a value; b value])",
        "Shows step by step that lhs = rhs: it brings the difference over a common denominator, expands and simplifies it to 0, and lists what it had to assume (denominators nonzero, square roots of nonnegative numbers, logarithms of positive numbers). With a third argument it first substitutes the given values. If the difference is not 0 it looks for a counterexample at random points; if it finds none it says so, which is evidence but not a proof.",
        "prove(x^2/x, x)"
    ),
    e!(
        "best",
        "Discovery",
        "best(d)",
        "The formula discover marked as best, as an expression you can use further.",
        "best(discover([1 1; 2 4; 3 9], [x y]))"
    ),
    e!(
        "scatter",
        "Plotting",
        "scatter(data, [x y]) or scatter(data, [names...], [X Y]) (, sigma)",
        "Plots observations as points. data has one row per observation; the names label its columns. With two columns the first goes across and the second up; otherwise give [X Y], two expressions in the column names, such as [log(a) log(T)] for a log-log plot. A last argument with one uncertainty per row (for the last column) draws error bars. Combine with show(scatter(...), plot(...)) to draw a fitted curve through the data.",
        "scatter([1 1; 2 4; 3 9], [x y])"
    ),
    e!(
        "odedata",
        "Calculus",
        "odedata(rhs, unknowns, t, start, t0, t1, n)",
        "Like odesolve, but returns the whole solution as a table with n+1 rows [t y1 y2 ...], evenly spaced from t0 to t1, ready for scatter or discover.",
        "odedata([v, -x], [x v], t, [1 0], 0, 6, 60)"
    ),
    e!(
        "transform",
        "Matrices",
        "transform(data, [names], [expr1 expr2 ...])",
        "Makes a new table from a data table: the names label the columns of data, and each expression (in those names) gives one column of the result, computed row by row. Use it to change units, take logarithms or compute 1/r.",
        "transform([1 2; 2 4], [x y], [x 1/y])"
    ),
    e!(
        "indexing",
        "Matrices",
        "A(k), A(i, j)",
        "Picks from a stored table or list, counting from 1: A(k) is entry k of a list (row or column) or row k of a table, A(i, j) the entry in row i, column j.",
        "v = [3 5 7]; v(2)"
    ),
    e!(
        "rows",
        "Matrices",
        "rows(A), cols(A), length(v)",
        "The number of rows or columns of a table; length is the number of entries of a list (the larger of the two).",
        "rows(generate([2 3 1]))"
    ),
    e!(
        "rref",
        "Matrices",
        "rref(A), rank(A), null(A)",
        "Row reduction: rref is the reduced row echelon form, rank the number of independent rows, null a basis of the solutions of A*x = 0 (one vector per column; empty when x = 0 is the only solution).",
        "rref([1 2 3; 4 5 6; 7 8 9])"
    ),
    e!(
        "eigvec",
        "Matrices",
        "eigvec(A), [V, D] = eig(A)",
        "Eigenvectors, one per column, for exact eigenvalues. [V, D] = eig(A) gives the eigenvectors V and the diagonal matrix D of eigenvalues, so that A = V*D*inv(V).",
        "[V, D] = eig([2 1; 1 2])"
    ),
    e!(
        "orbitdata",
        "Calculus",
        "orbitdata(rule, x, x0, r, r0, r1, nr, skip, keep)",
        "For nr+1 values of the parameter r from r0 to r1: iterate x -> rule (an expression in x and r) from x0, throw away the first skip values and keep the next keep values. Returns rows [r x] for scatter: a bifurcation diagram. Computed with decimals.",
        "scatter(orbitdata(r*x*(1-x), x, 0.5, r, 2.8, 4, 200, 500, 25), [r x])"
    ),
    e!(
        "mandelbrot",
        "Plotting",
        "mandelbrot(re0, re1, im0, im1, n), julia(c, re0, re1, im0, im1, n)",
        "The points of a grid n points wide whose orbit under z -> z^2 + c stays within |z| <= 2 for 200 steps, as rows [re im] for scatter. mandelbrot varies c (starting from z = 0); julia fixes c and varies the start z.",
        "scatter(mandelbrot(-2.2, 0.8, -1.2, 1.2, 180), [re im])"
    ),
    e!(
        "column",
        "Matrices",
        "column(A, k)",
        "Column k of a matrix (counting from 1), as a column vector.",
        "column([1 2; 3 4], 2)"
    ),
];

const ALIASES: &[(&str, &str)] = &[
    ("julia", "mandelbrot"),
    ("rank", "rref"),
    ("null", "rref"),
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
    ("semifactors", "semiprime"),
    ("nfs", "gnfs"),
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
    ("composeforms", "reduceform"),
    ("formpow", "reduceform"),
    ("cycles", "perm"),
    ("pinverse", "compose"),
    ("psign", "porder"),
    ("cayley", "generate"),
];

/// all names help knows about, for completion
pub fn names() -> Vec<&'static str> {
    let mut v: Vec<&str> = ENTRIES
        .iter()
        .map(|e| e.name)
        .chain(ALIASES.iter().map(|a| a.0))
        .collect();
    v.retain(|n| !matches!(*n, "functions" | "index" | "range" | "backslash"));
    v.sort();
    v.dedup();
    v
}

pub fn plain(e: &Entry) -> String {
    format!("{}\n  {}\n  Example: {}", e.usage, e.description, e.example)
}

pub fn markdown(e: &Entry) -> String {
    format!(
        "**`{}`**\n\n{}\n\nExample: `{}`",
        e.usage, e.description, e.example
    )
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
                s += &format!(
                    "  {}: {}\n",
                    t,
                    es.iter().map(|e| e.name).collect::<Vec<_>>().join(", ")
                );
            }
            s + "Operators: + - * / ^ \\ && || == ~= < > <= >=. Constants: pi, e, i (or j). End a line with ; to hide its result."
        }
        Some(n) => match entry(n) {
            Some(e) => plain(e),
            None => {
                let close: Vec<&str> = ENTRIES
                    .iter()
                    .map(|e| e.name)
                    .filter(|c| c.starts_with(&n[..n.len().min(2)]))
                    .collect();
                if close.is_empty() {
                    format!("help: no command called '{}'. Type help for a list.", n)
                } else {
                    format!(
                        "help: no command called '{}'. Did you mean: {}?",
                        n,
                        close.join(", ")
                    )
                }
            }
        },
    }
}
