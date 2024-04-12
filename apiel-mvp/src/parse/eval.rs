// MARK: eval
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