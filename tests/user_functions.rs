//! User-defined functions and what may not be redefined.

mod common;

use common::*;

#[test]
fn multi_parameter_function() {
    let mut calc = Calculator::new();
    // Area of a cylinder: 2*pi*r*h + 2*pi*r^2
    calc.solve("cylinder_area(r, h) = 2pi*r*h + 2pi*r^2")
        .unwrap();
    let result = calc.solve("cylinder_area(3, 5)").unwrap().unwrap();
    let expected = round(
        2.0 * std::f64::consts::PI * 3.0 * 5.0 + 2.0 * std::f64::consts::PI * f64::powi(3.0, 2),
    );
    assert_eq!(result, expected);
}

#[test]
fn function_redefinition() {
    let mut calc = Calculator::new();
    calc.solve("f(x) = x + 1").unwrap();
    assert_eq!(calc.solve("f(5)").unwrap().unwrap(), 6.0);

    // Redefine f(x)
    calc.solve("f(x) = x * 2").unwrap();
    assert_eq!(calc.solve("f(5)").unwrap().unwrap(), 10.0);
}

#[test]
fn nested_user_calls() {
    let mut calc = Calculator::new();
    calc.solve("square(x) = x * x").unwrap();
    calc.solve("double(x) = x + x").unwrap();
    // double(square(3)) = (3*3) + (3*3) = 18
    assert_eq!(calc.solve("double(square(3))").unwrap().unwrap(), 18.0);
}

#[test]
fn function_using_variables() {
    let mut calc = Calculator::new();
    calc.solve("k = 2").unwrap();
    calc.solve("f(x) = k * x").unwrap();
    assert_eq!(calc.solve("f(10)").unwrap().unwrap(), 20.0);

    // Changing k should change the result of the function call
    calc.solve("k = 5").unwrap();
    assert_eq!(calc.solve("f(10)").unwrap().unwrap(), 50.0);
}

#[test]
fn arity_mismatch() {
    let mut calc = Calculator::new();
    calc.solve("f(x, y) = x + y").unwrap();
    // Too few arguments
    assert!(calc.solve("f(1)").is_err());
    // Too many arguments
    assert!(calc.solve("f(1, 2, 3)").is_err());
}

#[test]
fn illegal_parameter_definition() {
    // Parameters must be identifiers, not numbers or expressions
    assert!(solve_is_err("f(10) = 10 * 2"));
}

#[test]
fn implicit_mul() {
    let mut calc = Calculator::new();
    calc.solve("f(x) = 2x").unwrap();

    assert_eq!(calc.solve("f5").unwrap().unwrap(), 10.0);
    assert_eq!(calc.solve("2f5").unwrap().unwrap(), 20.0);
}

#[test]
fn test_function_composition() {
    let mut calc = Calculator::new();
    calc.solve("f(x) = 10x").unwrap();
    calc.solve("g(x) = 20x").unwrap();

    assert_eq!(calc.solve("f(g(10))").unwrap().unwrap(), 2000.0);
}

#[test]
fn invalid_redefinition() {
    let mut calc = Calculator::new();

    //ans is a reserved keyword
    assert!(calc.solve("ans = 50").is_err());

    // constants are reserved
    assert!(calc.solve("pi = 13").is_err());
    assert!(calc.solve("e = 13").is_err());
    assert!(calc.solve("inf = 13").is_err());
    assert!(calc.solve("true = 1").is_err());
    assert!(calc.solve("false = 1").is_err());

    //sin is a built-in global
    assert!(calc.solve("sin(x) = 50x").is_err());
    assert!(calc.solve("cos(x, y, z) = 50x").is_err());

    // u can't make functions variables. It'll just break during parsing
    assert!(calc.solve("sin = 10").is_err());

    // same with user-defined functions
    calc.solve("f(x) = 2x").unwrap();
    assert!(calc.solve("f = 10").is_err());

    // a defined function's name can still be a parameter, and shadows it in
    // the body (see tests/scoping.rs)
    calc.solve("g(x) = 2x").unwrap();
    calc.solve("h(g) = 2g").unwrap();
    assert_eq!(calc.solve("h(3)").unwrap().unwrap(), 6.0);

    // this is fine with variables though
    calc.solve("x = 10").unwrap();
    calc.solve("f(x) = 10x").unwrap();
    assert_eq!(calc.solve("f(2)").unwrap().unwrap(), 20.0);
}

