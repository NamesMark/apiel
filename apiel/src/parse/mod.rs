pub mod eval;
pub mod val;

use cfgrammar::Span;
use lrlex::{lrlex_mod, DefaultLexerTypes};
use lrpar::{lrpar_mod, Lexeme, Lexer, NonStreamingLexer};

lrlex_mod!("apiel.l");
lrpar_mod!("apiel.y");

pub use eval::Env;
use val::{Scalar, Val};

pub fn parse_and_evaluate(line: &str) -> Result<Vec<f64>, String> {
    let mut env = Env::new();
    parse_and_evaluate_with_env(line, &mut env)
}

pub fn parse_and_evaluate_with_env(line: &str, env: &mut Env) -> Result<Vec<f64>, String> {
    eval_to_val(line, env).map(|val| val.data.into_iter().map(f64::from).collect())
}

pub fn eval_to_val(line: &str, env: &mut Env) -> Result<Val, String> {
    let lexerdef = apiel_l::lexerdef();
    let lexer = lexerdef.lexer(line);

    {
        let mut tokens = String::new();
        for token in lexer.iter() {
            match token {
                Ok(token) => tokens.push_str(&format!("{} ", token.tok_id())),
                Err(e) => {
                    tracing::warn!("Failed to parse a token: {e}");
                    tokens.push_str("UNKNOWN");
                },
            }
        }
        tracing::debug!(tokens, "Tokens:");
    }

    let (res, errs) = apiel_y::parse(&lexer);

    if !errs.is_empty() {
        return Err(format!("Parse error: {:?}", errs));
    }

    if let Some(Ok(r)) = res {
        eval::eval(&lexer, r, env).map_err(|(span, msg)| {
            let ((line, col), _) = lexer.line_col(span);
            format!(
                "Evaluation error at line {} column {}: '{}', {}.",
                line, col, lexer.span_str(span), msg
            )
        })
    } else {
        Err("Failed to evaluate expression".to_string())
    }
}

pub fn format_val(val: &Val) -> String {
    if val.data.iter().all(|s| matches!(s, Scalar::Char(_))) {
        // All chars: display as string
        val.data.iter().map(|s| match s {
            Scalar::Char(c) => *c,
            _ => ' ',
        }).collect()
    } else {
        val.data.iter().map(|v| match v {
            Scalar::Integer(i) => format!("{i}"),
            Scalar::Float(f) if f.fract() == 0.0 => format!("{}", *f as i64),
            Scalar::Float(f) => format!("{f}"),
            Scalar::Char(c) => format!("{c}"),
        }).collect::<Vec<_>>().join(" ")
    }
}
