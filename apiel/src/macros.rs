#[macro_export] 
macro_rules! apl {
    ($expr:expr) => {{
        $crate::parse::parse_and_evaluate($expr)
    }};
}