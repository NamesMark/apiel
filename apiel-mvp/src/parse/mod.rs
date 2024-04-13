pub mod eval;
pub mod val;

use cfgrammar::Span;
use lrlex::{lrlex_mod, DefaultLexerTypes};
use lrpar::{lrpar_mod, Lexeme, Lexer, NonStreamingLexer};

lrlex_mod!("apiel.l");
lrpar_mod!("apiel.y");

use apiel_y::Expr;

#[cfg(feature = "debug")]
use lrlex::DefaultLexeme;
#[cfg(feature = "debug")]
use lrpar::{Lexeme, Lexer};

pub fn parse_and_evaluate(line: &str) -> Result<Vec<f64>, String> {
    let lexerdef = apiel_l::lexerdef();
    let lexer = lexerdef.lexer(line);

    //#[cfg(feature = "debug")]
    {
        let mut tokens = String::new();
        for tok in lexer.iter() {
            if let Ok(token) = tok {
                tokens.push_str(&format!("{} ", token.tok_id()));
            } else {
                tokens.push_str("UNKNOWN");
            }
        }
        tracing::debug!(tokens, "Tokens:");
    }

    let (res, errs) = apiel_y::parse(&lexer);

    if !errs.is_empty() {
        return Err(format!("Parse error: {:?}", errs));
    }

    if let Some(Ok(r)) = res {
        match eval::eval(&lexer, r) {
            Ok(i) => Ok(i.into_iter().map(f64::from).collect()),
            Err((span, msg)) => {
                let ((line, col), _) = lexer.line_col(span);
                Err(format!(
                    "Evaluation error at line {} column {}: '{}', {}.",
                    line,
                    col,
                    lexer.span_str(span),
                    msg
                ))
            }
        }
    } else {
        Err("Failed to evaluate expression".to_string())
    }
}
