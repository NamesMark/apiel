pub mod eval;

use cfgrammar::Span;
use lrlex::{lrlex_mod, DefaultLexerTypes};
use lrpar::{lrpar_mod, NonStreamingLexer};

lrlex_mod!("apiel.l");
lrpar_mod!("apiel.y");

use apiel_y::Expr;

#[cfg(feature = "debug")]
use lrlex::DefaultLexeme; 
#[cfg(feature = "debug")]
use lrpar::{Lexer, Lexeme};

pub fn parse_and_evaluate(line: &str) -> Result<i64, String> {
    let lexerdef = apiel_l::lexerdef();
    let lexer = lexerdef.lexer(line);

    #[cfg(feature = "debug")]
    {
        println!("Tokens:");
        for tok in lexer.iter() {
            if let Ok(token) = tok {
                print!("{} ", token.tok_id());
            } else {
                print!("UNKNOWN ");
            }
        }
        println!();
    }

    let (res, errs) = apiel_y::parse(&lexer);

    if !errs.is_empty() {
        return Err(format!("Parse error: {:?}", errs));
    }

    if let Some(Ok(r)) = res {
        match eval::eval::<i64>(&lexer, r) {
            Ok(i) => Ok(i),
            Err((span, msg)) => {
                let ((line, col), _) = lexer.line_col(span);
                Err(format!("Evaluation error at line {} column {}: '{}', {}.", line, col, lexer.span_str(span), msg))
            }
        }
    } else {
        Err("Failed to evaluate expression".to_string())
    }
}
