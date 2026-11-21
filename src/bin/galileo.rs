//! Galileo REPL: `galileo` for an interactive prompt, `galileo file.gg` to run a file.

use galileo_rs::{Output, Session};
use std::io::{self, BufRead, IsTerminal, Write};

fn main() {
    let mut s = Session::new();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(file) = args.first() {
        let src = std::fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("{}: {}", file, e);
            std::process::exit(1)
        });
        let out = s.run_to_string(&src);
        if !out.is_empty() {
            println!("{}", out);
        }
        return;
    }
    let interactive = io::stdin().is_terminal();
    let stdin = io::stdin();
    loop {
        if interactive {
            print!("galileo> ");
            io::stdout().flush().ok();
        }
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        for r in s.run(&line) {
            match r {
                Ok(Output::Exit) => return,
                Ok(o) => {
                    let t = o.render();
                    if !t.is_empty() {
                        println!("{}", t);
                    }
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    }
}
