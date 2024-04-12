#![allow(clippy::unnecessary_wraps)]

use std::io::{self, BufRead, Write};

use cfgrammar::Span;
use lrlex::{lrlex_mod, DefaultLexerTypes};
use lrpar::{lrpar_mod, NonStreamingLexer};

lrlex_mod!("apiel.l");
lrpar_mod!("apiel.y");

use apiel_y::Expr;

fn main() {
    // Get the `LexerDef` for the `apiel` language.
    let lexerdef = apiel_l::lexerdef();
    let stdin = io::stdin();
    loop {
        print!(">>> ");
        io::stdout().flush().ok();
        match stdin.lock().lines().next() {
            Some(Ok(ref l)) => {
                if l.trim().is_empty() {
                    continue;
                }
                // Now we create a lexer with the `lexer` method with which we can lex an input.
                let lexer = lexerdef.lexer(l);
                // Pass the lexer to the parser and lex and parse the input.
                let (res, errs) = apiel_y::parse(&lexer);
                for e in errs {
                    println!("{}", e.pp(&lexer, &apiel_y::token_epp));
                }
                if let Some(Ok(r)) = res {
                    match eval::eval::<i64>(&lexer, r) {
                        Ok(i) => println!("Result: {}", i),
                        Err((span, msg)) => {
                            let ((line, col), _) = lexer.line_col(span);
                            eprintln!(
                                "Evaluation error at line {} column {}, '{}' {}.",
                                line,
                                col,
                                lexer.span_str(span),
                                msg
                            )
                        }
                    }
                }
            }
            _ => break,
        }
    }
}

// MARK: eval

mod eval {
    use super::*;
    use num_traits::{CheckedAdd, CheckedSub, CheckedDiv, CheckedMul};

    pub fn eval<N: CheckedAdd + CheckedSub + CheckedDiv + CheckedMul + std::str::FromStr + std::fmt::Debug + Copy>(
        lexer: &dyn NonStreamingLexer<DefaultLexerTypes<u32>>,
        e: Expr,
    ) -> Result<N, (Span, &'static str)> {
        match e {
            Expr::Add { span, lhs, rhs } => eval::<N>(lexer, *lhs)?
                .checked_add(&eval::<N>(lexer, *rhs)?)
                .ok_or((span, "addition overflowed")),
            Expr::Sub { span, lhs, rhs } => eval::<N>(lexer, *lhs)?
                .checked_sub(&eval::<N>(lexer, *rhs)?)
                .ok_or((span, "subtraction overflowed")),
            Expr::Mul { span, lhs, rhs } => eval::<N>(lexer, *lhs)?
                .checked_mul(&eval::<N>(lexer, *rhs)?)
                .ok_or((span, "multiplication overflowed")),
            Expr::Div { span, lhs, rhs } => eval::<N>(lexer, *lhs)?
                .checked_div(&eval::<N>(lexer, *rhs)?)
                .ok_or((span, "division overflowed")),
            Expr::Number { span } => lexer
                .span_str(span)
                .parse::<N>()
                .map_err(|_| (span, "cannot be represented as a valid number")),
        }
    }
}

