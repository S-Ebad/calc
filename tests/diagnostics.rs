//! What an error reports, not just that one happened.
//!
//! Every stage carries a typed kind, a span and sometimes a note, and an
//! error raised inside a function body renders a call stack. These assert the
//! kind so a failure cannot quietly change stage, and assert the rendered
//! output so the spans stay pointed at the right text.

mod common;

use common::*;

// --- which stage, and which failure ---

#[test]
fn lexer_kinds() {
    assert!(matches!(
        lexer_kind(&err("2 @ 3")),
        Some(LexerErrorKind::InvalidToken('@'))
    ));

    assert!(matches!(
        lexer_kind(&err("9e9e9")),
        Some(LexerErrorKind::InvalidNumber(_))
    ));
}

#[test]
fn parse_kinds() {
    assert!(matches!(
        parse_kind(&err("")),
        Some(ParseErrorKind::EmptyExpression)
    ));

    assert!(matches!(
        parse_kind(&err("(1 + 2))")),
        Some(ParseErrorKind::ExtraClosingParenthesis)
    ));

    assert!(matches!(
        parse_kind(&err("(1 + 2")),
        Some(ParseErrorKind::MissingClosingParenthesis)
    ));

    assert!(matches!(
        parse_kind(&err("1 +")),
        Some(ParseErrorKind::UnexpectedEndOfInput)
    ));

    assert!(matches!(
        parse_kind(&err("1 2")),
        Some(ParseErrorKind::MissingOperator)
    ));

    assert!(matches!(
        parse_kind(&err("sin(1,)")),
        Some(ParseErrorKind::TrailingComma)
    ));

    assert!(matches!(
        parse_kind(&err("1 ? 2")),
        Some(ParseErrorKind::ExpectedColonAfterQuestionMark)
    ));

    assert!(matches!(
        parse_kind(&err("max(1, sin)")),
        Some(ParseErrorKind::FunctionUsedAsValue(_))
    ));

    assert!(matches!(
        parse_kind(&err("2 .")),
        Some(ParseErrorKind::ExpectedMethodName(None))
    ));

    assert!(matches!(
        parse_kind(&err("2 .5x")),
        Some(ParseErrorKind::ExpectedMethodName(Some(_)))
    ));

    // a group holding something other than one expression
    assert!(matches!(
        parse_kind(&err("(1, 2)")),
        Some(ParseErrorKind::ExpectedClosingParenthesis(_))
    ));

    // a token that cannot begin one
    assert!(matches!(
        parse_kind(&err("*5")),
        Some(ParseErrorKind::CannotStartExpression(_))
    ));

    // and one left over after a complete expression
    assert!(matches!(
        parse_kind(&err("1 , 2")),
        Some(ParseErrorKind::UnexpectedToken(_))
    ));
}

#[test]
fn assignment_targets_are_rejected_by_the_parser() {
    assert!(matches!(
        parse_kind(&err("ans = 1")),
        Some(ParseErrorKind::InvalidAssignmentTarget(_))
    ));

    assert!(matches!(
        parse_kind(&err("pi = 3")),
        Some(ParseErrorKind::InvalidAssignmentTarget(_))
    ));

    assert!(matches!(
        parse_kind(&err("sin(x) = 2x")),
        Some(ParseErrorKind::InvalidAssignmentTarget(_))
    ));

    assert!(matches!(
        parse_kind(&err("f(1) = 2")),
        Some(ParseErrorKind::InvalidParameter(_))
    ));
}

#[test]
fn resolver_kinds() {
    assert!(matches!(
        resolver_kind(&err("nope(1)")),
        Some(ResolverErrorKind::UnknownFunction(_))
    ));

    assert!(matches!(
        resolver_kind(&err("nope")),
        Some(ResolverErrorKind::UnknownIdentifier(_))
    ));

    assert!(matches!(
        resolver_kind(&err("ans")),
        Some(ResolverErrorKind::UndefinedAns)
    ));

    let mut session = Session::new();
    session.ok("f(a, b) = a + b");

    assert!(matches!(
        resolver_kind(&session.err("f(1)")),
        Some(ResolverErrorKind::ArityMismatch {
            expected: 2,
            got: 1,
            ..
        })
    ));

    session.ok("x = 5");

    assert!(matches!(
        resolver_kind(&session.err("x(1, 2)")),
        Some(ResolverErrorKind::InvalidMultiplication { .. })
    ));
}

