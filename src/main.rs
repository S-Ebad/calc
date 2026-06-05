use calc::calc::Calculator;
use rustyline::{DefaultEditor, error::ReadlineError};

fn repl(mut calculator: Calculator) {
    let mut rl = DefaultEditor::new().expect("Default Editor initialization failed");

    loop {
        let input = match rl.readline("> ") {
            Ok(ok) => ok,

            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        };

        let _ = rl.add_history_entry(&input);

        if input.trim() == "exit" {
            break;
        }

        match calculator.solve(&input) {
            Ok(Some(ans)) => println!("= {}", ans),
            Err(err) => eprintln!("{}", err),
            Ok(None) => (), // for function definitions.
        }

        println!();
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut calculator = Calculator::new();

    if args.len() > 1 {
        for expr in &args[1..] {
            match calculator.solve(expr.as_str()) {
                Ok(Some(ans)) => println!(" = {}", ans),
                Ok(None) => (),
                Err(err) => {
                    eprintln!("{}", err);
                    break;
                }
            }
        }
    } else {
        repl(calculator);
    }
}
