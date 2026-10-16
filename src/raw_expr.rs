use crate::{
    constant::Constant,
    errors::{ParseError, ParseErrorKind, Span},
    expr::Depth,
    function::Function,
    lexer::{Lexer, Token, TokenKind},
    operator::Operator,
    user_function::UserFunction,
    write_args,
};

use std::{collections::HashMap, fmt, rc::Rc};

#[derive(Debug, PartialEq, Clone)]
pub enum RawExprKind {
    Number(f64),
    Constant(Constant),
    Identifier(String),

    /// a reference to the enclosing function's own parameter, bound to a slot
    /// at definition time. the name is kept only so diagnostics can quote it,
    /// shared rather than owned because a body is cloned on every call
    Param {
        slot: usize,
        name: Rc<str>,
    },

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

    /// a '.' call to a function not known yet. unlike Apply it can only ever
    /// be a call: UFCS is for functions, so a value of the name is an error
    /// rather than something to multiply
    Method {
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

pub enum Statement {
    FuncDef(UserFunction),
    Assign(String, RawExpr),
    Eval(RawExpr),
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

/// What the parser needs to know about names to tell a call from a value.
struct ParseCtx<'a> {
    funcs: &'a HashMap<String, UserFunction>,

    // the parameters of the definition this line is, if it is one. inside the
    // body they shadow any function of the same name, the same way they shadow
    // globals when the body runs
    params: &'a [String],
}

impl ParseCtx<'_> {
    fn is_param(&self, name: &str) -> bool {
        self.params.iter().any(|param| param == name)
    }

    /// Whether a name is known to be a function here, rather than a value.
    fn is_func(&self, name: &str) -> bool {
        !self.is_param(name) && (Function::from(name).is_some() || self.funcs.contains_key(name))
    }

    /// The node for applying a name to arguments.
    ///
    /// A parameter applied to something is a value being multiplied, exactly
    /// as a variable would be, so it is left for the resolver to read that way.
    fn call(&self, name: String, args: Vec<RawExpr>) -> RawExprKind {
        if self.is_param(&name) {
            return RawExprKind::Apply { name, args };
        }

        self.global_call(name, args)
    }

    /// The node for a '.' call, which has to be a function.
    ///
    /// Whether a name nobody knows yet is a function is left to the resolver,
    /// as it may be defined after the body that calls it. A parameter is known
    /// to be a value already, and refused here.
    fn method(
        &self,
        name: String,
        args: Vec<RawExpr>,
        span: Span,
    ) -> Result<RawExprKind, ParseError> {
        if self.is_param(&name) {
            let note = "only functions can be called with '.'".to_string();
            let kind = ParseErrorKind::NotAFunction(name);

            return Err(ParseError::with_note(kind, Some(span), note));
        }

        Ok(match self.global_call(name, args) {
            RawExprKind::Apply { name, args } => RawExprKind::Method { name, args },
            known => known,
        })
    }

    /// The node for calling a name as it is known outside any definition.
    fn global_call(&self, name: String, args: Vec<RawExpr>) -> RawExprKind {
        if let Some(func) = Function::from(&name) {
            RawExprKind::Call { func, args }
        } else if self.funcs.contains_key(&name) {
            RawExprKind::UserCall { name, args }
        } else {
            RawExprKind::Apply { name, args }
        }
    }
}

/// Parses a call's argument list, and reports the span it occupies.
///
/// The span reaches the closing parenthesis, so a caller can merge it with the
/// callee's name to span the whole call rather than just the name. Without
/// parentheses (i.e: sin10) there is nothing to close, so it is the lone
/// argument's span.
fn consume_args(
    lexer: &mut Lexer,
    ctx: &ParseCtx,
    depth: Depth,
) -> Result<(Vec<RawExpr>, Span), ParseError> {
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
        let arg = nud(lexer, ctx, depth)?;
        let span = *arg.span();

        return Ok((vec![arg], span));
    }

    let open_span = open.span;
    lexer.next();

    // empty arguments. i.e: sin()
    if lexer.peek() == Some(&TokenKind::RParen) {
        let close = lexer.next_token().unwrap();
        return Ok((vec![], open_span.merge(close.span)));
    }

    let mut args: Vec<RawExpr> = Vec::new();

