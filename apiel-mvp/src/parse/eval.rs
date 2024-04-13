// MARK: eval
use super::*;
use eyre::{eyre, OptionExt, Result};
use num::{checked_pow, FromPrimitive, ToPrimitive};
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub, One, Pow};
use tracing::{debug, error};

fn apply_dyadic_operation<N, F>(
    span: Span,
    lhs: &[N],
    rhs: &[N],
    operation: F,
) -> Result<Vec<N>, (Span, &'static str)>
where
    N: PartialOrd,
    F: Fn(&N, &N) -> Result<N, &'static str>,
{
    match (lhs.len(), rhs.len()) {
        (1, _) => {
            // Scalar on lhs, vector on rhs
            rhs.iter()
                .map(|r| operation(&lhs[0], &r))
                .collect::<Result<Vec<N>, _>>()
                .map_err(|_| (span, "Operation failed"))
        }
        (_, 1) => {
            // Vector on lhs, scalar on rhs
            lhs.iter()
                .map(|l| operation(&l, &rhs[0]))
                .collect::<Result<Vec<N>, _>>()
                .map_err(|_| (span, "Operation failed"))
        }
        (_, _) if lhs.len() == rhs.len() => {
            // Both vectors of the same size
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| operation(&l, &r))
                .collect::<Result<Vec<N>, _>>()
                .map_err(|_| (span, "Operation failed"))
        }
        // Bad vector sizes
        _ => Err((
            Span::new(0, 0),
            "operands must be of the same size or one must be scalar",
        )),
    }
}

