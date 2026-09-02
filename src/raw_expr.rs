use crate::{
    constant::Constant,
    err_fmt,
    errors::{ParseError, ParseErrorKind, Span, render_error},
    function::Function,
    lexer::{Lexer, Token, TokenKind},
    operator::Operator,
    user_function::UserFunction,
    write_args,
};

use std::{collections::HashMap, fmt};

#[derive(Debug, PartialEq, Clone)]
pub enum RawExprKind {
    Number(f64),
    Constant(Constant),
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

#[derive(Debug, PartialEq, Clone)]
pub struct RawExpr {
    pub(crate) kind: RawExprKind,
    pub(crate) span: Span,
}

impl RawExpr {
    pub fn new(kind: RawExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn kind(&self) -> &RawExprKind {
        &self.kind
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}

impl std::ops::DerefMut for RawExpr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}

impl std::ops::Deref for RawExpr {
    type Target = RawExprKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

fn consume_args(
    lexer: &mut Lexer,
    funcs: &HashMap<String, UserFunction>,
) -> Result<Vec<RawExpr>, ParseError> {
    // no parenthesis. i.e: sin10
    let open = lexer.peek_token();

    let Some(open) = open else {
        let span = Span::new(lexer.end_pos(), lexer.end_pos());
        return Err(ParseError::new(
            ParseErrorKind::UnexpectedEndOfInput,
            Some(span),
        ));
    };

    if !matches!(open.kind, TokenKind::LParen) {
        return Ok(vec![nud(lexer, funcs)?]);
    }

    let open_span = open.span;
    lexer.next();

    // empty arguments. i.e: sin()
    if lexer.peek() == Some(&TokenKind::RParen) {
        lexer.next();
        return Ok(vec![]);
    }

    let mut args: Vec<RawExpr> = Vec::new();

    loop {
        args.push(parse_expression(lexer, 0, funcs)?);

        if lexer.peek() == Some(&TokenKind::Comma) {
            let comma = lexer.next_token();

            // trailing comma
            if matches!(lexer.peek(), Some(&TokenKind::RParen)) {
                return Err(ParseError::with_note(
                    ParseErrorKind::TrailingComma,
                    Some(comma.unwrap().span),
                    "remove the ',' or add another argument/parameter".to_string(),
                ));
            }
        } else {
            break;
        }
    }

    if !matches!(lexer.next(), Some(TokenKind::RParen)) {
        return Err(ParseError::new(
            ParseErrorKind::MissingClosingParenthesis,
            Some(open_span),
        ));
    }

    Ok(args)
}

fn nud(lexer: &mut Lexer, funcs: &HashMap<String, UserFunction>) -> Result<RawExpr, ParseError> {
    let expr = match lexer.next_token() {
        Some(Token {
            kind: TokenKind::Number(num),
            span,
        }) => RawExpr::new(RawExprKind::Number(num), span),

        Some(Token {
            kind: TokenKind::Constant(constant),
            span,
        }) => RawExpr::new(RawExprKind::Constant(constant), span),

        Some(Token {
            kind: TokenKind::Identifier(name),
            span,
        }) => {
            let is_func = Function::from(&name).is_some()
                || funcs.contains_key(&name)
                || lexer.peek() == Some(&TokenKind::LParen);

            let raw_expr = if !is_func {
                RawExprKind::Identifier(name)
            } else {
                if matches!(lexer.peek(), Some(TokenKind::Comma | TokenKind::RParen)) {
                    let note = format!("call it with parentheses, e.g. '{name}()'");
                    let kind = ParseErrorKind::FunctionUsedAsValue(name);

                    return Err(ParseError::with_note(kind, Some(span), note));
                }

                let args = consume_args(lexer, funcs)?;

                if let Some(func) = Function::from(&name) {
                    RawExprKind::Call { func, args }
                } else if funcs.contains_key(&name) {
                    RawExprKind::UserCall { name, args }
                } else {
                    RawExprKind::Apply { name, args }
                }
            };

            RawExpr::new(raw_expr, span)
        }

        Some(Token {
            kind: TokenKind::LParen,
            span,
        }) => {
            let lhs: RawExpr = parse_expression(lexer, 0, funcs)?;

            match lexer.next_token() {
                Some(Token {
                    kind: TokenKind::RParen,
                    span: token_span,
                }) => {
                    let new_span = Span::merge(span, token_span);

                    RawExpr::new(lhs.kind, new_span)
                }

                _ => {
                    return Err(ParseError::new(
                        ParseErrorKind::MissingClosingParenthesis,
                        Some(span),
                    ));
                }
            }
        }

        Some(Token {
            kind: TokenKind::Operator(op @ (Operator::Sub | Operator::Add)),
            span,
        }) => {
            let unary = if op == Operator::Sub {
                Operator::Neg
            } else {
                return nud(lexer, funcs);
            };

            // for chained unary (i.e --x)
            let expr = nud(lexer, funcs)?;

            let raw_expr_kind = RawExprKind::Unary {
                op: unary,
                expr: Box::new(expr),
            };

            RawExpr::new(raw_expr_kind, span)
        }

        Some(token) => {
            let span = token.span;

            return Err(ParseError::new(
                ParseErrorKind::CannotStartExpression(token),
                Some(span),
            ));
        }

        None => {
            let span = Span::new(lexer.end_pos(), lexer.end_pos());
            return Err(ParseError::with_note(
                ParseErrorKind::UnexpectedEndOfInput,
                Some(span),
                "expected an expression after this".to_string(),
            ));
        }
    };

    Ok(expr)
}

fn led(
    lexer: &mut Lexer,
    lhs: RawExpr,
    funcs: &HashMap<String, UserFunction>,
) -> Result<RawExpr, ParseError> {
    let token = lexer.peek_token();
    let expr = match token {
        Some(Token {
            kind: TokenKind::RParen | TokenKind::Comma,
            span: _,
        }) => lhs,

        Some(Token {
            kind: TokenKind::Dot,
            span,
        }) => {
            let span = *span;
            lexer.next();

            let name = match lexer.next_token() {
                Some(Token {
                    kind: TokenKind::Identifier(name),
                    ..
                }) => name,
                Some(other) => {
                    let span = other.span;
                    let kind = ParseErrorKind::ExpectedMethodName(Some(other));

                    return Err(ParseError::new(kind, Some(span)));
                }

                None => {
                    let span = Span::new(lexer.end_pos(), lexer.end_pos());

                    return Err(ParseError::new(
                        ParseErrorKind::ExpectedMethodName(None),
                        Some(span),
                    ));
                }
            };

            let mut args = consume_args(lexer, funcs)?;
            args.insert(0, lhs);

            let raw_expr_kind = if let Some(func) = Function::from(&name) {
                RawExprKind::Call { func, args }
            } else if funcs.contains_key(&name) {
                RawExprKind::UserCall { name, args }
            } else {
                RawExprKind::Apply { name, args }
            };

            RawExpr::new(raw_expr_kind, span)
        }

        Some(Token {
            kind:
                token @ (TokenKind::LParen
                | TokenKind::Identifier(_)
                | TokenKind::Number(_)
                | TokenKind::Constant(_)),
            span,
        }) => {
            if matches!(token, TokenKind::Number(_)) && matches!(lhs.kind, RawExprKind::Number(_)) {
                let span = span.merge(*lhs.span());

                return Err(ParseError::new(ParseErrorKind::MissingOperator, Some(span)));
            }

            let op = Operator::ImplicitMul;
            let (_, r_bp) = op.bp();
            let rhs = parse_expression(lexer, r_bp, funcs)?;

            let span = Span::merge(*lhs.span(), *rhs.span());
            let raw_expr_kind = RawExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };

            RawExpr::new(raw_expr_kind, span)
        }

        Some(Token {
            kind: TokenKind::QuestionMark,
            span,
        }) => {
            let span = *span;
            lexer.next();

            let then = parse_expression(lexer, 0, funcs)?;
            if lexer.next() != Some(TokenKind::Colon) {
                let span = span.merge(*then.span());

                return Err(ParseError::with_note(
                    ParseErrorKind::ExpectedColonAfterQuestionMark,
                    Some(span),
                    "add a ':' followed by the else expression".into(),
                ));
            }

            let else_ = parse_expression(lexer, 0, funcs)?;

            let span = Span::merge(*lhs.span(), *then.span());

            let raw_expr_kind = RawExprKind::If {
                condition: Box::new(lhs),
                then: Box::new(then),
                else_: Box::new(else_),
            };

            RawExpr::new(raw_expr_kind, span)
        }

        Some(Token {
            kind: TokenKind::Operator(op),
            span,
        }) => {
            let op = *op;
            let span = *span;

            lexer.next();

            let raw_expr_kind: RawExprKind;
            let raw_expr_span: Span;
            if !op.is_postfix() {
                let (_, r_bp) = op.bp();
                let rhs = parse_expression(lexer, r_bp, funcs)?;

                raw_expr_span = span.merge(*rhs.span()).merge(*lhs.span());
                raw_expr_kind = RawExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            } else {
                raw_expr_span = span.merge(*lhs.span());
                raw_expr_kind = RawExprKind::Postfix {
                    op,
                    expr: Box::new(lhs),
                };
            }

            RawExpr::new(raw_expr_kind, raw_expr_span)
        }

        _ => unreachable!(),
    };

