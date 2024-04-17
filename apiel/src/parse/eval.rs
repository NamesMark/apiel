// MARK: eval
use super::*;
use crate::parse::apiel_y::Operator;
use val::{Val, CheckedPow, Log};
use eyre::{OptionExt, Result};
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub};
use rand::Rng;
use tracing::{debug, error};

fn apply_dyadic_operation<F>(
    span: Span,
    lhs: &[Val],
    rhs: &[Val],
    operation: F,
) -> Result<Vec<Val>, (Span, &'static str)>
where
    F: Fn(&Val, &Val) -> Result<Val>,
{
    match (lhs.len(), rhs.len()) {
        (1, _) => {
            // Scalar on lhs, vector on rhs
            rhs.iter()
                .map(|r| operation(&lhs[0], r))
                .collect::<Result<Vec<Val>, _>>()
                .map_err(|_| (span, "Operation failed"))
        }
        (_, 1) => {
            // Vector on lhs, scalar on rhs
            lhs.iter()
                .map(|l| operation(l, &rhs[0]))
                .collect::<Result<Vec<Val>, _>>()
                .map_err(|_| (span, "Operation failed"))
        }
        (_, _) if lhs.len() == rhs.len() => {
            // Both vectors of the same size
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| operation(l, r))
                .collect::<Result<Vec<Val>, _>>()
                .map_err(|_| (span, "Operation failed"))
        }
        // Bad vector sizes
        _ => Err((
            Span::new(0, 0),
            "operands must be of the same size or one must be scalar",
        )),
    }
}

