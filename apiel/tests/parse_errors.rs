use apiel::Env;
use apiel::parse::eval_to_val;

fn parse_error(expr: &str) -> String {
    eval_to_val(expr, &mut Env::new()).expect_err(expr)
}

#[test]
fn points_at_unexpected_token() {
    assert_eq!(
        parse_error("2*[1 2 3]"),
        "Parse error: unexpected '['\n2*[1 2 3]\n  ^\nhint: APL lists need no brackets, write `1 2 3`"
    );
}

#[test]
fn reports_end_of_input() {
    assert_eq!(parse_error("1+"), "Parse error: unexpected end of input\n1+\n  ^");
}

#[test]
fn hints_apl_glyphs() {
    let cases = [
        ("1<=2", "`≤`"),
        ("1>=2", "`≥`"),
        ("1!=2", "`≠`"),
        ("1==2", "single `=`"),
        ("1&0", "`∧`"),
        ("7%2", "`÷`"),
    ];
    for (expr, hint) in cases {
        let message = parse_error(expr);
        assert!(message.contains(hint), "`{expr}`: {message}");
    }
}
