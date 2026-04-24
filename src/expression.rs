use std::fmt;

use crate::function::Function;
use crate::lexer::{Lexer, Token};
use crate::operator::Operator;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Number(f64),
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
}

#[derive(Debug)]
pub struct IntoFunction {
    pub name: String,
    pub params: Vec<Expression>,
    pub body: Expression,
}

impl IntoFunction {
    fn new(name: String, args: Vec<Expression>, body: Expression) -> Self {
        Self {
            name,
            params: args,
            body,
        }
    }
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
            if let Expression::Apply {
                identifier: _,
                args: _,
            } = lhs.as_ref()
            {
                return true;
            }
        }

        false
    }

    pub fn into_func(self) -> Option<IntoFunction> {
        match self {
            Expression::Binary {
                op: Operator::Equal,
                lhs,
                rhs,
            } => match *lhs {
                Expression::Apply { identifier, args } => {
                    Some(IntoFunction::new(identifier, args, *rhs))
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
                let res = args
                    .into_iter()
                    .map(|expr| expr.eval())
                    .collect::<Result<Vec<f64>, _>>()?;

                func.call(&res)
            }

            Expression::Apply {
                identifier: _,
                args: _,
            } => unreachable!(),
        }
    }

    pub fn parse(lexer: &mut Lexer) -> Result<Self, String> {
        if lexer.is_empty() {
            return Err("Invalid Expression: no expression to parse".to_string());
        }

        let expr = parse_expression(lexer, 0)?;

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

fn nud(lexer: &mut Lexer) -> Result<Expression, String> {
    Ok(match lexer.next() {
        Some(Token::Number(num)) => Expression::Number(num),
        Some(Token::Identifier(ident)) => {

            let res = match lexer.peek() {
                // call
                Some(Token::LParen) => {
                    lexer.next();
                    let mut args: Vec<Expression> = Vec::new();

                    if lexer.peek() != Some(&Token::RParen) {
                        loop {
                            args.push(parse_expression(lexer, 0)?);

                            if lexer.peek() == Some(&Token::Comma) {
                                lexer.next();
                            } else {
                                break;
                            }
                        }
                    }

                    if !matches!(lexer.peek(), Some(&Token::RParen)) {
                        return Err("Unclosed Parenthesis".to_string());
                    }
                    lexer.next();

                    Expression::Apply {
                        identifier: ident,
                        args,
                    }
                }

                Some(
                    Token::RParen
                    | Token::Comma
                    | Token::Operator(_)
                    | Token::Identifier(_)
                    | Token::Number(_),
                )
                | None => Expression::Identifier(ident),

                token => {
                    return Err(format!(
                        "Invalid Token: unexpected token after identifier: {:?}",
                        token
                    ));
                }
            };

            res
        }
        Some(Token::LParen) => {
            let lhs = parse_expression(lexer, 0)?;

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

            let expr = parse_expression(lexer, 100)?;
            Expression::Unary {
                op: unary,
                expr: Box::new(expr),
            }
        }

        t => return Err(format!("bad number: {:?}", t)),
    })
}

// pratt parser
fn parse_expression(lexer: &mut Lexer, min_bp: u8) -> Result<Expression, String> {
    let mut lhs = nud(lexer)?;

    loop {
        let (op, is_explicit) = match lexer.peek() {
            Some(Token::Operator(op)) => (op.clone(), true),
            Some(Token::LParen | Token::Identifier(_)) => (Operator::ImplicitMul, false),
            Some(Token::Number(_)) => {
                if matches!(lhs, Expression::Number(_)) {
                    return Err(
                        "Invalid Expression: missing operator between expressions".to_string()
                    );
                }

                (Operator::ImplicitMul, false)
            }
            Some(Token::RParen | Token::Comma) | None => break,
            t => return Err(format!("bad operator: {:?}", t)),
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

        let rhs = parse_expression(lexer, r_bp).map_err(|_| {
            format!(
                "Invalid Expression: expected expression after operator {:?}",
                op
            )
        })?;

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
            Expression::Number(num) => write!(f, "{}", num),
            Expression::Identifier(s) => write!(f, "{}", s),

            // Op(...operands, args?=[])
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
        }
    }
}
