# galileo-rs

The [Galileo](https://github.com/cascala/galileo) computer algebra system, rewritten in Rust.

This is a redesign rather than a line-by-line port. The language is the same
(`deriv(x^3, x)`, `[a b; c d]`, `[L, U, P] = lu(A)`, ...), but the engine
underneath is new:

* **Exact numbers.** Integers and fractions are exact rationals: `1/3 + 2/6` is `2/3`,
  `sqrt(8)` is `2*sqrt(2)`. Decimals like `0.5` are floats, and floats are contagious.
* **Canonical forms.** Every expression is built through smart constructors that flatten
  sums and products, fold numbers, collect like terms and powers, and sort operands. So
  `8*y*u*y*u*y^5*y*u*u^6*r*y*7*y` is simply `56*r*u^9*y^10`, with no hand-written
  pattern rules.
* **Small core.** Subtraction is `a + (-1)*b`, division is `a * b^-1`, `exp(x)` is `e^x`.
  The printer turns these back into `-`, `/` and `exp`, and its output can be pasted back in.
* **Values, not statements.** The parser produces a syntax tree; functions are looked up by
  name at evaluation time, so `sinx` is just a variable and adding a function is one match arm.

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

Type `help` for the list of commands.

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

## Parity with the Scala version

`tests/parity/scala_tests.txt` holds every input/expected pair from the Scala test
suite. Output is formatted differently (`2*x` instead of `2.0*x`), so a case passes when
the Rust answer is mathematically equal to the Scala one, checked at random points. For
`simplify` and `factor` the Rust answer must also be no larger. Known, deliberate
differences are listed in `tests/parity.rs` (for example, `acos(cos(y))` stays as it is,
because it only equals `y` for `0 <= y <= pi`).

## Status

- [x] Core algebra, calculus, simplification, matrices, parser, REPL
- [ ] Tensors and differential geometry (metrics, Christoffel, Riemann, Ricci, Einstein, Weyl, geodesics, ODE solver)
- [ ] Plotting (SVG and Plotly) and LaTeX output
- [ ] Jupyter kernel

## License

Apache 2.0, like the original.
