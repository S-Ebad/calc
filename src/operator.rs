#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Add, // Binary+
    Sub, // Binary-
    Neg, // Unary-
    Pos, // Unary+
    Mul,
    Div,
    Pow,
    Fac, // Factorial
    Mod, // modulos
    Equal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Assoc {
    Left,
    Right,
}

impl Operator {
    // get operator from char. last_token is needed for Unary
    pub fn from(c: char) -> Result<Self, String> {
        match c {
            '+' => Ok(Operator::Add),
            '-' => Ok(Operator::Sub),
            '/' => Ok(Operator::Div),
            '*' => Ok(Operator::Mul),
            '^' => Ok(Operator::Pow),
            '!' => Ok(Operator::Fac),
            '%' => Ok(Operator::Mod),
            '=' => Ok(Operator::Equal),
            _ => Err(format!("Invalid Operator: {}", c)),
        }
    }

    // binding power
    pub fn bp(&self) -> (u8, u8) {
        match self {
            Operator::Equal => (1, 0),
            Operator::Add | Operator::Sub => (2, 3),
            Operator::Mul | Operator::Div | Operator::Mod => (4, 5),
            Operator::Neg | Operator::Pos => (6, 7),
            Operator::Pow => (8, 8),
            Operator::Fac => (100, 0),
        }
    }

    fn associativity(&self) -> Assoc {
        match self {
            Operator::Pow | Operator::Neg | Operator::Pos => Assoc::Right,
            _ => Assoc::Left,
        }
    }

    pub fn is_left_assoc(&self) -> bool {
        matches!(self.associativity(), Assoc::Left)
    }


    // perform operator. It'll perform the operator depending on if num2 is supplied or not
    pub fn perform_op(&self, num1: f64, num2: Option<f64>) -> Result<f64, String> {
        match num2 {
            Some(num2) => self.perform_infix(num1, num2),
            None => self.perform_postfix_prefix(num1),
        }
    }

    fn perform_postfix_prefix(&self, num1: f64) -> Result<f64, String> {
        use Operator as OP;

        match self {
            OP::Pos => Ok(num1),
            OP::Neg => Ok(-num1),
            OP::Fac => factorial(num1),

            _ => Err(format!(
                "Invalid Operator: {:?} is not a postfix/prefix operator",
                self
            )),
        }
    }

    pub fn is_postfix(&self) -> bool {
        matches!(self, Self::Fac)
    }

    fn perform_infix(&self, num1: f64, num2: f64) -> Result<f64, String> {
        use Operator as OP;

        let result = match self {
            OP::Add => num1 + num2,
            OP::Sub => num1 - num2,
            OP::Mul => num1 * num2,
            OP::Mod => num1 % num2,

            OP::Div => {
                if num2 == 0f64 {
                    return Err(format!(
                        "Invalid Expression: division by zero ({}/{})",
                        num1, num2
                    ));
                }

                num1 / num2
            }

            OP::Pow => {
                // num1 ^ -num2 where num1 is 0 is undefined
                if num1 == 0f64 && num2 < 0f64 {
                    return Err(format!(
                        "Invalid Expression: division by zero ({0}^{1} = 1/({0}^{2}) = 1 / {0})",
                        num1,
                        num2,
                        num2.abs()
                    ));
                }

                f64::powf(num1, num2)
            }

            _ => {
                return Err(format!(
                    "Invalid Token: {:?} Must be handled during parser",
                    self
                ));
            }
        };

        if result.is_nan() {
            return Err(format!(
                "Invalid Expression: {} {} {} is NaN",
                num1, self, num2
            ));
        }

        Ok(result)
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Operator as OP;
        let name = match self {
            OP::Add => "Add",
            OP::Sub => "Sub",
            OP::Neg => "USub",
            OP::Pos => "UAdd",
            OP::Mul => "Mul",
            OP::Div => "Div",
            OP::Pow => "Pow",
            OP::Fac => "Fac",
            OP::Mod => "Mod",
            OP::Equal => "Eq",
        };

        write!(f, "{}", name)
    }
}

fn factorial(n: f64) -> Result<f64, String> {
    if n < 0.0 || n.fract() != 0.0 {
        return Err(format!("Invalid Argument: factorial undefined for {}", n));
    }

    if n > 170.0 {
        return Err(format!("Invalid Argument: {}! is too large", n));
    }

    Ok((1..=n as u64).map(|x| x as f64).product())
}
