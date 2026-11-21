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
    e!("max", "Functions", "max(a, b, ...), min(a, b, ...)", "Largest or smallest of numbers (matrix entries count individually).", "max(1, 2/3, pi)"),
    e!("simplify", "Algebra", "simplify(expr)", "Combines fractions, cancels common factors and applies sin^2 + cos^2 = 1, keeping the simplest form.", "simplify(sin(y)^2 + cos(y)^2)"),
    e!("expand", "Algebra", "expand(expr)", "Multiplies out products and powers.", "expand((a+b)^2)"),
    e!("factor", "Algebra", "factor(expr)", "Pulls common factors out of sums.", "factor(a*b + a*c)"),
    e!("subs", "Algebra", "subs(expr, x, value)", "Replaces the variable x by a value.", "subs(x^2 + 1, x, 3)"),
    e!("complexity", "Algebra", "complexity(expr)", "Size of an expression (number of nodes).", "complexity(a*b + a*c)"),
    e!("and", "Algebra", "and(a, b), or(a, b), xor(a, b), not(a); also a && b, a || b, !a", "Logic on true, false and numbers (nonzero is true).", "and(true, false)"),
    e!("deriv", "Calculus", "deriv(expr, x)", "Derivative of expr with respect to x.", "deriv(x^3 - 2*x, x)"),
    e!("inv", "Matrices", "inv(A)", "Inverse of a matrix; works with symbolic entries. Matrices are written [a b; c d].", "A = [a b; c d]; inv(A)"),
    e!("det", "Matrices", "det(A)", "Determinant.", "det([a b; c d])"),
    e!("lu", "Matrices", "[L, U, P] = lu(A)", "LU factorization with pivoting, P*A = L*U (also [L, U] = lu(A) with A = L*U).", "[L, U, P] = lu([1 2; 3 4])"),
    e!("transpose", "Matrices", "transpose(A) or A'", "Transpose of a matrix.", "[1 2; 3 4]'"),
    e!("solve", "Matrices", "A \\ b", "Solves A*x = b.", "[2 1; 1 3] \\ [3; 5]"),
    e!("norm", "Matrices", "norm(A), norm(A, 1), norm(A, inf), norm(A, fro)", "Matrix norms: 2-norm (largest singular value, the default), 1-norm (largest column sum), inf-norm (largest row sum), Frobenius.", "norm([1 -2; 3 4], 1)"),
    e!("eye", "Matrices", "eye(n) or eye(n, m)", "Identity matrix.", "eye(3)"),
    e!("ones", "Matrices", "ones(n) or ones(n, m), zeros(n) or zeros(n, m)", "Matrix of ones or zeros.", "ones(2, 3)"),
    e!("rand", "Matrices", "rand(n) or rand(n, m)", "Random matrix with entries uniform in [0, 1).", "rand(2)"),
    e!("range", "Matrices", "a:b or a:step:b", "Row vector from a to b. Exact for exact numbers.", "0:1/4:1"),
    e!("linspace", "Matrices", "linspace(a, b) or linspace(a, b, n)", "Row vector of n evenly spaced values from a to b (default 100).", "linspace(0, 1, 5)"),
    e!("logspace", "Matrices", "logspace(a, b) or logspace(a, b, n)", "Row vector of n values from 10^a to 10^b (default 50).", "logspace(0, 2, 3)"),
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
];

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
            s + "Operators: + - * / ^ \\ && ||. Constants: pi, e, i (or j)."
        }
        Some(n) => match entry(n) {
            Some(e) => format!("{}\n  {}\n  Example: {}", e.usage, e.description, e.example),
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
