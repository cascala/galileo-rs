//! Galileo: `galileo` for an interactive prompt, `galileo file.gg` to run a file,
//! `galileo --kernel` to serve the notebook kernel protocol on stdin/stdout.

use galileo_rs::{Output, Session};
use std::io::{self, BufRead, IsTerminal, Write};

/// Print results; statements ending in ; stay quiet. Returns false on exit.
fn report(s: &mut Session, src: &str) -> bool {
    for (r, quiet, notes) in s.run_notes(src) {
        match r {
            Ok(Output::Exit) => return false,
            Ok(o) => {
                let t = o.render();
                if !quiet && !t.is_empty() {
                    println!("{}", t);
                }
                // explanations as comments, so the output stays Galileo source
                let c = galileo_rs::eval::comment_notes(&notes);
                if !quiet && !c.is_empty() {
                    println!("{}", c);
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }
    true
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // deep expressions recurse deeply: give the engine a large stack
    let worker = std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(move || run(args))
        .expect("thread");
    let code = worker.join().unwrap_or(1);
    std::process::exit(code);
}

fn run(args: Vec<String>) -> i32 {
    match args.first().map(|s| s.as_str()) {
        Some("--kernel") => {
            galileo_rs::kernel::serve();
            0
        }
        Some("-h") | Some("--help") => {
            println!("usage: galileo [file.gg] | galileo --kernel | galileo --version");
            0
        }
        Some("--version") => {
            println!("galileo {}", env!("CARGO_PKG_VERSION"));
            0
        }
        Some(file) => {
            let src = match std::fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}: {}", file, e);
                    return 1;
                }
            };
            report(&mut Session::new(), &src);
            0
        }
        None => {
            let mut s = Session::new();
            let interactive = io::stdin().is_terminal();
            if interactive {
                println!(
                    "Galileo {} (type help for commands, exit to quit)",
                    env!("CARGO_PKG_VERSION")
                );
            }
            let stdin = io::stdin();
            loop {
                if interactive {
                    print!("galileo> ");
                    io::stdout().flush().ok();
                }
                let mut line = String::new();
                if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
                    return 0;
                }
                if !report(&mut s, &line) {
                    return 0;
                }
            }
        }
    }
}