    loop {
        args.push(parse_expression(lexer, 0, ctx, depth)?);

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

    let close = lexer.next_token();

    let Some(close) = close.filter(|token| matches!(token.kind, TokenKind::RParen)) else {
        return Err(ParseError::new(
            ParseErrorKind::MissingClosingParenthesis,
            Some(open_span),
        ));
    };

    Ok((args, open_span.merge(close.span)))
}

fn nud(lexer: &mut Lexer, ctx: &ParseCtx, depth: Depth) -> Result<RawExpr, ParseError> {
    // nesting recurses here: a group, an argument list or a unary chain each
    // re-enter nud, so the native stack grows with how deeply they nest. a
    // flat left-associative chain loops in parse_expression instead and is
    // deliberately not bounded by this
    if depth.exhausted() {
        let span = Span::new(lexer.end_pos(), lexer.end_pos());
        return Err(ParseError::new(
            ParseErrorKind::ExpressionTooDeep,
            Some(span),
        ));
    }

    let depth = depth.nest();

    // unary plus is a no-op: it builds nothing, so a run of them collapses
    // here rather than costing a frame and a descent each
    while matches!(lexer.peek(), Some(TokenKind::Operator(Operator::Add))) {
        lexer.next();
    }

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
            let is_func = ctx.is_func(&name) || lexer.peek() == Some(&TokenKind::LParen);

            if !is_func {
                RawExpr::new(RawExprKind::Identifier(name), span)
            } else {
                // nothing an argument could start with, so the name is being
                // used as a value, which it cannot be
                if !lexer.peek().is_some_and(TokenKind::can_start_operand) {
                    let note = format!("call it with parentheses, e.g. '{name}()'");
                    let kind = ParseErrorKind::FunctionUsedAsValue(name);

                    return Err(ParseError::with_note(kind, Some(span), note));
                }

                let (args, args_span) = consume_args(lexer, ctx, depth)?;

                // the call runs from the name through the closing parenthesis
                let call_span = span.merge(args_span);

                RawExpr::new(ctx.call(name, args), call_span)
            }
        }

        Some(Token {
            kind: TokenKind::LParen,
            span,
        }) => {
            let lhs: RawExpr = parse_expression(lexer, 0, ctx, depth)?;

            match lexer.next_token() {
                Some(Token {
                    kind: TokenKind::RParen,
                    span: token_span,
                }) => {
                    let new_span = Span::merge(span, token_span);

                    RawExpr::new(lhs.kind, new_span)
                }

                Some(other) if matches!(other.kind, TokenKind::Comma) => {
                    let span = *other.span();
                    let kind = ParseErrorKind::ExpectedClosingParenthesis(other);

                    return Err(ParseError::with_note(
                        kind,
                        Some(span),
                        "multiple expressions aren't supported".into(),
                    ));
                }

                Some(other) => {
                    let span = *other.span();
                    let kind = ParseErrorKind::ExpectedClosingParenthesis(other);

                    return Err(ParseError::new(kind, Some(span)));
                }

                None => {
                    return Err(ParseError::new(
                        ParseErrorKind::MissingClosingParenthesis,
                        Some(span),
                    ));
                }
            }
        }

