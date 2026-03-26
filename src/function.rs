#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Function {
  // 1-arg functions
  Sin,
  Cos,
  Tan,
  Sqrt,
  Abs,
  Ln,
  Exp,
  Floor,
  Ceil,
  Round,
  Recip,
  Cbrt,
  Log,
  Pow,
  Max,
  Min,
}

impl Function {
  pub fn from(name: &str) -> Result<Self, String> {
    use Function as F;
    match name {
      "sin" => Ok(F::Sin),
      "cos" => Ok(F::Cos),
      "tan" => Ok(F::Tan),
      "sqrt" => Ok(F::Sqrt),
      "abs" => Ok(F::Abs),
      "ln" => Ok(F::Ln),
      "exp" => Ok(F::Exp),
      "floor" => Ok(F::Floor),
      "ceil" => Ok(F::Ceil),
      "round" => Ok(F::Round),
      "recip" => Ok(F::Recip),
      "cbrt" => Ok(F::Cbrt),
      "log" => Ok(F::Log),
      "max" => Ok(F::Max),
      "min" => Ok(F::Min),
      "pow" => Ok(F::Pow),
      _ => Err(format!("Invalid Function: {}", name)),
    }
  }

  pub fn get_function_name(&self) -> &'static str {
    use Function as F;
    match self {
      F::Sin => "sin",
      F::Cos => "cos",
      F::Tan => "tan",
      F::Sqrt => "sqrt",
      F::Abs => "abs",
      F::Ln => "ln",
      F::Exp => "exp",
      F::Floor => "floor",
      F::Ceil => "ceil",
      F::Round => "round",
      F::Recip => "recip",
      F::Cbrt => "cbrt",
      F::Log => "log",
      F::Max => "max",
      F::Min => "min",
      F::Pow => "pow",
    }
  }

  // returns arity of each function. 0xFF represents any number of arguments EXCEPT 0
  pub fn arity(&self) -> (usize, usize) {
    use Function as F;

    match self {
      F::Max | F::Min => (2, usize::MAX),
      F::Sqrt | F::Log => (1, 2),
      F::Pow => (2, 2),
      _ => (1, 1),
    }
  }

  pub fn call(&self, args: &[f64]) -> Result<f64, String> {
    use Function as F;

    let (min, max) = self.arity();
    if !(min..=max).contains(&args.len()) {
      // special case for usize::MAX or min==max
      let arity_str = if max == usize::MAX {
        format!("at least {}", min)
      } else if min == max {
        format!("{}", min)
      } else {
        format!("{}-{}", min, max)
      };

      return Err(format!(
        "Invalid Arguments: {} takes {} argument(s) but got {}",
        self.get_function_name(),
        arity_str,
        args.len()
      ));
    }

    let result = match self {
      F::Sin => args[0].sin(),
      F::Cos => args[0].cos(),
      F::Tan => args[0].tan(),
      F::Abs => args[0].abs(),
      F::Ln => args[0].ln(),
      F::Exp => args[0].exp(),
      F::Floor => args[0].floor(),
      F::Ceil => args[0].ceil(),
      F::Round => args[0].round(),
      F::Cbrt => args[0].cbrt(),
      F::Recip => {
        if args[0] == 0f64 {
          return Err(format!(
            "Invalid Expression: division by zero recip({0}) (1/{0})",
            args[0]
          ));
        }

        args[0].recip()
      }
      F::Sqrt => {
        let root = if args.len() == 2 { args[1] } else { 2.0 };

        args[0].powf(1.0 / root)
      }
      F::Log => {
        let base = if args.len() == 2 { args[1] } else { 10.0 };

        args[0].log(base)
      }

      F::Max => args
        .iter()
        .copied()
        .reduce(f64::max)
        .ok_or("Function Error: max function has thrown an error")?,
      F::Min => args
        .iter()
        .copied()
        .reduce(f64::min)
        .ok_or("Function Error: min function has thrown an error")?,
      F::Pow => args[0].powf(args[1]),
    };

    if result.is_nan() {
      return Err(format!(
        "Invalid Expression: {}({})",
        self.get_function_name(),
        args[0]
      ));
    }

    Ok(result)
  }
}
