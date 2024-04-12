// MARK: eval
use super::*;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use num::checked_pow;

pub fn eval<
    N: CheckedAdd + CheckedSub + CheckedDiv + CheckedMul + std::str::FromStr + std::fmt::Debug + Copy + num::One + TryInto<usize>,
>(
    lexer: &dyn NonStreamingLexer<DefaultLexerTypes<u32>>,
    e: Expr,
) -> Result<Vec<N>, (Span, &'static str)> {
    match e {
        Expr::Add { span, lhs, rhs } => {
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            if !check_lengths(&lhs_eval, &rhs_eval) {
                return Err((
                    span,
                    "Can only add same-sized vectors, scalars, or scalar to vector",
                ));
            }

            if lhs_eval.len() == 1 {
                rhs_eval
                    .iter()
                    .map(|num| {
                        num.checked_add(&lhs_eval[0])
                            .ok_or((span, "addition overflowed"))
                    })
                    .collect()
            } else if rhs_eval.len() == 1 {
                lhs_eval
                    .iter()
                    .map(|num| {
                        num.checked_add(&rhs_eval[0])
                            .ok_or((span, "addition overflowed"))
                    })
                    .collect()
            } else {
                lhs_eval
                    .iter()
                    .zip(rhs_eval.iter())
                    .map(|(l, r)| l.checked_add(r).ok_or((span, "addition overflowed")))
                    .collect()
            }
        }
        Expr::Sub { span, lhs, rhs } => {
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            if !check_lengths(&lhs_eval, &rhs_eval) {
                return Err((
                    span,
                    "Can only substract same-sized vectors, scalars, or scalar from vector",
                ));
            }

            if lhs_eval.len() == 1 {
                rhs_eval
                    .iter()
                    .map(|num| {
                        num.checked_sub(&lhs_eval[0])
                            .ok_or((span, "subtraction overflowed"))
                    })
                    .collect()
            } else if rhs_eval.len() == 1 {
                lhs_eval
                    .iter()
                    .map(|num| {
                        num.checked_sub(&rhs_eval[0])
                            .ok_or((span, "subtraction overflowed"))
                    })
                    .collect()
            } else {
                lhs_eval
                    .iter()
                    .zip(rhs_eval.iter())
                    .map(|(l, r)| l.checked_sub(r).ok_or((span, "subtraction overflowed")))
                    .collect()
            }
        }
        Expr::Mul { span, lhs, rhs } => {
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            if !check_lengths(&lhs_eval, &rhs_eval) {
                return Err((
                    span,
                    "Can only multiply same-sized vectors, scalars, or vector by scalar",
                ));
            }

            if lhs_eval.len() == 1 {
                rhs_eval
                    .iter()
                    .map(|num| {
                        num.checked_mul(&lhs_eval[0])
                            .ok_or((span, "multiplication overflowed"))
                    })
                    .collect()
            } else if rhs_eval.len() == 1 {
                lhs_eval
                    .iter()
                    .map(|num| {
                        num.checked_mul(&rhs_eval[0])
                            .ok_or((span, "multiplication overflowed"))
                    })
                    .collect()
            } else {
                lhs_eval
                    .iter()
                    .zip(rhs_eval.iter())
                    .map(|(l, r)| l.checked_mul(r).ok_or((span, "multiplication overflowed")))
                    .collect()
            }
        }
        Expr::Div { span, lhs, rhs } => {
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            if !check_lengths(&lhs_eval, &rhs_eval) {
                return Err((
                    span,
                    "Can only divide same-sized vectors, scalars, or vector by scalar",
                ));
            }

            if lhs_eval.len() == 1 {
                rhs_eval
                    .iter()
                    .map(|num| {
                        num.checked_div(&lhs_eval[0])
                            .ok_or((span, "division overflowed"))
                    })
                    .collect()
            } else if rhs_eval.len() == 1 {
                lhs_eval
                    .iter()
                    .map(|num| {
                        num.checked_div(&rhs_eval[0])
                            .ok_or((span, "division overflowed"))
                    })
                    .collect()
            } else {
                lhs_eval
                    .iter()
                    .zip(rhs_eval.iter())
                    .map(|(l, r)| l.checked_div(r).ok_or((span, "division overflowed")))
                    .collect()
            }
        }
        Expr::Exp { span, lhs, rhs } => {
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;
        
            if !check_lengths(&lhs_eval, &rhs_eval) {
                return Err((
                    span,
                    "Can only raise to the power of same-sized vectors, scalars, or vector by scalar",
                ));
            }
        
            if lhs_eval.len() == 1 {
                rhs_eval
                    .iter()
                    .map(|num| {
                        let exponent = TryInto::<usize>::try_into(lhs_eval[0])
                            .map_err(|_| (span, "cannot be represented as a valid number"))?;
                        checked_pow(*num, exponent)
                            .ok_or((span, "exponentiation overflowed"))
                    })
                    .collect()
            } else if rhs_eval.len() == 1 {
                lhs_eval
                    .iter()
                    .map(|num| {
                        let exponent = TryInto::<usize>::try_into(rhs_eval[0])
                            .map_err(|_| (span, "cannot be represented as a valid number"))?;
                        checked_pow(*num, exponent)
                            .ok_or((span, "exponentiation overflowed"))
                    })
                    .collect()
            } else {
                lhs_eval
                    .iter()
                    .zip(rhs_eval.iter())
                    .map(|(l, r)| {
                        let exp = TryInto::<usize>::try_into(*r)
                            .map_err(|_| (span, "exponentiation overflowed"))?;
                        checked_pow(*l, exp)
                            .ok_or((span, "exponentiation overflowed"))
                    })
                    .collect()
            }
        }
        Expr::Scalar { span, .. } => lexer
            .span_str(span)
            .parse::<N>()
            .map(|num| vec![num])
            .map_err(|_| (span, "cannot be represented as a valid number")),
        Expr::Vector { span, elements } => {
            #[cfg(feature = "debug")]
            {
                println!("Vector elements: {:?}", elements);
            }

            elements
                .into_iter()
                .map(|elem| eval::<N>(lexer, elem))
                .collect::<Result<Vec<_>, _>>()
                .map(|vals| vals.into_iter().flatten().collect())
                .map_err(|_| (span, "error evaluating vector"))
        }
    }
}

fn check_lengths<N>(lhs_eval: &Vec<N>, rhs_eval: &Vec<N>) -> bool {
    lhs_eval.len() == rhs_eval.len() || lhs_eval.len() == 1 || rhs_eval.len() == 1
}
