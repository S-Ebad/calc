mod parser;
mod token;

use std::{
  io::{self, Write},
  process,
};

fn main() {
  let mut buf = String::new();

  loop {
    buf.clear();
    print!("\n> ");
    let _ = io::stdout().flush();

    if let Err(err) = io::stdin().read_line(&mut buf) {
      eprintln!("Error: {}", err);

      process::exit(1);
    }

    if buf.is_empty() {
      break;
    }

    let tokens = token::tokenize(&buf);

    match parser::parse(tokens) {
      Ok(_) => {}
      Err(err) => eprintln!("Error: {}", err),
    };
  }
}
