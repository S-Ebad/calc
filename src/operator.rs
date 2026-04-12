use crate::token::Token;

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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Assoc {
    Left,
    Right,
}

impl Operator {
    // get operator from char. last_token is needed for Unary
    pub fn from(c: char, last_token: Option<&Token>) -> Result<Self, String> {
        // is unary?
        if is_unary(c, last_token) {
            return if c == '-' {
                Ok(Operator::Neg)
            } else {
                Ok(Operator::Pos)
            };
        }

        match c {
            '+' => Ok(Operator::Add),
            '-' => Ok(Operator::Sub),
            '/' => Ok(Operator::Div),
            '*' => Ok(Operator::Mul),
            '^' => Ok(Operator::Pow),
            '!' => Ok(Operator::Fac),
            '%' => Ok(Operator::Mod),
            _ => Err(format!("Invalid Operator: {}", c)),
        }
    }

    pub fn precedence(&self) -> u16 {
        match self {
            Operator::Add | Operator::Sub => 1,
            Operator::Mul | Operator::Div | Operator::Mod => 2,
            Operator::Neg | Operator::Pos => 3,
            Operator::Pow => 4,
            Operator::Fac => 5,
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

    pub fn perform_op(&self, num1: f64, num2: f64) -> Result<f64, String> {
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
                if num1 == 0f64 && num2 <= 0f64 {
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
            OP::Add => "+",
            OP::Sub => "-",
            OP::Neg => "U-",
            OP::Pos => "U+",
            OP::Mul => "*",
            OP::Div => "/",
            OP::Pow => "^",
            OP::Fac => "!",
            OP::Mod => "%",
        };

        write!(f, "{}", name)
    }
}

fn is_unary(c: char, last_token: Option<&Token>) -> bool {
    (c == '-' || c == '+')
        && matches!(
            last_token,
            None | Some(Token::LParen | Token::Operator(_) | Token::Comma)
        )
}