fn apply_monadic_operation<N, F>(
    span: Span,
    arg: &[N],
    operation: F,
) -> Result<Vec<N>, (Span, &'static str)>
where
    N: PartialOrd,
    F: Fn(&N) -> Result<N, &'static str>,
{
    arg.iter()
        .map(|el| operation(el))
        .collect::<Result<Vec<N>, _>>()
        .map_err(|_| (span, "Operation failed"))
}

//#[tracing::instrument(skip(lexer, e))]
pub fn eval<N>(
    lexer: &dyn NonStreamingLexer<DefaultLexerTypes<u32>>,
    e: Expr,
) -> Result<Vec<N>, (Span, &'static str)>
where
    N: CheckedAdd
        + CheckedSub
        + CheckedDiv
        + CheckedMul
        + CheckedNeg
        + Pow<usize, Output = N>
        + std::str::FromStr
        + std::fmt::Debug
        + Copy
        + TryInto<usize>
        + Ord
        + One
        + ToPrimitive
        + FromPrimitive,
{
    match e {
        Expr::Add { span, lhs, rhs } => {
            debug!("Dyadic Add");
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            let add_operation = |a: &N, b: &N| Ok(*a + *b);

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, add_operation)
        }
        Expr::Sub { span, lhs, rhs } => {
            debug!("Dyadic Sub");
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            let sub_operation = |a: &N, b: &N| Ok(*a - *b);

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, sub_operation)
        }
        Expr::Mul { span, lhs, rhs } => {
            debug!("Dyadic Mul");
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            let mul_operation = |a: &N, b: &N| Ok(*a * *b);

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, mul_operation)
        }
        Expr::Div { span, lhs, rhs } => {
            debug!("Dyadic Div");
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            let div_operation = |a: &N, b: &N| Ok(*a / *b);

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, div_operation)
        }
        Expr::Power { span, lhs, rhs } => {
            debug!("Dyadic Power");
            // raise left to the power of right
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            let pow_operation = |a: &N, b: &N| -> Result<N, &'static str> {
                let exponent = match TryInto::<usize>::try_into(*b) {
                    Ok(exp) => exp,
                    Err(e) => return Err("Exponent must be a non-negative integer: {e}"),
                };
                num_traits::pow::checked_pow(*a, exponent)
                    .ok_or("Exponentiation overflow or invalid operation")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, pow_operation)
        }
        Expr::Exp { span, arg } => {
            debug!("Monadic Exponential");
            // raise e to the power of arg
            let arg_eval = eval::<N>(lexer, *arg)?;

            let arg_float: Vec<f64> = arg_eval.iter().filter_map(|n| n.to_f64()).collect();

            let exp_operation = |a: &f64| -> Result<f64, &'static str> { Ok(a.exp()) };

            let results: Result<Vec<_>, _> =
                apply_monadic_operation(span, &arg_float, exp_operation)
                    .map(|vec| vec.iter().filter_map(|&n| N::from_f64(n)).collect());

            match results {
                Ok(vec) if vec.len() == arg_float.len() => Ok(vec),
                _ => Err((span, "conversion from floating-point failed")),
            }
        }
        Expr::Log { span, lhs, rhs } => {
            debug!("Dyadic Log");
            let lhs_eval = eval::<N>(lexer, *lhs)?;
            let rhs_eval = eval::<N>(lexer, *rhs)?;

            // Convert both operands to f64 for floating-point operations
            let lhs_float: Vec<f64> = lhs_eval.iter().filter_map(|n| n.to_f64()).collect();
            let rhs_float: Vec<f64> = rhs_eval.iter().filter_map(|n| n.to_f64()).collect();

            if lhs_float.len() != lhs_eval.len() || rhs_float.len() != rhs_eval.len() {
                return Err((span, "conversion to floating-point failed"));
            }

            let log_operation = |base: &f64, value: &f64| -> Result<f64, &'static str> {
                if *value > 0.0 && *base > 0.0 && *base != 1.0 {
                    Ok(value.log(*base))
                } else {
                    Err("logarithm undefined for non-positive base or value")
                }
            };

            // Apply operation and convert results back to N
            let results: Result<Vec<_>, _> =
                apply_dyadic_operation(span, &lhs_float, &rhs_float, log_operation)
                    .map(|vec| vec.iter().filter_map(|&n| N::from_f64(n)).collect());

            match results {
                Ok(vec) if vec.len() == lhs_float.len() => Ok(vec),
                _ => Err((span, "conversion from floating-point failed")),
            }
        }
        Expr::NaturalLog { span, arg } => {
            debug!("Monadic Natural Log");
            let arg_eval = eval::<N>(lexer, *arg)?;

            let arg_float: Vec<f64> = arg_eval.iter().filter_map(|n| n.to_f64()).collect();

            let nat_log_operation = |value: &f64| -> Result<f64, &'static str> {
                if *value > 0.0 {
                    Ok(value.ln())
                } else {
                    Err("logarithm undefined for non-positive base or value")
                }
            };

            let results: Result<Vec<_>, _> =
                apply_monadic_operation(span, &arg_float, nat_log_operation)
                    .map(|vec| vec.iter().filter_map(|&n| N::from_f64(n)).collect());

            match results {
                Ok(vec) if vec.len() == arg_float.len() => Ok(vec),
                _ => Err((span, "conversion from floating-point failed")),
            }
        }
        Expr::Conjugate { span, arg } => {
            debug!("Monadic Conjugate");
            // negates complex part of the number
            // real and non-numeric values remain the same
            // TODO: add support for imaginary numbers
            let arg_eval = eval::<N>(lexer, *arg)?;
            Ok(arg_eval)
        }
        Expr::Negate { span, arg } => {
            debug!("Monadic Negate");
            let arg_eval = eval::<N>(lexer, *arg)?;

            apply_monadic_operation(span, &arg_eval, |&n| {
                n.checked_neg().ok_or("Negation overflowed")
            })
        }
        Expr::Direction { span, arg } => {
            debug!("Monadic Direction");
            // For real numbers: returns -1, 0, or 1 for each number
            // TODO: add support for imaginary numbers
            let arg_eval = eval::<N>(lexer, *arg)?;

            let arg_float: Vec<f64> = arg_eval.iter().filter_map(|n| n.to_f64()).collect();

            fn direction_op(value: &f64) -> Result<f64, &'static str> {
                match value.partial_cmp(&0.0) {
                    Some(std::cmp::Ordering::Less) => Ok(-1.0),
                    Some(std::cmp::Ordering::Equal) => Ok(0.0),
                    Some(std::cmp::Ordering::Greater) => Ok(1.0),
                    None => Err("Comparison failed, possibly due to NaN"),
                }
            }

            let results: Result<Vec<_>, _> =
                apply_monadic_operation(span, &arg_float, direction_op)
                    .map(|vec| vec.iter().filter_map(|&n| N::from_f64(n)).collect());

            match results {
                Ok(vec) if vec.len() == arg_float.len() => Ok(vec),
                _ => Err((span, "conversion from floating-point failed")),
            }
        }
        Expr::Ceil { span, arg } => {
            debug!("Monadic Ceiling");
            // TODO: complete after float branching added
            let arg_eval = eval::<N>(lexer, *arg)?;

            // let ceil_operation = |a: &N| Ok(a.ceil());

            // apply_monadic_operation(span, &arg_eval, ceil_operation)

            Ok(arg_eval)
        }
        Expr::Floor { span, arg } => {
            debug!("Monadic Floor");
            // TODO: complete after float branching added
            let arg_eval = eval::<N>(lexer, *arg)?;

            // let floor_operation = |a: &N| Ok(a.floor());

            // apply_monadic_operation(span, &arg_eval, floor_operation)

            Ok(arg_eval)
        }
        Expr::Reciprocal { span, arg } => {
            debug!("Monadic Reciprocal");
            // Returns 1 ÷ arg
            // TODO
            let arg_eval = eval::<N>(lexer, *arg)?;

            // let floor_operation = |a: &N| Ok(a.floor());

            // apply_monadic_operation(span, &arg_eval, floor_operation)

            Ok(arg_eval)
        }
        Expr::Max { span, arg } => {
            debug!("Monadic Maximum");
            let arg_eval = eval::<N>(lexer, *arg)?;

            arg_eval
                .iter()
                .max()
                .ok_or((span, "Cannot find max"))
                .map(|&num| vec![num])
        }
        Expr::Min { span, arg } => {
            debug!("Monadic Minimum");
            let arg_eval = eval::<N>(lexer, *arg)?;

            arg_eval
                .iter()
                .min()
                .ok_or((span, "Cannot find max"))
                .map(|&num| vec![num])
        }
        Expr::Scalar { span, .. } => {
            debug!("Scalar");
            lexer
                .span_str(span)
                .parse::<N>()
                .map(|num| vec![num])
                .map_err(|_| (span, "cannot be represented as a valid number"))
        }
        Expr::Vector { span, elements } => {
            debug!("Vector");
            debug!(?elements, "Vector elements");

            let results: Vec<Result<Vec<N>, (Span, &'static str)>> = elements
                .into_iter()
                .map(|elem| eval::<N>(lexer, elem))
                .collect();

            if let Some(err) = results.iter().find_map(|r| r.as_ref().err()) {
                error!(?span, "Error in vector evaluation at span: {:?}", err);
                return Err(err.clone());
            }

            let flattened_results: Vec<N> = results.into_iter()
                .filter_map(Result::ok)
                .flatten()
                .collect();

            Ok(flattened_results)
        }
    }
}

fn check_lengths<N>(lhs_eval: &Vec<N>, rhs_eval: &Vec<N>) -> bool {
    lhs_eval.len() == rhs_eval.len() || lhs_eval.len() == 1 || rhs_eval.len() == 1
}
