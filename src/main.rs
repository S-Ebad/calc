mod calc;
mod eval;
mod function;
mod parser;
mod token;

use std::{
  io::{self, Write},
  process,
};

use crate::calc::Calculator;

// this formats the number in 10 decimal place.
// So equations like 0.1+0.2 don't result in 0.30000000000000004 
fn format_num(n: f64) -> String {
  let mut s = format!("{:.10}", n);

  if s.contains('.') {
    while s.ends_with('0'){
      s.pop();
    }

    if s.ends_with('.') {
      s.pop();
    }
  }

  s
}

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
      Ok(ans) => println!("= {}", format_num(ans)),
      Err(err) => eprintln!("Error: {}", err),
    }
  }
}
