//! Lexer and Pratt parser for the Galileo language.
//!
//! ```text
//! statement := name = expr | [a, b, ...] = expr | command | expr
//! expr      := range ; precedence from low to high:
//!              a:b:c   ||   &&   + -   * / \   unary -   ^ (right assoc)
//!              postfix: f(args)  A[i,j]  A'
//! matrix    := [a b c; d e f]   (elements separated by spaces or commas)
//! ```
//! Statements are separated by `;` or newlines; `//` starts a comment.

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Num(String),
    Ident(String),
    Str(String),
    Neg(Box<Node>),
    Bin(Op, Box<Node>, Box<Node>),
    Range(Box<Node>, Option<Box<Node>>, Box<Node>),
    Call(String, Vec<Node>),
    Index(Box<Node>, Vec<Node>),
    Matrix(Vec<Vec<Node>>),
    Transpose(Box<Node>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Solve,
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expr(Node),
    Assign(String, Node),
    MultiAssign(Vec<String>, Node),
    /// who, whos, clear, exit, ls, pwd, cd <dir>, help [name], load(<file>)
    Command(String, Option<String>),
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(String),
    Ident(String),
    Str(String),
    /// raw text argument of load(...), cd ..., help(...)
    Raw(String),
    Sym(&'static str),
    Sep, // ; or newline at top level
    Newline,
}

#[derive(Clone, Debug)]
struct Token {
    tok: Tok,
    space_before: bool,
}

const SYMS: [&str; 18] = [
    "&&", "||", "+", "-", "*", "/", "^", "\\", "'", "(", ")", "[", "]", ",", ";", ":", "=", "!",
];

/// Commands whose argument is taken as raw text
const RAW: [&str; 4] = ["load", "cd", "help", "metric.generate"];

fn lex(src: &str) -> Result<Vec<Token>, String> {
    let c: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out: Vec<Token> = vec![];
    let mut space = false;
    let mut depth = 0i32; // bracket/paren depth: newlines inside are row separators
    while i < c.len() {
        let ch = c[i];
        if ch == '/' && c.get(i + 1) == Some(&'/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if ch == '\n' {
            out.push(Token { tok: Tok::Newline, space_before: space });
            space = true;
            i += 1;
            continue;
        }
        if ch.is_whitespace() {
            space = true;
            i += 1;
            continue;
        }
        let start = i;
        let tok = if ch.is_ascii_digit() || (ch == '.' && c.get(i + 1).is_some_and(|d| d.is_ascii_digit())) {
            while i < c.len() && (c[i].is_ascii_digit() || c[i] == '.') {
                i += 1;
            }
            if i < c.len() && (c[i] == 'e' || c[i] == 'E') {
                let mut j = i + 1;
                if j < c.len() && (c[j] == '+' || c[j] == '-') {
                    j += 1;
                }
                if j < c.len() && c[j].is_ascii_digit() {
                    i = j;
                    while i < c.len() && c[i].is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            Tok::Num(c[start..i].iter().collect())
        } else if ch.is_alphabetic() || ch == '_' {
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_' || (c[i] == '.' && c.get(i + 1).is_some_and(|d| d.is_alphabetic()))) {
                i += 1;
            }
            let name: String = c[start..i].iter().collect();
            if RAW.contains(&name.as_str()) {
                out.push(Token { tok: Tok::Ident(name.clone()), space_before: space });
                space = false;
                // raw argument: (text) or a space-separated word
                let mut j = i;
                while j < c.len() && c[j] == ' ' {
                    j += 1;
                }
                if j < c.len() && c[j] == '(' {
                    let close = c[j..].iter().position(|&x| x == ')').ok_or("missing )")?;
                    let raw: String = c[j + 1..j + close].iter().collect();
                    out.push(Token { tok: Tok::Raw(raw.trim().to_string()), space_before: false });
                    i = j + close + 1;
                } else if name == "cd" && j < c.len() && c[j] != ';' && c[j] != '\n' {
                    let end = c[j..].iter().position(|&x| x == ';' || x == '\n').map_or(c.len(), |p| j + p);
                    out.push(Token { tok: Tok::Raw(c[j..end].iter().collect::<String>().trim().to_string()), space_before: true });
                    i = end;
                }
                continue;
            }
            Tok::Ident(name)
        } else if ch == '"' {
            i += 1;
            while i < c.len() && c[i] != '"' {
                i += 1;
            }
            if i >= c.len() {
                return Err("unterminated string".into());
            }
            i += 1;
            Tok::Str(c[start + 1..i - 1].iter().collect())
        } else {
            let rest: String = c[i..(i + 2).min(c.len())].iter().collect();
            let s = SYMS
                .iter()
                .find(|s| rest.starts_with(**s))
                .ok_or_else(|| format!("unexpected character '{}'", ch))?;
            i += s.len();
            match *s {
                "(" | "[" => depth += 1,
                ")" | "]" => depth -= 1,
                _ => {}
            }
            Tok::Sym(s)
        };
        out.push(Token { tok, space_before: space });
        space = false;
    }
    let _ = depth;
    Ok(out)
}

struct P {
    t: Vec<Token>,
    i: usize,
    /// inside [ ]: whitespace separates elements and newlines separate rows
    in_matrix: usize,
}

type R<T> = Result<T, String>;

impl P {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i).map(|t| &t.tok)
    }
    fn space_before(&self) -> bool {
        self.t.get(self.i).is_some_and(|t| t.space_before)
    }
    fn is(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Sym(x)) if *x == s)
    }
    fn eat(&mut self, s: &str) -> bool {
        if self.is(s) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, s: &str) -> R<()> {
        if self.eat(s) {
            Ok(())
        } else {
            Err(format!("expected '{}' {}", s, self.here()))
        }
    }
    fn here(&self) -> String {
        match self.peek() {
            None => "at end of input".into(),
            Some(t) => format!("near {}", describe(t)),
        }
    }
    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Some(Tok::Newline)) {
            self.i += 1;
        }
    }

    fn statement(&mut self) -> R<Stmt> {
        // commands
        if let Some(Tok::Ident(name)) = self.peek().cloned() {
            let next = self.t.get(self.i + 1).map(|t| t.tok.clone());
            let ends = matches!(next, None | Some(Tok::Sep) | Some(Tok::Newline) | Some(Tok::Sym(";")));
            match name.as_str() {
                "who" | "whos" | "clear" | "exit" | "quit" | "ls" | "pwd" if ends => {
                    self.i += 1;
                    return Ok(Stmt::Command(name, None));
                }
                "help" | "cd" | "load" => {
                    self.i += 1;
                    let arg = match self.peek() {
                        Some(Tok::Raw(r)) => {
                            let r = r.clone();
                            self.i += 1;
                            Some(r)
                        }
                        _ => None,
                    };
                    return Ok(Stmt::Command(name, arg));
                }
                _ => {}
            }
            if matches!(next, Some(Tok::Sym("="))) {
                self.i += 2;
                return Ok(Stmt::Assign(name, self.expr()?));
            }
        }
        // [a, b] = ...
        if self.is("[") {
            let save = self.i;
            if let Some(names) = self.try_name_list() {
                if self.eat("=") {
                    return Ok(Stmt::MultiAssign(names, self.expr()?));
                }
            }
            self.i = save;
        }
        Ok(Stmt::Expr(self.expr()?))
    }

    fn try_name_list(&mut self) -> Option<Vec<String>> {
        self.eat("[");
        let mut names = vec![];
        loop {
            match self.peek() {
                Some(Tok::Ident(n)) => {
                    names.push(n.clone());
                    self.i += 1;
                }
                _ => return None,
            }
            if self.eat(",") {
                continue;
            }
            if self.eat("]") {
                return Some(names);
            }
            return None;
        }
    }

    fn expr(&mut self) -> R<Node> {
        let a = self.binary(0)?;
        if self.is(":") {
            self.i += 1;
            let b = self.binary(0)?;
            if self.eat(":") {
                let c = self.binary(0)?;
                return Ok(Node::Range(Box::new(a), Some(Box::new(b)), Box::new(c)));
            }
            return Ok(Node::Range(Box::new(a), None, Box::new(b)));
        }
        Ok(a)
    }

    fn infix(&self) -> Option<(Op, u8, bool)> {
        // (op, precedence, right associative)
        let s = match self.peek() {
            Some(Tok::Sym(s)) => *s,
            _ => return None,
        };
        Some(match s {
            "||" => (Op::Or, 1, false),
            "&&" => (Op::And, 2, false),
            "+" => (Op::Add, 3, false),
            "-" => (Op::Sub, 3, false),
            "*" => (Op::Mul, 4, false),
            "/" => (Op::Div, 4, false),
            "\\" => (Op::Solve, 4, false),
            "^" => (Op::Pow, 6, true),
            _ => return None,
        })
    }

    fn binary(&mut self, min: u8) -> R<Node> {
        let mut lhs = self.unary()?;
        while let Some((op, prec, right)) = self.infix() {
            if prec < min {
                break;
            }
            self.i += 1;
            let rhs = if op == Op::Pow { self.unary_pow()? } else { self.binary(if right { prec } else { prec + 1 })? };
            lhs = Node::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// exponent: allows a sign (2^-1) and is right associative (2^3^2 = 2^9)
    fn unary_pow(&mut self) -> R<Node> {
        if self.eat("-") {
            return Ok(Node::Neg(Box::new(self.unary_pow()?)));
        }
        let base = self.postfix()?;
        if self.eat("^") {
            return Ok(Node::Bin(Op::Pow, Box::new(base), Box::new(self.unary_pow()?)));
        }
        Ok(base)
    }

    fn unary(&mut self) -> R<Node> {
        if self.eat("-") {
            // binds looser than ^ : -x^2 = -(x^2)
            let operand = self.binary(5)?;
            return Ok(Node::Neg(Box::new(operand)));
        }
        if self.eat("+") {
            return self.binary(5);
        }
        if self.eat("!") {
            let operand = self.binary(5)?;
            return Ok(Node::Call("not".into(), vec![operand]));
        }
        self.postfix()
    }

    fn postfix(&mut self) -> R<Node> {
        let mut e = self.primary()?;
        loop {
            if self.is("'") && !self.space_before() {
                self.i += 1;
                e = Node::Transpose(Box::new(e));
            } else if self.is("[") && !self.space_before() {
                self.i += 1;
                let save = self.in_matrix;
                self.in_matrix = 0;
                let mut idx = vec![self.expr()?];
                while self.eat(",") {
                    idx.push(self.expr()?);
                }
                self.in_matrix = save;
                self.expect("]")?;
                e = Node::Index(Box::new(e), idx);
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn primary(&mut self) -> R<Node> {
        let tok = self.peek().cloned().ok_or("unexpected end of input")?;
        self.i += 1;
        match tok {
            Tok::Num(s) => Ok(Node::Num(s)),
            Tok::Str(s) => Ok(Node::Str(s)),
            Tok::Ident(name) => {
                if let Some(Tok::Raw(r)) = self.peek().cloned() {
                    self.i += 1;
                    return Ok(Node::Call(name, vec![Node::Str(r)]));
                }
                if self.is("(") && !self.space_before() {
                    self.i += 1;
                    let save = self.in_matrix;
                    self.in_matrix = 0;
                    let mut args = vec![];
                    if !self.is(")") {
                        args.push(self.expr()?);
                        while self.eat(",") {
                            args.push(self.expr()?);
                        }
                    }
                    self.in_matrix = save;
                    self.expect(")")?;
                    return Ok(Node::Call(name, args));
                }
                Ok(Node::Ident(name))
            }
            Tok::Sym("(") => {
                let save = self.in_matrix;
                self.in_matrix = 0;
                let e = self.expr()?;
                self.in_matrix = save;
                self.expect(")")?;
                Ok(e)
            }
            Tok::Sym("[") => self.matrix(),
            t => Err(format!("unexpected {}", describe(&t))),
        }
    }

    fn matrix(&mut self) -> R<Node> {
        self.in_matrix += 1;
        let mut rows: Vec<Vec<Node>> = vec![];
        let mut row: Vec<Node> = vec![];
        loop {
            match self.peek() {
                None => return Err("missing ]".into()),
                Some(Tok::Sym("]")) => {
                    self.i += 1;
                    break;
                }
                Some(Tok::Sym(";")) | Some(Tok::Newline) => {
                    self.i += 1;
                    if !row.is_empty() {
                        rows.push(std::mem::take(&mut row));
                    }
                }
                Some(Tok::Sym(",")) => {
                    self.i += 1;
                }
                _ => row.push(self.expr()?),
            }
        }
        if !row.is_empty() {
            rows.push(row);
        }
        self.in_matrix -= 1;
        if rows.iter().any(|r| r.len() != rows[0].len()) {
            return Err("not all rows have the same size".into());
        }
        Ok(Node::Matrix(rows))
    }
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Num(s) | Tok::Ident(s) => format!("'{}'", s),
        Tok::Str(s) | Tok::Raw(s) => format!("\"{}\"", s),
        Tok::Sym(s) => format!("'{}'", s),
        Tok::Sep | Tok::Newline => "end of statement".into(),
    }
}

/// Parse a program: statements separated by `;` or newlines.
pub fn parse(src: &str) -> Result<Vec<Stmt>, String> {
    let toks = lex(src)?;
    // newlines outside brackets separate statements
    let mut t: Vec<Token> = vec![];
    let mut depth = 0i32;
    for tok in toks {
        match &tok.tok {
            Tok::Sym("(") | Tok::Sym("[") => depth += 1,
            Tok::Sym(")") | Tok::Sym("]") => depth -= 1,
            _ => {}
        }
        let sep = depth <= 0 && matches!(tok.tok, Tok::Newline | Tok::Sym(";"));
        t.push(if sep { Token { tok: Tok::Sep, space_before: tok.space_before } } else { tok });
    }
    let mut p = P { t, i: 0, in_matrix: 0 };
    let mut out = vec![];
    loop {
        while matches!(p.peek(), Some(Tok::Sep)) {
            p.i += 1;
        }
        if p.peek().is_none() {
            break;
        }
        out.push(p.statement()?);
        p.skip_newlines();
        match p.peek() {
            None => break,
            Some(Tok::Sep) => continue,
            Some(_) => return Err(format!("unexpected input {}", p.here())),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn precedence() {
        let s = parse("-x^2+3*y").unwrap();
        assert_eq!(
            s[0],
            Stmt::Expr(Node::Bin(
                Op::Add,
                Box::new(Node::Neg(Box::new(Node::Bin(
                    Op::Pow,
                    Box::new(Node::Ident("x".into())),
                    Box::new(Node::Num("2".into()))
                )))),
                Box::new(Node::Bin(Op::Mul, Box::new(Node::Num("3".into())), Box::new(Node::Ident("y".into()))))
            ))
        );
    }
    #[test]
    fn statements_and_matrices() {
        let s = parse("A=[4 5 6;6 5 4];norm(A-A,1)\n[L,U]=lu(A)").unwrap();
        assert_eq!(s.len(), 3);
        assert!(matches!(&s[0], Stmt::Assign(n, Node::Matrix(rows)) if n == "A" && rows.len() == 2));
        assert!(matches!(&s[2], Stmt::MultiAssign(n, _) if n.len() == 2));
        assert!(matches!(&parse("[a -b]").unwrap()[0], Stmt::Expr(Node::Matrix(r)) if r[0].len() == 1));
        assert!(matches!(&parse("load(examples/a.gg)").unwrap()[0], Stmt::Command(c, Some(p)) if c == "load" && p == "examples/a.gg"));
    }
}
