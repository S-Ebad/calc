//! Several statements in one input, separated by ';'.

mod common;

use common::*;

#[test]
fn statements_run_in_order_and_the_last_one_answers() {
    assert_eq!(solve("a = 2; b = a * 3; a + b"), 8.0);
    assert_eq!(solve("2; ans * 3"), 6.0);
}

#[test]
fn a_definition_is_usable_by_the_statements_after_it() {
    assert_eq!(solve("f(x) = 2x; g(x) = f(x) + 1; g(5)"), 11.0);
}

#[test]
fn a_statement_with_no_value_answers_nothing() {
    let mut calc = Calculator::new();

    assert_eq!(calc.solve("1 + 2; f(x) = x").unwrap(), None);
}

#[test]
fn empty_statements_are_skipped() {
    assert_eq!(solve("1;; ;2;"), 2.0);

    let mut calc = Calculator::new();
    assert_eq!(calc.solve(";").unwrap(), None);

    // without a separator, nothing at all is still a mistake
    assert!(solve_is_err(""));
}

#[test]
fn whitespace_and_line_breaks_only_separate_statements() {
    let src = "sq(x) = x^2;\ncube(x) = x * sq(x);\n\n  cube(2) + sq(3)\n";

    assert_eq!(solve(src), 17.0);
}

/// The first error stops the rest, and what ran before it stays done.
#[test]
fn an_error_stops_the_statements_after_it() {
    let mut session = Session::new();
    session.err("a = 1; b = missing; c = 3");

    assert_eq!(session.value("a"), 1.0);
    assert!(session.is_err("b"));
    assert!(session.is_err("c"));
}

/// Each statement is solved on its own, but its error is reported against the
/// whole input that was given.
#[test]
fn an_error_points_into_the_whole_input() {
    assert_eq!(
        render("a = 1; b = 2; c = zzz + 1; d = 4"),
        "Resolver Error: unknown identifier 'zzz'\n \
         a = 1; b = 2; c = zzz + 1; d = 4\n                   ^^^"
    );

    assert_eq!(
        render("1; 2 $ 3"),
        "Lexer Error: invalid token $\n 1; 2 $ 3\n      ^"
    );

    // a multi-byte name before the statement must not shift the carets
    assert!(render("café = 1; 1 + zzz").ends_with("\n               ^^^"));
}

#[test]
fn an_error_on_a_later_line_points_at_that_line() {
    let src = "sq(x) = x^2;\nhyp(a, b) = sqrt(sq(a) + sq(b));\nhyp(3, nope);";

    assert_eq!(
        render(src),
        "Resolver Error: unknown identifier 'nope'\n hyp(3, nope);\n        ^^^^"
    );
}

/// Inside a call the spans belong to the definition, which knows nothing of
/// where it was written in the input; only the call made from the input moves.
#[test]
fn an_error_inside_a_call_points_into_the_definition() {
    let rendered = render("f(x) = x / q; 1 + f(3)");

    assert_eq!(
        rendered,
        "Resolver Error: unknown identifier 'q'\n \
         f(x) = x / q; 1 + f(3)\n                   ^^^^ in the body of 'f'\n \
         f(x) = x / q\n            ^ unknown identifier 'q'"
    );
}

#[test]
fn help_is_a_statement_like_any_other() {
    assert_eq!(solve("help sin; 2"), 2.0);
}

#[test]
fn a_comment_runs_to_the_end_of_its_line() {
    assert_eq!(solve("1 + 2 # three"), 3.0);
    assert_eq!(solve("1 + # not this\n 2"), 3.0);
    assert_eq!(solve("f(x) = x # the input\n * 2; f(3)"), 6.0);
}

#[test]
fn a_separator_inside_a_comment_separates_nothing() {
    assert_eq!(solve("a = 1 # not; a separator\n + 1; a"), 2.0);
}

#[test]
fn a_statement_that_is_only_a_comment_is_skipped() {
    let mut calc = Calculator::new();

    assert_eq!(calc.solve("# nothing to see").unwrap(), None);
    assert_eq!(calc.solve("1; # the end").unwrap(), Some(1.0));
}

#[test]
fn a_file_of_definitions_with_comments() {
    let src = "\
# std.math
sq(x) = x^2;   # square
cube(x) = x * sq(x);

# hypotenuse; uses sq
hyp(a, b) = sqrt(sq(a) + sq(b));
";

    let mut session = Session::new();
    session.ok(src);

    assert_eq!(session.value("hyp(3, 4)"), 5.0);
    assert_eq!(session.value("cube(2)"), 8.0);
}

#[test]
fn help_can_be_followed_by_a_comment() {
    let mut calc = Calculator::new();

    assert_eq!(calc.solve("help sin # what is sin").unwrap(), None);
    assert_eq!(calc.solve("# first a comment\nhelp sin").unwrap(), None);
}

#[test]
fn an_error_after_a_comment_points_past_it() {
    assert_eq!(
        render("# a comment; with a separator\n1 + zzz"),
        "Resolver Error: unknown identifier 'zzz'\n 1 + zzz\n     ^^^"
    );
}

/// What a function remembers is shown in a call's frame, so the frame shows
/// which comment went with which definition.
#[test]
fn a_comment_after_the_separator_stays_with_its_statement() {
    let mut session = Session::new();
    session.ok("a(x) = x / q;  # divides by q\nb(x) = x / q;  # also divides\n");

    assert_eq!(
        session.render("a(1)"),
        "Resolver Error: unknown identifier 'q'\n \
         a(1)\n ^^^^ in the body of 'a'\n \
         a(x) = x / q;  # divides by q\n            ^ unknown identifier 'q'"
    );

    // the comment before b's line went to a, so b keeps only its own
    assert!(
        session
            .render("b(1)")
            .contains("\n b(x) = x / q;  # also divides\n")
    );
}

/// Only a comment on the rest of the same line goes with the statement before
/// it; anything on the line after starts the next statement.
#[test]
fn a_comment_on_the_next_line_belongs_to_the_next_statement() {
    let mut session = Session::new();
    session.ok("a(x) = x / q;\n# about b\nb(x) = 1;");

    assert!(session.render("a(1)").contains("\n a(x) = x / q\n"));
    assert_eq!(session.value("b(5)"), 1.0);
}