        Some(Token {
            kind: TokenKind::Operator(Operator::Sub),
            span,
        }) => {
            // negation does build a node, so a chain of them (i.e --x) is
            // real depth and stays bounded by the budget
            let expr = nud(lexer, ctx, depth)?;

            let raw_expr_kind = RawExprKind::Unary {
                op: Operator::Neg,
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
    ctx: &ParseCtx,
    depth: Depth,
) -> Result<RawExpr, ParseError> {
    let token = lexer.peek_token();
    let expr = match token {
        Some(Token {
            kind: TokenKind::RParen | TokenKind::Comma,
            span: _,
        }) => lhs,

        Some(Token {
            kind: TokenKind::Dot,
            span: _,
        }) => {
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

            // taken before lhs is moved into the argument list
            let lhs_span = *lhs.span();

            let (mut args, args_span) = consume_args(lexer, ctx, depth)?;
            args.insert(0, lhs);

            // a dot-call spans its receiver too, since that is its first argument
            let call_span = lhs_span.merge(args_span);

            RawExpr::new(ctx.method(name, args, call_span)?, call_span)
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
            let rhs = parse_expression(lexer, r_bp, ctx, depth)?;

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

            let then = parse_expression(lexer, 0, ctx, depth)?;
            if lexer.next() != Some(TokenKind::Colon) {
                let span = span.merge(*then.span());

                return Err(ParseError::with_note(
                    ParseErrorKind::ExpectedColonAfterQuestionMark,
                    Some(span),
                    "add a ':' followed by the else expression".into(),
                ));
            }

            let else_ = parse_expression(lexer, 0, ctx, depth)?;

            // the whole conditional: condition through else branch
            let span = Span::merge(*lhs.span(), *else_.span());

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
                let rhs = parse_expression(lexer, r_bp, ctx, depth)?;

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
    ctx: &ParseCtx,
    depth: Depth,
) -> Result<RawExpr, ParseError> {
    let lhs = nud(lexer, ctx, depth)?;

    parse_infix(lexer, lhs, min_bp, ctx, depth)
}

/// Continues an expression from an operand that has already been parsed.
fn parse_infix(
    lexer: &mut Lexer,
    mut lhs: RawExpr,
    min_bp: u8,
    ctx: &ParseCtx,
    depth: Depth,
) -> Result<RawExpr, ParseError> {
    let mut depth = depth;

    while let Some(token) = lexer.peek() {
        if token.left_bp() <= min_bp {
            break;
        }

        // this loop does not recurse, but every pass wraps lhs in another
        // node, so the tree it leaves behind is dropped recursively later.
        // resolve refuses anything this deep anyway, so bounding it here
        // costs nothing and keeps the tree from being built at all
        depth = depth.nest();

        if depth.exhausted() {
            let span = *lhs.span();
            return Err(ParseError::new(
                ParseErrorKind::ExpressionTooDeep,
                Some(span),
            ));
        }

        lhs = led(lexer, lhs, ctx, depth)?;
    }

    Ok(lhs)
}

/// Parses the `name(a, b, ...)` a definition starts with.
///
/// The name is the one thing on the line the parameters do not shadow: in
/// `f(f) = f + 1` the first 'f' is what is being defined. So it is taken here,
/// as it would be outside any definition, rather than by nud.
fn parse_head(lexer: &mut Lexer, ctx: &ParseCtx) -> Result<RawExpr, ParseError> {
    let Some(Token {
        kind: TokenKind::Identifier(name),
        span,
    }) = lexer.next_token()
    else {
        unreachable!("a definition starts with its name")
    };

    let (args, args_span) = consume_args(lexer, ctx, Depth::new())?;

    Ok(RawExpr::new(
        ctx.global_call(name, args),
        span.merge(args_span),
    ))
}

impl RawExpr {
    pub fn parse(
        mut lexer: Lexer,
        funcs: &HashMap<String, UserFunction>,
    ) -> Result<RawExpr, ParseError> {
        if lexer.is_empty() {
            return Err(ParseError::new(ParseErrorKind::EmptyExpression, None));
        }

        let params = lexer.definition_params();

        let expr = match &params {
            Some(params) => {
                let ctx = ParseCtx { funcs, params };
                let head = parse_head(&mut lexer, &ctx)?;

                parse_infix(&mut lexer, head, 0, &ctx, Depth::new())?
            }

            None => {
                let ctx = ParseCtx { funcs, params: &[] };

                parse_expression(&mut lexer, 0, &ctx, Depth::new())?
            }
        };

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

    pub fn check_errors(&self) -> Result<(), ParseError> {
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
                    let kind = ParseErrorKind::InvalidAssignmentTarget("ans".into());

                    return Err(ParseError::with_note(
                        kind,
                        Some(*lhs.span()),
                        "'ans' always holds the last result and can't be reassigned".into(),
                    ));
                }

                RawExpr {
                    kind: RawExprKind::Constant(constant),
                    ..
                } => {
                    let kind = ParseErrorKind::InvalidAssignmentTarget(constant.to_string());

                    return Err(ParseError::with_note(
                        kind,
                        Some(*lhs.span()),
                        "constants can't be redefined".to_string(),
                    ));
                }

                RawExpr {
                    kind: RawExprKind::Call { func, .. },
                    ..
                } => {
                    let kind = ParseErrorKind::InvalidAssignmentTarget(func.to_string());
                    return Err(ParseError::with_note(
                        kind,
                        Some(*lhs.span()),
                        "built-in functions can't be redefined".to_string(),
                    ));
                }

                _ => (),
            }
        }

        Ok(())
    }

    pub fn classify(self, src: &str) -> Result<Statement, ParseError> {
        match self {
            RawExpr {
                kind:
                    RawExprKind::Binary {
                        op: crate::operator::Operator::Equal,
                        lhs,
                        rhs,
                    },
                span,
            } => match &lhs.kind {
                RawExprKind::Apply { .. } | RawExprKind::UserCall { .. } => {
                    let (RawExprKind::Apply { name, args } | RawExprKind::UserCall { name, args }) =
                        lhs.kind
                    else {
                        unreachable!()
                    };

                    let function = UserFunction::new(name, args, rhs, src.to_owned())?;

                    Ok(Statement::FuncDef(function))
                }

                RawExprKind::Identifier(_) => {
                    let RawExprKind::Identifier(ident) = lhs.kind else {
                        unreachable!()
                    };

                    Ok(Statement::Assign(ident, *rhs))
                }

                _ => Ok(Statement::Eval(RawExpr::new(
                    RawExprKind::Binary {
                        op: crate::operator::Operator::Equal,
                        lhs,
                        rhs,
                    },
                    span,
                ))),
            },

            other => Ok(Statement::Eval(other)),
        }
    }
}

impl fmt::Display for RawExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Identifier(ident) => write!(f, "{}", ident),
            Self::Param { name, .. } => write!(f, "{}", name),
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

            Self::Apply { name, args } | Self::Method { name, args } => write_args!(f, name, args),
            Self::Call { func, args } => write_args!(f, func, args),
            Self::UserCall { name, args } => write_args!(f, name, args),
            Self::Constant(constant) => write!(f, "{}", constant),
        }
    }
}