/// A name can only stand for one slot, so a repeated parameter would make an
/// argument unreachable and silently discard it.
#[test]
fn a_parameter_name_cannot_repeat() {
    let mut session = Session::new();

    assert!(matches!(
        parse_kind(&session.err("f(a, a) = a")),
        Some(ParseErrorKind::DuplicateParameter(_))
    ));

    assert!(session.is_err("f(a, b, a) = a"));

    // different names are of course fine
    session.ok("f(a, b) = a + b");
    assert_eq!(session.value("f(1, 2)"), 3.0);
}

/// A name stands for one kind of thing. Whether it is a call is decided while
/// the line is still being read, so a name that is both leaves one of them
/// permanently unreachable.
#[test]
fn a_name_is_a_variable_or_a_function_but_not_both() {
    let mut session = Session::new();
    session.ok("v = 2");

    assert!(matches!(
        resolver_kind(&session.err("v(n) = n")),
        Some(ResolverErrorKind::AlreadyAVariable(_))
    ));

    session.ok("g(n) = n");

    assert!(matches!(
        resolver_kind(&session.err("g = 5")),
        Some(ResolverErrorKind::AlreadyAFunction(_))
    ));
}

/// Replacing a name with another of its own kind is not a collision.
#[test]
fn a_name_may_be_replaced_by_its_own_kind() {
    let mut session = Session::new();

    session.ok("x = 1").ok("x = 2");
    assert_eq!(session.value("x"), 2.0);

    session.ok("f(n) = n").ok("f(n) = n * 2");
    assert_eq!(session.value("f(3)"), 6.0);
}

/// A built-in is not a user function, so the collision check never sees it;
/// assigning to one has to be refused on its own.
#[test]
fn a_builtin_name_cannot_be_assigned_to() {
    assert!(matches!(
        parse_kind(&err("sin = 5")),
        Some(ParseErrorKind::InvalidAssignmentTarget(_))
    ));

    assert!(matches!(
        parse_kind(&err("sin(x) = 2x")),
        Some(ParseErrorKind::InvalidAssignmentTarget(_))
    ));
}

/// There are no function values, so a name used where one is expected is an
/// error rather than something the parser should try to call.
#[test]
fn a_function_used_as_a_value_says_so() {
    let mut session = Session::new();
    session.ok("f(n) = n * 10");

    for line in ["f", "2 f", "f!", "f * 2", "max(1, f)", "sin"] {
        assert!(
            matches!(
                parse_kind(&session.err(line)),
                Some(ParseErrorKind::FunctionUsedAsValue(_))
            ),
            "{:?}",
            line
        );
    }
}

/// An eager call still takes the next operand, including a negative one, so
/// the check above must not swallow these.
#[test]
fn an_implicit_call_still_takes_its_argument() {
    let mut session = Session::new();
    session.ok("f(n) = n * 10");

    assert_eq!(session.value("f 3"), 30.0);
    assert_eq!(session.value("2 f 3"), 60.0);
    assert_eq!(round(session.value("sin -1")), round(-(1f64).sin()));
    assert_eq!(session.value("sin 0"), 0.0);
}

/// Names in a body are only looked up when the call reaches them, so a branch
/// the call does not take may refer to something that does not exist (yet).
#[test]
fn an_untaken_branch_of_a_body_is_not_resolved() {
    let mut session = Session::new();
    session.ok("f(x) = x > 0 ? x : missing");

    assert_eq!(session.value("f(3)"), 3.0);

    assert!(matches!(
        eval_kind(&session.err("f(-3)")),
        Some(EvalErrorKind::Resolution(kind))
            if matches!(kind.as_ref(), ResolverErrorKind::UnknownIdentifier(name) if name == "missing")
    ));

    session.ok("missing = 7");

    assert_eq!(session.value("f(-3)"), 7.0);
}

/// How many arguments a built-in takes cannot change, so a wrong count in a
/// body is refused when the function is written, whichever branch it is in.
#[test]
fn a_builtin_with_the_wrong_arity_is_refused_at_definition() {
    let mut session = Session::new();

    for def in ["f(x) = sin(x, 2)", "f(x) = x > 0 ? x : max()"] {
        assert!(matches!(
            resolver_kind(&session.err(def)),
            Some(ResolverErrorKind::BuiltinArity { .. })
        ));
    }

    // nothing was defined by the refused attempts
    assert!(session.is_err("f(1)"));
}