    Ok(expr)
}

fn parse_expression(
    lexer: &mut Lexer,
    min_bp: u8,
    funcs: &HashMap<String, UserFunction>,
) -> Result<RawExpr, ParseError> {
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
    ) -> Result<RawExpr, ParseError> {
        if lexer.is_empty() {
            return Err(ParseError::new(ParseErrorKind::EmptyExpression, None));
        }

        let expr = parse_expression(&mut lexer, 0, funcs)?;

        if let Some(token) = lexer.next_token() {
            let span;
            let kind = if matches!(token.kind, TokenKind::RParen) {
                span = token.span;

                ParseErrorKind::ExtraClosingParenthesis
            } else {
                span = token.span;

                ParseErrorKind::UnexpectedToken(token)
            };

            return Err(ParseError::new(kind, Some(span)));
        }

        Ok(expr)
    }

    pub fn check_errors(&self) -> Result<(), String> {
        if let RawExprKind::Binary {
            op: Operator::Equal,
            lhs,
            rhs: _,
        } = &self.kind
        {
            match lhs.as_ref() {
                RawExpr {
                    kind: RawExprKind::Apply { name, .. } | RawExprKind::Identifier(name),
                    ..
                } if name == "ans" => {
                    return Err("Parse Error: 'ans' is a reserved read-only variable".to_string());
                }

                RawExpr {
                    kind: RawExprKind::Constant(constant),
                    ..
                } => {
                    return err_fmt!("Parse Error: attempt to redefine constant '{}'", constant);
                }

                RawExpr {
                    kind: RawExprKind::Call { func, .. },
                    ..
                } => {
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

impl fmt::Display for RawExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Identifier(ident) => write!(f, "{}", ident),
            Self::Binary { op, lhs, rhs } => {
                let (my_left, my_right) = op.bp();

                let lhs_str = match lhs.kind {
                    Self::Binary { op: child_op, .. } if child_op.bp().0 <= my_left => {
                        format!("({})", lhs.kind)
                    }
                    _ => format!("{}", lhs.kind),
                };

                let rhs_str = match rhs.kind {
                    Self::Binary { op: child_op, .. } if child_op.bp().0 < my_right => {
                        format!("({})", rhs.kind)
                    }
                    _ => format!("{}", rhs.kind),
                };

                write!(f, "{} {} {}", lhs_str, op, rhs_str)
            }
            Self::Unary { op, expr } => {
                write!(f, "{}({})", op, expr.kind)
            }
            Self::Postfix { op, expr } => {
                write!(f, "({}){}", expr.kind, op)
            }
            Self::If {
                condition,
                then,
                else_,
            } => write!(f, "{} ? {} : {}", condition.kind, then.kind, else_.kind),

            Self::Apply { name, args } => write_args!(f, name, args),
            Self::Call { func, args } => write_args!(f, func, args),
            Self::UserCall { name, args } => write_args!(f, name, args),
            Self::Constant(constant) => write!(f, "{}", constant),
        }
    }
}
