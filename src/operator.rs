use crate::err_fmt;

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

impl Operator {
    // get operator from char. next is needed for comparisons.
    // Returns whether the operator and whether should consume lookahead or not
    pub fn from(c: char, next: Option<char>) -> Option<(Self, bool)> {
        match c {
            '&' if matches!(next, Some('&')) => Some((Operator::And, true)),
            '|' if matches!(next, Some('|')) => Some((Operator::Or, true)),
            '=' if matches!(next, Some('=')) => Some((Operator::IsEqual, true)),
            '>' if matches!(next, Some('=')) => Some((Operator::GreaterEqual, true)),
            '<' if matches!(next, Some('=')) => Some((Operator::LessEqual, true)),
            '!' if matches!(next, Some('=')) => Some((Operator::NotEqual, true)),

            '=' => Some((Operator::Equal, false)),
            '>' => Some((Operator::GreaterThan, false)),
            '<' => Some((Operator::LessThan, false)),
            '!' => Some((Operator::Fac, false)),
            '+' => Some((Operator::Add, false)),
            '-' => Some((Operator::Sub, false)),
            '/' => Some((Operator::Div, false)),
            '*' => Some((Operator::Mul, false)),
            '^' => Some((Operator::Pow, false)),
            '%' => Some((Operator::Mod, false)),
            _ => None,
        }
    }

    // binding power
    pub fn bp(&self) -> (u8, u8) {
        match self {
            Operator::Equal => (1, 0),

            Operator::Or => (3, 4),
            Operator::And => (5, 6),

            Operator::IsEqual | Operator::NotEqual => (7, 8),

            Operator::LessThan
            | Operator::GreaterThan
            | Operator::LessEqual
            | Operator::GreaterEqual => (9, 10),

            Operator::Add | Operator::Sub => (11, 12),
            Operator::Mul | Operator::Div | Operator::Mod => (13, 14),
            Operator::ImplicitMul => (15, 15),
            Operator::Neg | Operator::Pos => (15, 16),
            Operator::Pow => (17, 16),
            Operator::Fac => (18, 0),
        }
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

            _ => err_fmt!("Eval Error: {} is not a postfix/prefix operator", self),
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
                    return err_fmt!(
                        "Eval Error: {}/{} is undefined (division by zero)",
                        num1,
                        num2
                    );
                }

                num1 / num2
            }

            OP::Pow => {
                // num1 ^ -num2 where num1 is 0 is undefined
                if num1 == 0f64 && num2 < 0f64 {
                    return err_fmt!(
                        "Eval Error: {0}^{1} is undefined (division by zero: 1/{0}^{2})",
                        num1,
                        num2,
                        num2.abs()
                    );
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

            OP::Equal => {
                return Err(
                    "Eval Error: '=' can only be used at the top level for assignment".to_string(),
                );
            }
            _ => unreachable!(),
        };

        if result.is_nan() {
            return err_fmt!("Eval Error: {} {} {} is undefined", num1, self, num2);
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
            OP::Neg => "-",
            OP::Pos => "+",
            OP::Mul => "*",
            OP::ImplicitMul => "*",
            OP::Div => "/",
            OP::Pow => "^",
            OP::Fac => "!",
            OP::Mod => "%",
            OP::Equal => "=",

            OP::IsEqual => "==",
            OP::NotEqual => "!=",
            OP::LessThan => "<",
            OP::GreaterThan => ">",
            OP::LessEqual => "<=",
            OP::GreaterEqual => ">=",
            OP::And => "&&",
            OP::Or => "||",
        };

        write!(f, "{}", name)
    }
}

fn factorial(n: f64) -> Result<f64, String> {
    if n < 0.0 || n.fract() != 0.0 {
        return err_fmt!("Eval Error: factorial is undefined for {}", n);
    }

    if n > 170.0 {
        return err_fmt!("Eval Error: {}! is too large (max is 170!)", n);
    }

    Ok((1..=n as u64).map(|x| x as f64).product())
}
