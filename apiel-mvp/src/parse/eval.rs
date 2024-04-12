// MARK: eval
use super::*;
use num_traits::{CheckedAdd, CheckedSub, CheckedDiv, CheckedMul};

pub fn eval<N: CheckedAdd + CheckedSub + CheckedDiv + CheckedMul + std::str::FromStr + std::fmt::Debug + Copy>(
    lexer: &dyn NonStreamingLexer<DefaultLexerTypes<u32>>,
    e: Expr,
) -> Result<Vec<N>, (Span, &'static str)> {
    match e {
        Expr::Add { span, lhs, rhs } => {
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            if !check_lengths(&lhs_eval, &rhs_eval) {
                return Err((span, "Can only add same-sized vectors, scalars, or scalar to vector"));
            }

            lhs_eval.iter().zip(rhs_eval.iter())
                    .map(|(l, r)| l.checked_add(r).ok_or((span, "addition overflowed")))
                    .collect()
        },
        Expr::Sub { span, lhs, rhs } => {
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            if !check_lengths(&lhs_eval, &rhs_eval) {
                return Err((span, "Can only substract same-sized vectors, scalars, or scalar from vector"));
            }

            lhs_eval.iter().zip(rhs_eval.iter())
                    .map(|(l, r)| l.checked_sub(r).ok_or((span, "subtraction overflowed")))
                    .collect()
        },
        Expr::Mul { span, lhs, rhs } => {
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            if !check_lengths(&lhs_eval, &rhs_eval) {
                return Err((span, "Can only multiply same-sized vectors, scalars, or vector by scalar"));
            }

            lhs_eval.iter().zip(rhs_eval.iter())
                    .map(|(l, r)| l.checked_mul(r).ok_or((span, "multiplication overflowed")))
                    .collect()
        },
        Expr::Div { span, lhs, rhs } => {
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            if !check_lengths(&lhs_eval, &rhs_eval) {
                return Err((span, "Can only divide same-sized vectors, scalars, or vector by scalar"));
            }

            lhs_eval.iter().zip(rhs_eval.iter())
                    .map(|(l, r)| l.checked_div(r).ok_or((span, "division overflowed")))
                    .collect()
        },
        Expr::Scalar { span, .. } => {
            lexer.span_str(span).parse::<N>()
                .map(|num| vec![num])
                .map_err(|_| (span, "cannot be represented as a valid number"))
        },
        Expr::Vector { span, elements } => {
            #[cfg(feature = "debug")]
            {
                println!("Vector elements: {:?}", elements);
            }

            elements.into_iter()
                .map(|elem| eval::<N>(lexer, elem))
                .collect::<Result<Vec<_>, _>>()
                .map(|vals| vals.into_iter().flatten().collect())
                .map_err(|_| (span, "error evaluating vector"))
        },
    }
}

fn check_lengths<N>(lhs_eval: &Vec<N>, rhs_eval: &Vec<N>) -> bool {
    lhs_eval.len() == rhs_eval.len() 
    || lhs_eval.len() == 1 
    || rhs_eval.len() == 1
}