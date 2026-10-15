//! Behaviour of the driver itself rather than of an expression: the help
//! command, and what a definition returns.

mod common;

use common::*;

#[test]
fn a_definition_returns_no_value() {
    let mut calc = Calculator::new();

    assert_eq!(calc.solve("f(x) = 2x").unwrap(), None);
    assert_eq!(calc.solve("f(3)").unwrap(), Some(6.0));
}

#[test]
fn an_assignment_returns_its_value() {
    let mut calc = Calculator::new();

    assert_eq!(calc.solve("x = 4").unwrap(), Some(4.0));
}

#[test]
fn help_is_accepted_in_its_various_forms() {
    let mut calc = Calculator::new();

    // the listing, a built-in, a constant, a name that is none of those, and
    // the word reached past surrounding whitespace
    for line in [
        "help",
        "help ",
        "  help  ",
        "help sin",
        "  help pi",
        "help nosuchthing",
    ] {
        assert_eq!(calc.solve(line).unwrap(), None, "{:?}", line);
    }
}

/// The word has to stand on its own. Letters running straight on from it are
/// one identifier, not a request for help with what follows.
#[test]
fn help_run_together_with_a_name_is_an_identifier() {
    let mut session = Session::new();

    assert!(matches!(
        resolver_kind(&session.err("helpme")),
        Some(ResolverErrorKind::UnknownIdentifier(name)) if name == "helpme"
    ));

    assert!(session.is_err("help_x"));

    // and so it can be used as one
    session.ok("helpme = 7");
    assert_eq!(session.value("helpme"), 7.0);
    assert_eq!(session.value("helpme * 2"), 14.0);
}

#[test]
fn help_describes_a_user_function() {
    let mut calc = Calculator::new();
    calc.solve("f(x) = 2x").unwrap();

    assert_eq!(calc.solve("help f").unwrap(), None);
}

/// `help` only takes the line when what follows it could name something, so
/// an expression that merely starts with those letters still evaluates.
#[test]
fn help_does_not_swallow_an_expression() {
    let mut session = Session::new();
    session.ok("help = 1");

    assert_eq!(session.value("help - 1"), 0.0);
    assert_eq!(session.value("help * 3"), 3.0);
}

#[test]
fn ans_survives_a_failed_line() {
    let mut session = Session::new();

    assert_eq!(session.value("6 * 7"), 42.0);
    assert!(session.is_err("1 +"));
    assert_eq!(session.value("ans"), 42.0);
}
