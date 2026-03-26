#[macro_export]
macro_rules! apl {
    ($expr:expr) => {{
        $crate::parse::parse_and_evaluate($expr)
    }};
    ($expr:expr, $env:expr) => {{
        $crate::parse::parse_and_evaluate_with_env($expr, $env)
    }};
}