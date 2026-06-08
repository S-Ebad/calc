use crate::{
    err_fmt,
    function::Function,
    lexer::{Lexer, Token},
    operator::Operator,
    resolver::is_constant,
    user_function::UserFunction,
    write_args,
};

use std::{collections::HashMap, fmt};

#[derive(Debug, PartialEq, Clone)]
pub enum RawExpr {
    Number(f64),

    Identifier(String),

    Binary {
        op: Operator,
        lhs: Box<RawExpr>,
        rhs: Box<RawExpr>,
    },

    Unary {
        op: Operator,
        expr: Box<RawExpr>,
    },

    Postfix {
        op: Operator,
        expr: Box<RawExpr>,
    },

    // before its resolved
    Apply {
        name: String,
        args: Vec<RawExpr>,
    },

    Call {
        func: Function,
        args: Vec<RawExpr>,
    },

    UserCall {
        name: String, //hold name only, definition is somewhere else
        args: Vec<RawExpr>,
    },

    If {
        condition: Box<RawExpr>,
        then: Box<RawExpr>,
        else_: Box<RawExpr>,
    },
}

fn consume_args(
    lexer: &mut Lexer,
    funcs: &HashMap<String, UserFunction>,
) -> Result<Vec<RawExpr>, String> {
    let mut args: Vec<RawExpr> = Vec::new();

    if lexer.peek() != Some(&Token::LParen) {
        args.push(nud(lexer, funcs)?);
    } else {
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
            return Err("Parse Error: missing closing parenthesis ')'".to_string());
        }
    }

    Ok(args)
}

fn nud(lexer: &mut Lexer, funcs: &HashMap<String, UserFunction>) -> Result<RawExpr, String> {
    let expr = match lexer.next() {
        Some(Token::Number(num)) => RawExpr::Number(num),
        Some(Token::Identifier(name)) => {
            let is_func = Function::from(&name).is_some()
                || funcs.contains_key(&name)
                || lexer.peek() == Some(&Token::LParen);

            if !is_func {
                RawExpr::Identifier(name)
            } else {

                if matches!(lexer.peek(), Some(Token::Comma | Token::RParen)) {
                    return err_fmt!("Parse Error: '{}' is a function, not a value", name);
                }

                let args = consume_args(lexer, funcs)?;

                if let Some(func) = Function::from(&name) {
                    RawExpr::Call { func, args }
                } else if funcs.contains_key(&name) {
                    RawExpr::UserCall { name, args }
                } else {
                    RawExpr::Apply { name, args }
                }
            }
        }

        Some(Token::LParen) => {
            let lhs: RawExpr = parse_expression(lexer, 0, funcs)?;

            if lexer.next() != Some(Token::RParen) {
                return Err("Parse Error: missing closing parenthesis ')'".to_string());
            }

            lhs
        }

        Some(Token::Operator(op @ (Operator::Sub | Operator::Add))) => {
            let unary = if op == Operator::Sub {
                Operator::Neg
            } else {
                return nud(lexer, funcs);
            };

            // for chained unary (i.e --x)
            let expr = nud(lexer, funcs)?;

            RawExpr::Unary {
                op: unary,
                expr: Box::new(expr),
            }
        }

        Some(token) => return err_fmt!("Parse Error: '{}' Cannot start an expression", token),
        None => return Err("Parse Error: unexpected end of input".to_string()),
    };

    Ok(expr)
}