#[test]
fn eval_kinds() {
    assert!(matches!(
        eval_kind(&err("1/0")),
        Some(EvalErrorKind::DivisionByZero { .. })
    ));

    assert!(matches!(
        eval_kind(&err("0^-2")),
        Some(EvalErrorKind::ZeroToNegativePower { .. })
    ));

    // the function and the operator report the same failure
    assert!(matches!(
        eval_kind(&err("pow(0, -2)")),
        Some(EvalErrorKind::ZeroToNegativePower { .. })
    ));

    assert!(matches!(
        eval_kind(&err("(-1)!")),
        Some(EvalErrorKind::FactorialUndefined(_))
    ));

    assert!(matches!(
        eval_kind(&err("171!")),
        Some(EvalErrorKind::FactorialTooLarge(_))
    ));

    assert!(matches!(
        eval_kind(&err("sin(1, 2)")),
        Some(EvalErrorKind::BuiltinArity { got: 2, .. })
    ));

    assert!(matches!(
        eval_kind(&err("clamp(1, 5, 2)")),
        Some(EvalErrorKind::ClampRange { .. })
    ));

    assert!(matches!(
        eval_kind(&err("tan(pi/2)")),
        Some(EvalErrorKind::TanAsymptote(_))
    ));

    assert!(matches!(
        eval_kind(&err("recip(0)")),
        Some(EvalErrorKind::RecipZero)
    ));

    assert!(matches!(
        eval_kind(&err("sqrt(4, 0)")),
        Some(EvalErrorKind::SqrtZeroRoot(_))
    ));

    assert!(matches!(
        eval_kind(&err("inf - inf")),
        Some(EvalErrorKind::UndefinedBinary { .. })
    ));

    assert!(matches!(
        eval_kind(&err("ln(-1)")),
        Some(EvalErrorKind::UndefinedCall { .. })
    ));

    assert!(matches!(
        eval_kind(&err("1 + 2 = 3")),
        Some(EvalErrorKind::AssignmentNotAtTopLevel)
    ));
}

/// Resolving a body happens during evaluation, but the failure is still the
/// resolver's and has to report as such rather than as an eval error.
#[test]
fn a_body_failing_to_resolve_stays_a_resolution_error() {
    let mut session = Session::new();
    session.ok("f(x) = x + missing");

    let err = session.err("f(1)");

    assert!(matches!(
        eval_kind(&err),
        Some(EvalErrorKind::Resolution(_))
    ));

    assert!(
        render_error("f(1)", err).starts_with("Resolver Error:"),
        "a resolution failure should not be relabelled by the stage that noticed it"
    );
}

// --- what the carets point at ---

#[test]
fn the_span_covers_the_offending_text() {
    assert_eq!(render("1/0").lines().nth(2).unwrap(), " ^^^");

    // a call is spanned from its name through the closing parenthesis
    assert_eq!(render("sin(1, 2)").lines().nth(2).unwrap(), " ^^^^^^^^^");
    assert_eq!(
        render("clamp(1, 5, 2)").lines().nth(2).unwrap(),
        " ^^^^^^^^^^^^^^"
    );

    // and the offending token, not the whole line, when that is the problem
    assert_eq!(render("1 + @").lines().nth(2).unwrap(), "     ^");
}

#[test]
fn a_note_is_rendered_beside_the_carets() {
    let rendered = render("sin(1,)");

    assert!(
        rendered.contains("remove the ','"),
        "expected the note, got:\n{}",
        rendered
    );
}

#[test]
fn an_error_with_no_span_renders_as_one_line() {
    assert_eq!(render("").lines().count(), 1);
}

// --- the call stack ---

#[test]
fn an_error_inside_a_body_renders_a_frame_per_call() {
    let mut session = Session::new();
    session
        .ok("g(x) = x/0")
        .ok("h(x) = g(x)")
        .ok("k(x) = h(x) + 1");

    let rendered = session.render("k(7)");
    let lines: Vec<&str> = rendered.lines().collect();

    // a header, then a source line and a caret line for each of k, h, g and
    // the division itself
    assert_eq!(lines.len(), 1 + 2 * 4, "got:\n{}", rendered);

    assert!(lines[0].starts_with("Eval Error: 7/0 is undefined"));

    assert_eq!(lines[1], " k(7)");
    assert!(lines[2].ends_with("in the body of 'k'"));

    assert_eq!(lines[3], " k(x) = h(x) + 1");
    assert!(lines[4].ends_with("in the body of 'h'"));

    assert_eq!(lines[5], " h(x) = g(x)");
    assert!(lines[6].ends_with("in the body of 'g'"));

    // the innermost frame repeats the message, since by then the carets are
    // four lines below the header
    assert_eq!(lines[7], " g(x) = x/0");
    assert!(lines[8].ends_with("7/0 is undefined (division by zero)"));
}

#[test]
fn each_frame_renders_against_the_source_it_was_defined_on() {
    let mut session = Session::new();
    session.ok("inner(a) = a + nope").ok("outer(b) = inner(b)");

    let rendered = session.render("outer(2)");

    // the definitions are not the line being evaluated, yet both appear
    assert!(rendered.contains("outer(b) = inner(b)"), "{}", rendered);
    assert!(rendered.contains("inner(a) = a + nope"), "{}", rendered);
}

#[test]
fn a_deep_stack_elides_its_middle() {
    let mut session = Session::new();
    session.ok("r(x) = r(x) + 1");

    let rendered = session.render("r(1)");

    assert!(rendered.contains("more call(s)"), "{}", rendered);
    assert!(
        rendered.lines().count() < 20,
        "a hundred frames should not all be printed:\n{}",
        rendered
    );
}

#[test]
fn an_unknown_name_bound_by_a_caller_says_so() {
    let mut session = Session::new();
    session
        .ok("callee(a) = a + outer")
        .ok("caller(outer) = callee(1)");

    let rendered = session.render("caller(7)");

    assert!(
        rendered.contains("parameter of a calling function"),
        "a name deliberately out of reach should not look like a typo:\n{}",
        rendered
    );
}

#[test]
fn an_ordinary_unknown_name_gets_no_such_note() {
    let mut session = Session::new();
    session.ok("f(x) = x + typo");

    let rendered = session.render("f(1)");

    assert!(
        !rendered.contains("parameter of a calling function"),
        "{}",
        rendered
    );
}
