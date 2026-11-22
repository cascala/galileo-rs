# galileo-rs

The [Galileo](https://github.com/cascala/galileo) computer algebra system, rewritten in Rust.

This is a redesign rather than a line-by-line port. The language is the same
(`deriv(x^3, x)`, `[a b; c d]`, `[L, U, P] = lu(A)`, ...), but the engine
underneath is new:

* **Exact numbers.** Integers and fractions are exact rationals: `1/3 + 2/6` is `2/3`,
  `sqrt(8)` is `2*sqrt(2)`, and they have no size limit: `2^521 - 1` keeps all 157 digits
  (small numbers use i128, larger ones switch to big integers automatically and never turn
  into floats). Decimals like `0.5` are floats, and floats are contagious.
* **Canonical forms.** Every expression is built through smart constructors that flatten
  sums and products, fold numbers, collect like terms and powers, and sort operands. So
  `8*y*u*y*u*y^5*y*u*u^6*r*y*7*y` is simply `56*r*u^9*y^10`, with no hand-written
  pattern rules.
* **Small core.** Subtraction is `a + (-1)*b`, division is `a * b^-1`, `exp(x)` is `e^x`.
  The printer turns these back into `-`, `/` and `exp`, and its output can be pasted back in.
* **Values, not statements.** The parser produces a syntax tree; functions are looked up by
  name at evaluation time, so `sinx` is just a variable and adding a function is one match arm.
  Unknown names are errors, except one-letter names like `a(t)` or `f2(x)`, which are
  undefined functions: `deriv(a(t)^2, t)` is `2*a(t)*a'(t)`.
* **Rational-function simplification.** `simplify` puts expressions over a common
  denominator, cancels polynomial factors exactly, and rewrites `sin^2` as `1-cos^2` where
  that helps. Curvature results come out tidy (the Kerr Ricci tensor is zero in about 2 s).
* **Late binding.** A variable holds an expression, and names in it are looked up again when
  it is used: after `f = a*x; a = 2`, `f` is `2*x`.

## Try it

```
cargo run --bin galileo            # interactive prompt
cargo run --bin galileo file.gg    # run a file
```

```
galileo> deriv(sin(x)/x^2, x)
cos(x)/x^2-2*sin(x)/x^3
galileo> simplify(ans)
(x*cos(x)-2*sin(x))/x^3
galileo> A = [a b; c d]; inv(A)
d/(a*d-b*c)	-b/(a*d-b*c)
-c/(a*d-b*c)	a/(a*d-b*c)
galileo> simplify(cos(y)^2 + sin(y)^2)
1
```

Or with Docker, on Intel or Apple Silicon:

```
docker run -it --rm cascala/galileo                          # interactive prompt
docker run --rm -v "$PWD:/work" cascala/galileo file.gg      # run a file
```

To publish the image for both architectures:

```
docker buildx build --platform linux/amd64,linux/arm64 -t cascala/galileo:0.3.0 -t cascala/galileo:latest --push .
```

Type `help` for the list of commands, and `help(name)` for one of them. A statement ending
in `;` runs without printing its result.

## Differential geometry

```
galileo> S = metric.generate(schwarzschild)
ds^2 = -(r-r_s)/r*dt^2 + r/(r-r_s)*dr^2 + r^2*dtheta^2 + r^2*sin(theta)^2*dphi^2
galileo> christoffel(S)[1,0,0]
r_s*(r-r_s)/(2*r^3)
galileo> kretschmann(S)
12*r_s^2/r^6
```

* `metric(coords, g)`, `metric.generate(name)` (minkowski, polar, euclidean, spherical,
  two-sphere, three-sphere, schwarzschild, torus, hyperbolic), `inducedmetric(embedding, coords)`
* `christoffel` (`christoffelfirst`, `christoffelsecond`), `riemann` (`riemannfirst`,
  `riemannsecond`), `riccitensor`, `ricciscalar`, `einsteintensor`, `einsteinscalar`,
  `schoutentensor`, `weyltensor`, `gausscurvature`, `kretschmann`, `geodesics`, `inverse`
* Components are indexed from 0: `g[0,0]`, `riemann(g)[1,0,1,0]`.
* `geodesic(g, x0, v0, s1)` integrates a geodesic and plots it.

## Plots and ODEs

`plot`, `parametric`, `plot3d`, `surface`, `parametric3d`, `odeplot`, `odesolve` and
`geodesic`. In the terminal a plot is drawn in text; in a notebook it is an interactive
Plotly chart with an SVG fallback. `odesolve` uses fixed-step RK4.

## Jupyter

`galileo --kernel` serves a simple line protocol on stdin/stdout (see `src/kernel.rs`):
a header line `execute <length>`, `complete <cursor> <length>` or `inspect <cursor> <length>`,
then the code; each reply is one line of JSON with MIME bundles (text, LaTeX, Markdown, SVG,
Plotly). [iGalileo](https://github.com/cascala/igalileo) wraps it in a small Python ipykernel.

## WebAssembly

The same engine runs in a browser. It has no dependencies and builds straight to WebAssembly
(about 800 KB, no imports):

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --lib
# target/wasm32-unknown-unknown/release/galileo_rs.wasm
```

`src/wasm.rs` exports a small C ABI (`galileo_alloc`, `galileo_execute`, `galileo_complete`,
`galileo_inspect`, `galileo_reset`, `galileo_seed`) whose replies are the same JSON as the
`--kernel` protocol. iGalileo's JupyterLite site and the Galileo playground use it.
`ls`, `cd` and `load` have no file system in the browser and report an error there.

## Language changes from the Scala version

* Inside a matrix, `[a -b]` has two elements (as in Matlab); `[a - b]` and `[a-b]` are one.
* `exit` is not available in a notebook.

## Layout

| File | What it does |
| --- | --- |
| `src/num.rs` | Exact rationals with float fallback |
| `src/expr.rs` | The expression tree and its canonical ordering |
| `src/build.rs` | Smart constructors (`add`, `mul`, `pow`, `func`) that keep expressions canonical |
| `src/print.rs` | Printer (fractions, signs, `sqrt`, `exp`) |
| `src/calc.rs` | `deriv`, `expand`, `factor`, `simplify`, substitution, numeric evaluation |
| `src/linalg.rs` | Matrices: product, inverse, determinant, LU, solve, norms |
| `src/parser.rs` | Lexer and Pratt parser |
| `src/eval.rs` | Sessions: variables, commands, built-in functions |
| `src/help.rs` | `help` and `help(name)` |
| `src/ratfun.rs` | Rational-function normal form behind `simplify` |
| `src/geometry.rs` | Metrics, curvature tensors, geodesic equations |
| `src/commands.rs` | Geometry, plot and ODE commands |
| `src/numeric.rs`, `src/ode.rs` | Compiled numeric evaluation and RK4 |
| `src/plot.rs` | Figures: text, SVG and Plotly JSON |
| `src/latex.rs`, `src/display.rs` | LaTeX and notebook MIME bundles |
| `src/kernel.rs` | `galileo --kernel` protocol |
| `src/wasm.rs` | WebAssembly entry points for the browser |

## Parity with the Scala version

`tests/parity/scala_tests.txt` holds every input/expected pair from the Scala test
suite. Output is formatted differently (`2*x` instead of `2.0*x`), so a case passes when
the Rust answer is mathematically equal to the Scala one, checked at random points. For
`simplify` and `factor` the Rust answer must also be no larger. Known, deliberate
differences are listed in `tests/parity.rs` (for example, `acos(cos(y))` stays as it is,
because it only equals `y` for `0 <= y <= pi`).

## Status

- [x] Core algebra, calculus, simplification, matrices, parser, REPL
- [x] Tensors and differential geometry (metrics, Christoffel, Riemann, Ricci, Einstein, Weyl, geodesics, ODE solver)
- [x] Plotting (SVG and Plotly) and LaTeX output
- [x] Jupyter kernel (through iGalileo)

## License

Apache 2.0, like the original.