fn led(
    lexer: &mut Lexer,
    lhs: RawExpr,
    funcs: &HashMap<String, UserFunction>,
) -> Result<RawExpr, String> {
    let expr = match lexer.peek() {
        // Expression is done. Stop parsing
        Some(Token::RParen | Token::Comma) => lhs,

        Some(token @ (Token::LParen | Token::Identifier(_) | Token::Number(_))) => {
            if matches!(token, &Token::Number(_)) && matches!(lhs, RawExpr::Number(_)) {
                return Err("Parse Error: missing operator between expression".to_string());
            }

            let op = Operator::ImplicitMul;
            let (_, r_bp) = op.bp();
            let rhs = parse_expression(lexer, r_bp, funcs)?;

            RawExpr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }

        Some(Token::QuestionMark) => {
            lexer.next();

            let then = parse_expression(lexer, 0, funcs)?;
            if lexer.next() != Some(Token::Colon) {
                return Err("Parse Error: expected colon ':' after '?' ".to_string());
            }

            let else_ = parse_expression(lexer, 0, funcs)?;

            RawExpr::If {
                condition: Box::new(lhs),
                then: Box::new(then),
                else_: Box::new(else_),
            }
        }

        Some(Token::Operator(op)) => {
            let op = *op;
            lexer.next();

            if !op.is_postfix() {
                let (_, r_bp) = op.bp();
                let rhs = parse_expression(lexer, r_bp, funcs)?;

                RawExpr::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            } else {
                RawExpr::Postfix {
                    op,
                    expr: Box::new(lhs),
                }
            }
        }

        _ => unreachable!(),
    };

    Ok(expr)
}

fn parse_expression(
    lexer: &mut Lexer,
    min_bp: u8,
    funcs: &HashMap<String, UserFunction>,
) -> Result<RawExpr, String> {
    let mut lhs = nud(lexer, funcs)?;

    while let Some(token) = lexer.peek() {
        if token.left_bp() <= min_bp {
            break;
        }

        lhs = led(lexer, lhs, funcs)?;
    }

    Ok(lhs)
}

impl RawExpr {
    pub fn parse(
        mut lexer: Lexer,
        funcs: &HashMap<String, UserFunction>,
    ) -> Result<RawExpr, String> {
        if lexer.is_empty() {
            return Err("Parse Error: no expression to parse".to_string());
        }

        let expr = parse_expression(&mut lexer, 0, funcs)?;

        if let Some(token) = lexer.peek() {
            return Err(if matches!(token, Token::RParen) {
                "Parse Error: unexpected closing parenthesis ')'".to_string()
            } else {
                format!("Parse Error: unexpected token: {:?}", token)
            });
        }

        Ok(expr)
    }

    pub fn check_errors(&self) -> Result<(), String> {
        if let RawExpr::Binary {
            op: Operator::Equal,
            lhs,
            rhs: _,
        } = self
        {
            match lhs.as_ref() {
                RawExpr::Apply { name, .. } | RawExpr::Identifier(name) if name == "ans" => {
                    return Err("Parse Error: 'ans' is a reserved read-only variable".to_string());
                }

                RawExpr::Apply { name, .. } | RawExpr::Identifier(name)
                    if is_constant(name).is_some() =>
                {
                    return err_fmt!("Parse Error: attempt to redefine constant '{}'", name);
                }

                RawExpr::Call { func, .. } => {
                    return err_fmt!(
                        "Parse Error: attempt to redefine built-in function '{}'",
                        func
                    );
                }

                _ => (),
            }
        }

        Ok(())
    }
}

impl fmt::Display for RawExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RawExpr::Number(n) => write!(f, "{:.2}", n),
            RawExpr::Identifier(ident) => write!(f, "{}", ident),
            RawExpr::Binary { op, lhs, rhs } => write!(f, "{}(lhs={}, lhs={})", op, lhs, rhs),
            RawExpr::Unary { op, expr } | RawExpr::Postfix { op, expr } => {
                write!(f, "{}({})", op, expr)
            }
            RawExpr::If {
                condition,
                then,
                else_,
            } => write!(f, "If({}, then={}, else={})", condition, then, else_),

            RawExpr::Apply { name, args } => write_args!(f, "Apply", name, args),
            RawExpr::Call { func, args } => write_args!(f, "Call", func, args),
            RawExpr::UserCall { name, args } => write_args!(f, "UserCall", name, args),
        }
    }
}
