#[macro_export]
macro_rules! apl {
    // Plain expression
    ($expr:expr) => {{
        $crate::parse::parse_and_evaluate($expr)
    }};
    // Expression with shared environment
    ($expr:expr, $env:expr) => {{
        $crate::parse::parse_and_evaluate_with_env($expr, $env)
    }};
    // Monadic: pass right argument (⍵)
    ($expr:expr, omega: $omega:expr) => {{
        let mut env = $crate::Env::new();
        env.vars.insert("⍵".to_string(), $crate::parse::val::Val::from_f64s($omega));
        $crate::parse::parse_and_evaluate_with_env($expr, &mut env)
    }};
    // Dyadic: pass both arguments (⍺ and ⍵)
    ($expr:expr, alpha: $alpha:expr, omega: $omega:expr) => {{
        let mut env = $crate::Env::new();
        env.vars.insert("⍺".to_string(), $crate::parse::val::Val::from_f64s($alpha));
        env.vars.insert("⍵".to_string(), $crate::parse::val::Val::from_f64s($omega));
        $crate::parse::parse_and_evaluate_with_env($expr, &mut env)
    }};
}
