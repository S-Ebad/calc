use std::{iter::Peekable, str::Chars};

macro_rules! to_bool {
    ($b:expr) => {
        ($b as u8) as f64
    };
}

macro_rules! is_true {
    ($x:expr) => {
        ($x != 0.0)
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Add, // Binary+
    Sub, // Binary-
    Neg, // Unary-
    Pos, // Unary+
    Mul,
    ImplicitMul,
    Div,
    Pow,
    Fac, // Factorial
    Mod, // modulos
    Equal,

    IsEqual,      // ==
    NotEqual,     // !=
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=

    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Assoc {
    Left,
    Right,
}

// simple helper
fn is_next_token(iter: &mut Peekable<Chars>, val: Option<&char>) -> bool {
    iter.next();

    iter.peek() == val
}

impl Operator {
    // get operator from char. last_token is needed for Unary
    pub fn from(c: char, iter: &mut Peekable<Chars>) -> Result<Self, String> {
        match c {
            '&' if is_next_token(iter, Some(&'&')) => Ok(Operator::And),

            '|' if is_next_token(iter, Some(&'|')) => Ok(Operator::Or),

            '=' => {
                if is_next_token(iter, Some(&'=')) {
                    Ok(Operator::IsEqual)
                } else {
                    Ok(Operator::Equal)
                }
            }

            '>' => {
                if is_next_token(iter, Some(&'=')) {
                    Ok(Operator::GreaterEqual)
                } else {
                    Ok(Operator::GreaterThan)
                }
            }

            '<' => {
                if is_next_token(iter, Some(&'=')) {
                    Ok(Operator::LessEqual)
                } else {
                    Ok(Operator::LessThan)
                }
            }

            '!' => {
                if is_next_token(iter, Some(&'=')) {
                    Ok(Operator::NotEqual)
                } else {
                    Ok(Operator::Fac)
                }
            }

            '+' => Ok(Operator::Add),
            '-' => Ok(Operator::Sub),
            '/' => Ok(Operator::Div),
            '*' => Ok(Operator::Mul),
            '^' => Ok(Operator::Pow),
            '%' => Ok(Operator::Mod),
            _ => Err(format!("Invalid Operator: {}", c)),
        }
    }

    // binding power
    pub fn bp(&self) -> (u8, u8) {
        match self {
            Operator::Equal => (1, 0),

            Operator::Or => (1, 2),
            Operator::And => (3, 4),

            Operator::IsEqual => (5, 6),
            Operator::NotEqual => (5, 6),
            Operator::LessThan => (5, 6),
            Operator::GreaterThan => (5, 6),
            Operator::LessEqual => (5, 6),
            Operator::GreaterEqual => (5, 6),

            Operator::Add | Operator::Sub => (7, 8),
            Operator::Mul | Operator::Div | Operator::Mod => (9, 10),
            Operator::ImplicitMul => (11, 11),
            Operator::Neg | Operator::Pos => (11, 12),
            Operator::Pow => (13, 12),
            Operator::Fac => (14, 0),
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
            OP::Mul | OP::ImplicitMul => num1 * num2,
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

            OP::IsEqual => to_bool!(num1 == num2),
            OP::NotEqual => to_bool!(num1 != num2),
            OP::LessThan => to_bool!(num1 < num2),
            OP::GreaterThan => to_bool!(num1 > num2),
            OP::LessEqual => to_bool!(num1 <= num2),
            OP::GreaterEqual => to_bool!(num1 >= num2),

            OP::And => to_bool!(is_true!(num1) && is_true!(num2)),
            OP::Or => to_bool!(is_true!(num1) || is_true!(num2)),

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
            OP::ImplicitMul => "IMul",
            OP::Div => "Div",
            OP::Pow => "Pow",
            OP::Fac => "Fac",
            OP::Mod => "Mod",
            OP::Equal => "Eq",

            OP::IsEqual => "IsEqual",
            OP::NotEqual => "NotEqual",
            OP::LessThan => "LessThan",
            OP::GreaterThan => "GreaterThan",
            OP::LessEqual => "LessEqual",
            OP::GreaterEqual => "GreaterEqual",
            OP::And => "AND",
            OP::Or => "OR",
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
