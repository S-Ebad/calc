mod calc;
mod eval;
mod parser;
mod token;

use std::{
  io::{self, Write},
  process,
};

use crate::calc::Calculator;

fn main() {
  let mut buf = String::new();

  let mut calculator = Calculator::new();

  loop {
    buf.clear();
    print!("\n> ");
    let _ = io::stdout().flush();

    if let Err(err) = io::stdin().read_line(&mut buf) {
      eprintln!("Error: {}", err);

      process::exit(1);
    }

    if buf.ends_with('\n') {
      buf.pop();
    }

    if buf.is_empty() {
      break;
    }

    match calculator.solve(&buf) {
      Ok(ans) => println!("= {}", ans),
      Err(err) => eprintln!("Error: {}", err),
    }
  }
}