fn apply_monadic_operation<F>(
    span: Span,
    arg: &[Val],
    operation: F,
) -> Result<Vec<Val>, (Span, &'static str)>
where
    F: Fn(&Val) -> Result<Val>,
{
    arg.iter()
        .map(operation)
        .collect::<Result<Vec<Val>>>()
        .map_err(|_| (span, "Operation failed"))
}

//#[tracing::instrument(skip(lexer, e))]
pub fn eval(
    lexer: &dyn NonStreamingLexer<DefaultLexerTypes<u32>>,
    e: Expr,
) -> Result<Vec<Val>, (Span, &'static str)> {
    match e {
        // MARK: Dyadic:
        Expr::Add { span, lhs, rhs } => {
            debug!("Dyadic Add");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let add_operation = |a: &Val, b: &Val| {
                a.checked_add(b)
                    .ok_or_eyre("Overflowed during addition of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, add_operation)
        }
        Expr::Sub { span, lhs, rhs } => {
            debug!("Dyadic Sub");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let sub_operation = |a: &Val, b: &Val| {
                a.checked_sub(b)
                    .ok_or_eyre("Overflowed during subtraction of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, sub_operation)
        }
        Expr::Mul { span, lhs, rhs } => {
            debug!("Dyadic Mul");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let mul_operation = |a: &Val, b: &Val| {
                a.checked_mul(b)
                    .ok_or_eyre("Overflowed during multiplication of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, mul_operation)
        }
        Expr::Div { span, lhs, rhs } => {
            debug!("Dyadic Div");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let div_operation = |a: &Val, b: &Val| {
                a.checked_div(b)
                    .ok_or_eyre("Overflowed during division of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, div_operation)
        }
        Expr::Power { span, lhs, rhs } => {
            debug!("Dyadic Power");
            // raise left to the power of right
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let pow_operation = |a: &Val, b: &Val| match TryInto::<usize>::try_into(*b) {
                Ok(int_exp) => a.checked_pow(int_exp).ok_or_eyre(format!(
                    "Exponentiation overflow or invalid operation for {a:?} and {int_exp:?}"
                )),
                Err(_) => a.checked_powf(f64::from(*b)).ok_or_eyre(format!(
                    "Exponentiation overflow or invalid operation for {a:?} and {b:?}"
                )),
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, pow_operation)
        }
        Expr::Log { span, lhs, rhs } => {
            debug!("Dyadic Log");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |base: &Val, value: &Val| {
                value.log(base).ok_or_eyre(format!(
                    "Somehow failed to compute the logarithm of {base:?} and {value:?}: {span:?}"
                ))
            })
        }
        Expr::Min { span, lhs, rhs } => {
            debug!("Dyadic Min");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let min_operation = |a: &Val, b: &Val| {
                let result = match a.cmp(b) {
                    std::cmp::Ordering::Greater => *b,
                    std::cmp::Ordering::Equal => *b,
                    std::cmp::Ordering::Less => *a,
                };
                Ok(result)
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, min_operation)
        }
        Expr::Max { span, lhs, rhs } => {
            debug!("Dyadic Max");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let max_operation = |a: &Val, b: &Val| {
                let result = match a.cmp(b) {
                    std::cmp::Ordering::Greater => *a,
                    std::cmp::Ordering::Equal => *a,
                    std::cmp::Ordering::Less => *b,
                };
                Ok(result)
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, max_operation)
        }
        Expr::Binomial { span, lhs, rhs } => {
            debug!("Dyadic Binominal");
            //  binomial coefficient between two arguments
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            fn gamma(n: f64) -> f64 {
                n.exp()
            }

            fn binomial(x: f64, y: f64) -> f64 {
                gamma(x + 1.0) / (gamma(y + 1.0) * gamma(x - y + 1.0))
            }

            let binomial_operation = |a: &Val, b: &Val| match (a, b) {
                (Val::Integer(n), Val::Integer(k)) if *n >= 0 && *k >= 0 => {
                    Ok(Val::Float(binomial(*n as f64, *k as f64)))
                }
                (Val::Float(x), Val::Float(y)) if *y >= 0.0 && x >= y => {
                    Ok(Val::Float(binomial(*x, *y)))
                }
                _ => eyre::bail!("Invalid input for binomial calculation"),
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, binomial_operation)
        }
        Expr::Deal { span, lhs, rhs } => {
            debug!("Dyadic Deal");
            // lhs random selections from rhs
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            if lhs_eval.len() > 1 || rhs_eval.len() > 1 {
                return Err((span, "Deal operation is only available for two scalars"));
            }

            let (lhs, rhs) = match (lhs_eval[0], rhs_eval[0]) {
                (Val::Integer(lhs), Val::Integer(rhs)) if rhs >= lhs => (lhs, rhs),
                _ => return Err((span, "Deal arguments must be integers; right argument must be greater than or equal to the left argument")),
            };

            let mut rng = rand::thread_rng();

            Ok((0..lhs)
                .map(|_| Val::Integer(rng.gen_range(0..=rhs)))
                .collect())
        }
        Expr::Residue { span, lhs, rhs } => {
            debug!("Dyadic Residue");
            // aka modulo
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let residue_operation = |a: &Val, b: &Val| match (&a, &b) {
                (Val::Integer(a), Val::Integer(b)) => Ok(Val::Integer(a % b)),
                (Val::Float(a), Val::Integer(b)) => Ok(Val::Float(a % *b as f64)),
                (Val::Integer(a), Val::Float(b)) => Ok(Val::Float(*a as f64 % b)),
                (Val::Float(a), Val::Float(b)) => Ok(Val::Float(a % b)),
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, residue_operation)
        }

        // MARK: Monadic
        Expr::Conjugate { span, arg } => {
            debug!("Monadic Conjugate");
            let _ = span;
            // negates complex part of the number
            // real and non-numeric values remain the same
            // TODO: add support for imaginary numbers
            let arg_eval = eval(lexer, *arg)?;
            Ok(arg_eval)
        }
        Expr::Negate { span, arg } => {
            debug!("Monadic Negate");
            let arg_eval = eval(lexer, *arg)?;

            apply_monadic_operation(span, &arg_eval, |&n| {
                n.checked_neg()
                    .ok_or_eyre(format!("Negation overflow or invalid operation for {n:?}"))
            })
        }
        Expr::Direction { span, arg } => {
            debug!("Monadic Direction");
            // For real numbers: returns -1, 0, or 1 for each number
            // TODO: add support for imaginary numbers
            let arg_eval = eval(lexer, *arg)?;

            fn direction_op(value: &Val) -> Result<Val> {
                match value.partial_cmp(&Val::Integer(0)) {
                    Some(std::cmp::Ordering::Less) => Ok(Val::Integer(-1)),
                    Some(std::cmp::Ordering::Equal) => Ok(Val::Integer(0)),
                    Some(std::cmp::Ordering::Greater) => Ok(Val::Integer(1)),
                    None => eyre::bail!("Comparison failed, possibly due to NaN"),
                }
            }

            apply_monadic_operation(span, &arg_eval, direction_op)
        }
        Expr::Reciprocal { span, arg } => {
            debug!("Monadic Reciprocal");
            // Returns 1 ÷ arg

            let arg_eval = eval(lexer, *arg)?;

            let reciprocal_operation = |a: &Val| {
                Val::Integer(1)
                    .checked_div(a)
                    .ok_or_eyre(format!("Invalid operation for {a:?}"))
            };

            apply_monadic_operation(span, &arg_eval, reciprocal_operation)
        }
        Expr::Exp { span, arg } => {
            debug!("Monadic Exponential");
            // Raises e to the power of arg

            let arg_eval = eval(lexer, *arg)?;

            let exp_operation = |a: &Val| match a {
                Val::Integer(val) => Ok(Val::Float((*val as f64).exp())),
                Val::Float(val) => Ok(Val::Float(val.exp())),
            };

            apply_monadic_operation(span, &arg_eval, exp_operation)
        }
        Expr::NaturalLog { span, arg } => {
            debug!("Monadic Natural Log");

            let arg_eval = eval(lexer, *arg)?;

            let nat_log_operation = |value: &Val| match value {
                Val::Integer(val) if *val > 0 => Ok(Val::Float((*val as f64).ln())),
                Val::Float(val) if *val > 0.0 => Ok(Val::Float(val.ln())),
                _ => eyre::bail!("logarithm undefined for non-positive base or value"),
            };

            apply_monadic_operation(span, &arg_eval, nat_log_operation)
        }
        Expr::PiMultiple { span, arg } => {
            debug!("Monadic Pi Multiple");
            let arg_eval = eval(lexer, *arg)?;

            let pi_multiple_operation = |a: &Val| match a {
                Val::Integer(i) => Ok(Val::Float(*i as f64 * std::f64::consts::PI)),
                Val::Float(f) => Ok(Val::Float(*f * std::f64::consts::PI)),
            };

            apply_monadic_operation(span, &arg_eval, pi_multiple_operation)
        }
        Expr::Factorial { span, arg } => {
            debug!("Monadic Factorial");
            let arg_eval = eval(lexer, *arg)?;

            let factorial_operation = |a: &Val| match a {
                Val::Integer(i) if *i >= 0 => {
                    let mut acc = 1;
                    for x in 1..=*i {
                        acc = acc
                            .checked_mul(&x)
                            .ok_or_else(|| eyre::eyre!("Factorial overflow"))?;
                    }
                    Ok(Val::Integer(acc))
                }
                Val::Float(f) if f.fract() == 0.0 && *f >= 0.0 => {
                    let mut acc = 1.0_f64;
                    for x in 1..=(*f as i64) {
                        acc *= x as f64;
                    }
                    Ok(Val::Float(acc))
                }
                _ => eyre::bail!("Factorial not defined for negative numbers or non-integers"),
            };

            apply_monadic_operation(span, &arg_eval, factorial_operation)
        }
        Expr::Roll { span, arg } => {
            debug!("Monadic Roll");
            let arg_eval = eval(lexer, *arg)?;

            let roll_operation = |limit: &Val| {
                let mut rng = rand::thread_rng();

                match limit {
                    Val::Integer(val) if *val == 0 => Ok(Val::Integer(rng.gen())),
                    Val::Integer(val) => Ok(Val::Integer(rng.gen_range(0..=*val))),
                    Val::Float(_) => {
                        eyre::bail!("Roll right argument must consist of non-negative integer(s)")
                    }
                }
            };

            apply_monadic_operation(span, &arg_eval, roll_operation)
        }
        Expr::Magnitude { span, arg } => {
            debug!("Monadic Roll");
            let arg_eval = eval(lexer, *arg)?;

            let magnitude_operation = |value: &Val| match value {
                Val::Integer(val) => Ok(Val::Integer(val.abs())),
                Val::Float(val) => Ok(Val::Float(val.abs())),
            };

            apply_monadic_operation(span, &arg_eval, magnitude_operation)
        }
        Expr::Ceil { span, arg } => {
            debug!("Monadic Ceiling");

            let arg_eval = eval(lexer, *arg)?;

            let ceil_operation = |a: &Val| match a {
                Val::Integer(i) => Ok(Val::Integer(*i)),
                Val::Float(f) => Ok(Val::Float(f.ceil())),
            };

            apply_monadic_operation(span, &arg_eval, ceil_operation)
        }
        Expr::Floor { span, arg } => {
            debug!("Monadic Floor");

            let arg_eval = eval(lexer, *arg)?;

            let floor_operation = |a: &Val| match a {
                Val::Integer(i) => Ok(Val::Integer(*i)),
                Val::Float(f) => Ok(Val::Float(f.floor())),
            };

            apply_monadic_operation(span, &arg_eval, floor_operation)
        }
        Expr::MonadicMax { span, arg } => {
            debug!("Monadic Maximum");
            let arg_eval = eval(lexer, *arg)?;

            arg_eval
                .iter()
                .max()
                .ok_or((span, "Cannot find max"))
                .map(|&num| vec![num])
        }
        Expr::MonadicMin { span, arg } => {
            debug!("Monadic Minimum");
            let arg_eval = eval(lexer, *arg)?;

            arg_eval
                .iter()
                .min()
                .ok_or((span, "Cannot find max"))
                .map(|&num| vec![num])
        }
        Expr::GenIndex { span, arg } => {
            debug!("Monadic Iota: generate index");
            let arg_eval = eval(lexer, *arg)?;

            if arg_eval.len() > 1 {
                return Err((span, "Generate index only one integer at the moment"));
            }

            match arg_eval[0] {
                Val::Integer(i) if i >= 0 => Ok((1..=i).map(Val::Integer).collect()),
                Val::Integer(_) => Err((
                    span,
                    "Generate index only accepts non-negative integer values as right operand",
                )),
                Val::Float(_) => Err((
                    span,
                    "Generate index only accepts non-negative integer values as right operand",
                )),
            }

            // TODO: implement multidimensional version
            // let generate_index_operation = |a: &ValArray| match a {
            //     ValArray::Single(Val::Integer(i)) if *i >= 0 => Ok(iota(*i)),
            //     ValArray::Array(vec) if vec.iter().all(|v| matches!(v, ValArray::Single(Val::Integer(_)))) => {
            //         let dims: Vec<i64> = vec.iter().map(|v| if let ValArray::Single(Val::Integer(i)) = v { *i } else { 0 }).collect();
            //         Ok(multidimensional_iota(&dims))
            //     },
            //     _ => eyre::bail!("Generate index only accepts non-negative integers or vectors of integers as right operand"),
            // };

            //apply_monadic_operation(span, &arg_eval, generate_index_operation)
        }
        Expr::Where { arg, .. } => {
            debug!("Monadic Where");
            let arg_eval = eval(lexer, *arg)?;

            let result = arg_eval
                .iter()
                .enumerate()
                .flat_map(|(index, val)| match val {
                    Val::Integer(i) if *i > 0 => vec![index as i64 + 1; *i as usize]
                        .into_iter()
                        .map(Val::Integer),
                    Val::Float(f) if *f > 0.0 => vec![index as i64 + 1; *f as usize]
                        .into_iter()
                        .map(Val::Integer),
                    _ => vec![].into_iter().map(Val::Integer),
                })
                .collect::<Vec<Val>>();

            Ok(result)
        }
        Expr::Reduce {
            span,
            operator,
            term,
        } => {
            debug!("Reduce");
            let term_eval = eval(lexer, *term)?;

            let result = match operator {
                Operator::Add => term_eval.iter().skip(1).try_fold(
                    term_eval.first().cloned().unwrap_or(Val::Integer(0)),
                    |acc, n| match (acc, n) {
                        (Val::Integer(a), Val::Integer(b)) => a.checked_add(*b).map(Val::Integer),
                        (Val::Float(a), Val::Float(b)) => Some(Val::Float(a + b)),
                        (Val::Integer(a), Val::Float(b)) => Some(Val::Float(a as f64 + b)),
                        (Val::Float(a), Val::Integer(b)) => Some(Val::Float(a + *b as f64)),
                    },
                ),
                Operator::Subtract => term_eval.iter().skip(1).try_fold(
                    term_eval.first().cloned().unwrap_or(Val::Integer(1)),
                    |acc, n| match (acc, n) {
                        (Val::Integer(a), Val::Integer(b)) => a.checked_sub(*b).map(Val::Integer),
                        (Val::Float(a), Val::Float(b)) => Some(Val::Float(a - b)),
                        (Val::Integer(a), Val::Float(b)) => Some(Val::Float(a as f64 - b)),
                        (Val::Float(a), Val::Integer(b)) => Some(Val::Float(a - *b as f64)),
                    },
                ),
                Operator::Multiply => {
                    term_eval
                        .into_iter()
                        .try_fold(Val::Integer(1), |acc, n| match (acc, n) {
                            (Val::Integer(a), Val::Integer(b)) => {
                                a.checked_mul(b).map(Val::Integer)
                            }
                            (Val::Float(a), Val::Float(b)) => Some(Val::Float(a * b)),
                            (Val::Integer(a), Val::Float(b)) => Some(Val::Float(a as f64 * b)),
                            (Val::Float(a), Val::Integer(b)) => Some(Val::Float(a * b as f64)),
                        })
                }
                Operator::Divide => term_eval.iter().skip(1).try_fold(
                    term_eval.first().cloned().unwrap_or(Val::Integer(1)),
                    |acc, n| match (acc, n) {
                        (Val::Integer(a), Val::Integer(b)) => a.checked_div(*b).map(Val::Integer),
                        (Val::Float(a), Val::Float(b)) => {
                            if *b != 0.0 {
                                Some(Val::Float(a / b))
                            } else {
                                None
                            }
                        }
                        (Val::Integer(a), Val::Float(b)) => {
                            if *b != 0.0 {
                                Some(Val::Float(a as f64 / b))
                            } else {
                                None
                            }
                        }
                        (Val::Float(a), Val::Integer(b)) => {
                            if *b != 0 {
                                Some(Val::Float(a / *b as f64))
                            } else {
                                None
                            }
                        }
                    },
                ),
            };

            result
                .map(|r| vec![r])
                .ok_or((span, "Arithmetic error or invalid operation in Reduce"))
        }

        // MARK: Values
        Expr::ScalarFloat { span, .. } => {
            debug!("Scalar Float");
            lexer
                .span_str(span)
                .parse::<f64>()
                .map(|num| vec![Val::Float(num)])
                .map_err(|_| (span, "cannot be represented as a valid number"))
        }
        Expr::ScalarInteger { span, .. } => {
            debug!("Scalar Integer");

            lexer
                .span_str(span)
                .parse::<i64>()
                .map(|num| vec![Val::Integer(num)])
                .map_err(|_| (span, "cannot be represented as a valid number"))
        }
        Expr::Vector { span, elements } => {
            debug!("Vector");
            debug!(?elements, "Vector elements");

            let results: Vec<Result<Vec<Val>, (Span, &'static str)>> =
                elements.into_iter().map(|elem| eval(lexer, elem)).collect();

            if let Some(err) = results.iter().find_map(|r| r.as_ref().err()) {
                error!(?span, "Error in vector evaluation at span: {:?}", err);
                return Err(*err);
            }

            let flattened_results: Vec<Val> = results
                .into_iter()
                .filter_map(Result::ok)
                .flatten()
                .collect();

            Ok(flattened_results)
        }
    }
}

// fn check_lengths<N>(lhs_eval: &Vec<N>, rhs_eval: &Vec<N>) -> bool {
//     lhs_eval.len() == rhs_eval.len() || lhs_eval.len() == 1 || rhs_eval.len() == 1
// }

// fn iota(n: i64) -> ValArray {
//     ValArray::Array((1..=n).map(|x| ValArray::Single(Val::Integer(x))).collect())
// }

// fn multidimensional_iota(dims: &[i64]) -> ValArray {
//     if dims.len() == 1 {
//         iota(dims[0])
//     } else {
//         let head = dims[0];
//         let tail = &dims[1..];
//         ValArray::Array((1..=head).map(|_| multidimensional_iota(tail)).collect())
//     }
// }