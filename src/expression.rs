use std::collections::HashMap;
use std::fmt;

use crate::constant::Constant;
use crate::function::Function;
use crate::lexer::{Lexer, Token};
use crate::operator::Operator;
use crate::user_function::UserFunction;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Number(f64),
    Constant(Constant),

    Identifier(String),

    Binary {
        op: Operator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },

    Unary {
        op: Operator,
        expr: Box<Expression>,
    },

    Postfix {
        expr: Box<Expression>,
        op: Operator,
    },

    // before its resolved
    Apply {
        identifier: String,
        args: Vec<Expression>,
    },

    // function call
    Call {
        func: Function,
        args: Vec<Expression>,
    },

    UserCall {
        func: String, //hold name only, reference is somewhere else
        args: Vec<Expression>,
    },
}

impl Expression {
    pub fn is_assign(&self) -> bool {
        if let Expression::Binary {
            op: Operator::Equal,
            lhs,
            rhs: _,
        } = self
        {
            if let Expression::Identifier(_) = lhs.as_ref() {
                return true;
            }
        }

        false
    }

    pub fn into_assign(self) -> Option<(String, Expression)> {
        match self {
            Expression::Binary {
                op: Operator::Equal,
                lhs,
                rhs,
            } => match *lhs {
                Expression::Identifier(ident) => Some((ident, *rhs)),
                _ => None,
            },

            _ => None,
        }
    }

    pub fn is_func_def(&self) -> bool {
        if let Expression::Binary {
            op: Operator::Equal,
            lhs,
            rhs: _,
        } = self
        {
            // if its a call
            if let Expression::Apply {
                identifier: _,
                args: _,
            }
            | Expression::UserCall { func: _, args: _ } = lhs.as_ref()
            {
                // UserCall means function redefinition
                return true;
            }
        }

        false
    }

    pub fn into_func(self) -> Option<UserFunction> {
        match self {
            Expression::Binary {
                op: Operator::Equal,
                lhs,
                rhs,
            } => match *lhs {
                Expression::Apply { identifier, args } => {
                    Some(UserFunction::new(identifier, args, rhs))
                }
                Expression::UserCall { func, args } => {
                    Some(UserFunction::new(func, args, rhs))
                }
                _ => None,
            },

            _ => None,
        }
    }

    pub fn eval(&self) -> Result<f64, String> {
        match self {
            Expression::Identifier(s) => Err(format!("Unable to evaluate {:?}", s)),
            Expression::Number(num) => Ok(*num),
            Expression::Binary { op, lhs, rhs } => op.perform_op(lhs.eval()?, Some(rhs.eval()?)),
            Expression::Unary { op, expr } => op.perform_op(expr.eval()?, None),
            Expression::Postfix { expr, op } => op.perform_op(expr.eval()?, None),
            Expression::Call { func, args } => {
                let args = args
                    .into_iter()
                    .map(|expr| expr.eval())
                    .collect::<Result<Vec<f64>, _>>()?;

                func.call(&args)
            }

            Expression::Constant(_)
            | Expression::Apply {
                identifier: _,
                args: _,
            }
            | Expression::UserCall { func: _, args: _ } => unreachable!(),
        }
    }

    pub fn parse(lexer: &mut Lexer, funcs: &HashMap<String, UserFunction>) -> Result<Self, String> {
        if lexer.is_empty() {
            return Err("Invalid Expression: no expression to parse".to_string());
        }

        let expr = parse_expression(lexer, 0, funcs)?;

        if let Some(token) = lexer.peek() {
            return Err(if matches!(token, Token::RParen) {
                "Invalid Expression: unexpected closing parenthesis ')'".to_string()
            } else {
                format!("Invalid Expression: unexpected token {:?}", token)
            });
        }

        Ok(expr)
    }
}

// consumes function arguments (implicit or explicit)
fn consume_args(
    args: &mut Vec<Expression>,
    lexer: &mut Lexer,
    funcs: &HashMap<String, UserFunction>,
) -> Result<(), String> {
    if lexer.peek() == Some(&Token::LParen) {
        lexer.next();

        loop {
            args.push(parse_expression(lexer, 0, funcs)?);

            if lexer.peek() == Some(&Token::Comma) {
                lexer.next();
            } else {
                break;
            }
        }

        if !matches!(lexer.next(), Some(Token::RParen)) {
            return Err("Invalid Expression: missing closing parentheses ')'".to_string());
        }
    } else {
        args.push(nud(lexer, funcs)?);
    }

    Ok(())
}

