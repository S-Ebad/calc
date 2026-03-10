mod token;

use std::{io::{self, Write}, process};

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

    let tokens = token::tokenize(&buf);

    for token in tokens.iter() {
      println!("{:?}", token);
    }
  }
}
