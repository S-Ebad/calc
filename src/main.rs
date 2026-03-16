mod eval;
mod parser;
mod token;

use std::{
  io::{self, Write},
  process,
};

use crate::{eval::eval, parser::parse, token::tokenize};

fn main() {
  let mut buf = String::new();
  let mut ans: Option<f64> = None;

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

    match tokenize(&buf)
      .and_then(|tokens| parse(tokens, ans))
      .and_then(|tokens| eval(tokens))
    {
      Ok(sum) => {
        ans = Some(sum);

        println!("= {}", sum);
      }
      Err(err) => eprintln!("Error: {}", err),
    }
  }
}