fn nud(lexer: &mut Lexer, funcs: &HashMap<String, UserFunction>) -> Result<Expression, String> {
    Ok(match lexer.next() {
        Some(Token::Number(num)) => Expression::Number(num),
        Some(Token::Identifier(ident)) => {
            if let Ok(func) = Function::from(&ident) {
                let mut args: Vec<Expression> = Vec::new();

                consume_args(&mut args, lexer, funcs)?;

                Expression::Call { func, args }

            } else if let Ok(cons) = Constant::from(&ident) {
                Expression::Constant(cons)

            } else if let Some(func) = funcs.get(&ident) {
                let mut args: Vec<Expression> = Vec::new();
                consume_args(&mut args, lexer, funcs)?;

                Expression::UserCall {
                    func: func.name.clone(),
                    args: args,
                }

            } else if lexer.peek() == Some(&Token::LParen) {
                let mut args: Vec<Expression> = Vec::new();
                consume_args(&mut args, lexer, funcs)?;

                Expression::Apply {
                    identifier: ident,
                    args: args,
                }

            } else {
                Expression::Identifier(ident)

            }
        }

        Some(Token::LParen) => {
            let lhs: Expression = parse_expression(lexer, 0, funcs)?;

            // check closing parenthesis
            if lexer.next() != Some(Token::RParen) {
                return Err("Invalid Expression: missing closing parentheses ')'".to_string());
            }

            lhs
        }

        Some(Token::Operator(op @ (Operator::Sub | Operator::Add))) => {
            let unary = if op == Operator::Sub {
                Operator::Neg
            } else {
                Operator::Pos
            };

            let expr = parse_expression(lexer, 100, funcs)?;
            Expression::Unary {
                op: unary,
                expr: Box::new(expr),
            }
        }

        Some(t) => return Err(format!("Invalid Token: unexpected token {:?} at start of expression", t)),
        None => return Err("Invalid Expression: unexpected end of input".to_string()),
    })
}

// pratt parser
fn parse_expression(
    lexer: &mut Lexer,
    min_bp: u8,
    funcs: &HashMap<String, UserFunction>,
) -> Result<Expression, String> {
    let mut lhs = nud(lexer, funcs)?;

    loop {
        let (op, is_explicit) = match lexer.peek() {
            Some(Token::Operator(op)) => (op.clone(), true),
            Some(Token::LParen | Token::Identifier(_)) => (Operator::Mul, false),
            Some(Token::Number(_)) => {
                if matches!(lhs, Expression::Number(_)) {
                    return Err(
                        "Invalid Expression: missing operator between expressions".to_string()
                    );
                }

                (Operator::Mul, false)
            }
            Some(Token::RParen | Token::Comma) | None => break,

            Some(t) => return Err(format!("Invalid Operator: unexpected token {:?} following expression", t)),
        };

        let (l_bp, r_bp) = if is_explicit { op.bp() } else { (6, 6) };
        if l_bp < min_bp {
            break;
        }

        // consume if its an operator token
        if is_explicit {
            lexer.next();
        }

        // led
        // postfix exception
        if op.is_postfix() {
            lhs = Expression::Postfix {
                op,
                expr: Box::new(lhs),
            };

            continue;
        }

        let rhs = parse_expression(lexer, r_bp, funcs)?;

        lhs = Expression::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
    }

    Ok(lhs)
}

// print "sin(10)" as: "Apply(sin, [10])"
impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Number(num) => write!(f, "Num({})", num),
            Expression::Identifier(s) => write!(f, "Ident({})", s),
            Expression::Unary { op, expr } => {
                write!(f, "{}({})", op, expr)
            }
            Expression::Binary { op, lhs, rhs } => {
                write!(f, "{}({}, {})", op, lhs, rhs)
            }
            Expression::Postfix { expr, op } => {
                write!(f, "{}({})", op, expr)
            }
            Expression::Apply { identifier, args } => {
                let args_str = args
                    .iter()
                    .map(|a| format!("{}", a))
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "Apply({}, [{}])", identifier, args_str)
            }
            Expression::Call { func, args } => {
                let args_str = args
                    .iter()
                    .map(|a| format!("{}", a))
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "Call({}, [{}])", func, args_str)
            }
            Expression::UserCall { func, args } => {
                let args_str = args
                    .iter()
                    .map(|a| format!("{}", a))
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "UserCall({}, [{}])", func, args_str)
            }
            Expression::Constant(constant) => write!(f, "Const({})", constant),
        }
    }
}
