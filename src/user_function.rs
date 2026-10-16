use std::collections::HashSet;
use std::fmt;

use crate::{
    errors::{ParseError, ParseErrorKind},
    raw_expr::{RawExpr, RawExprKind},
};

#[derive(Debug, Clone, PartialEq)]
pub struct UserFunction {
    name: String,
    params: Vec<String>,
    body: Box<RawExpr>,
    // the input line this was defined on. the body's spans index into it, so a
    // diagnostic raised inside the body needs this text to render against
    src: String,

    // names the body reads that it does not bind itself. a cached result
    // depends on these, and nothing in the cache key records that
    free: HashSet<String>,
}

impl UserFunction {
    pub fn new(
        name: String,
        params: Vec<RawExpr>,
        mut body: Box<RawExpr>,
        src: String,
    ) -> Result<Self, ParseError> {
        let mut names: Vec<String> = Vec::with_capacity(params.len());

        for param in params {
            let RawExpr {
                kind: RawExprKind::Identifier(name),
                span,
            } = param
            else {
                let span = param.span;
                let kind = ParseErrorKind::InvalidParameter(param);

                return Err(ParseError::new(kind, Some(span)));
            };

            // a name can only stand for one slot, so the later one could
            // never be read and an argument would be silently dropped
            if names.contains(&name) {
                let note = format!("'{}' is already a parameter of this function", name);
                let kind = ParseErrorKind::DuplicateParameter(name);

                return Err(ParseError::with_note(kind, Some(span), note));
            }

            names.push(name);
        }

        let params = names;

        bind_params(&mut body, &params);

        let mut free = HashSet::new();
        collect_free(&body, &mut free);

        Ok(Self {
            name,
            params,
            body,
            src,
            free,
        })
    }

    pub fn help(&self) -> String {
        // the definition as it was typed. reconstructing it from the body
        // instead would normalise the source away, turning '2x' into '2 * x'
        format!(
            "{} is a user-defined function.\n\n{}",
            self.name,
            self.src.trim()
        )
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[String] {
        &self.params
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }

    pub fn body(&self) -> &RawExpr {
        &self.body
    }

    pub fn src(&self) -> &str {
        &self.src
    }

    /// Whether the body reads this name from outside its own parameters.
    pub fn reads(&self, name: &str) -> bool {
        self.free.contains(name)
    }
}

impl fmt::Display for UserFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Rewrite references to the function's own parameters into slot lookups.
///
/// Which parameter a name refers to is fixed the moment the function is
/// written, so the search can happen once here instead of on every call. Only
/// the function's own parameters are bound: anything else stays a name,
/// because dynamic scoping decides what it means at the point of the call.
fn bind_params(expr: &mut RawExpr, params: &[String]) {
    match &mut expr.kind {
        RawExprKind::Identifier(name) => {
            if let Some(slot) = params.iter().position(|param| param == name) {
                let name = std::rc::Rc::from(name.as_str());
                expr.kind = RawExprKind::Param { slot, name };
            }
        }

        RawExprKind::Binary { lhs, rhs, .. } => {
            bind_params(lhs, params);
            bind_params(rhs, params);
        }

        RawExprKind::Unary { expr, .. } | RawExprKind::Postfix { expr, .. } => {
            bind_params(expr, params)
        }

        RawExprKind::If {
            condition,
            then,
            else_,
        } => {
            bind_params(condition, params);
            bind_params(then, params);
            bind_params(else_, params);
        }

        RawExprKind::Apply { args, .. }
        | RawExprKind::Method { args, .. }
        | RawExprKind::Call { args, .. }
        | RawExprKind::UserCall { args, .. } => {
            for arg in args {
                bind_params(arg, params);
            }
        }

        RawExprKind::Number(_) | RawExprKind::Constant(_) | RawExprKind::Param { .. } => (),
    }
}

/// Gather the names a body reads that it does not bind itself.
///
/// Runs after the parameters have been bound to slots, so every remaining
/// identifier is free by construction. An applied name is included too, since
/// it is a variable whenever one of that name exists.
fn collect_free(expr: &RawExpr, free: &mut HashSet<String>) {
    match &expr.kind {
        RawExprKind::Identifier(name) => {
            free.insert(name.clone());
        }

        RawExprKind::Apply { name, args } => {
            free.insert(name.clone());

            for arg in args {
                collect_free(arg, free);
            }
        }

        RawExprKind::Binary { lhs, rhs, .. } => {
            collect_free(lhs, free);
            collect_free(rhs, free);
        }

        RawExprKind::Unary { expr, .. } | RawExprKind::Postfix { expr, .. } => {
            collect_free(expr, free)
        }

        RawExprKind::If {
            condition,
            then,
            else_,
        } => {
            collect_free(condition, free);
            collect_free(then, free);
            collect_free(else_, free);
        }

        // a method's name is never read as a variable, so it is not free
        RawExprKind::Method { args, .. }
        | RawExprKind::Call { args, .. }
        | RawExprKind::UserCall { args, .. } => {
            for arg in args {
                collect_free(arg, free);
            }
        }

        RawExprKind::Number(_) | RawExprKind::Constant(_) | RawExprKind::Param { .. } => (),
    }
}
