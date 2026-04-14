mod calc;
mod eval;
mod function;
mod operator;
mod parser;
mod token;

use crate::calc::Calculator;
use rustyline::{DefaultEditor, error::ReadlineError};
use std::time::Instant;

fn time<F>(func: F, expr: &str) -> Result<String, String>
where
    F: FnOnce(&str) -> Result<f64, String>,
{
    let start = Instant::now();
    let res = func(expr)?;
    let dur = start.elapsed();

    Ok(format!("{} ({:0?})", res, dur))
}

fn main() {
    let mut calculator = Calculator::new();
    let mut rl = DefaultEditor::new().expect("Default Editor initialization failed");

    loop {
        let input;
        match rl.readline("> ") {
            Ok(ok) => input = ok,

            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }

        let _ = rl.add_history_entry(&input);

        match time(|s| calculator.solve(s), &input) {
            Ok(ans) => println!("= {}", ans),
            Err(err) => eprintln!("Error: {}", err),
        }

        println!();
    }
}
