use super::*;
use crate::parse::apiel_y::Operator;
use val::{Scalar, Val, CheckedPow, Log};
use eyre::{OptionExt, Result};
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub};
use rand::Rng;
use tracing::{debug, error};

fn apply_dyadic_operation<F>(
    span: Span,
    lhs: &Val,
    rhs: &Val,
    operation: F,
) -> Result<Val, (Span, &'static str)>
where
    F: Fn(&Scalar, &Scalar) -> Result<Scalar>,
{
    if lhs.is_scalar() {
        let data = rhs.data.iter()
            .map(|r| operation(&lhs.data[0], r))
            .collect::<Result<Vec<Scalar>, _>>()
            .map_err(|_| (span, "Operation failed"))?;
        Ok(Val::new(rhs.shape.clone(), data))
    } else if rhs.is_scalar() {
        let data = lhs.data.iter()
            .map(|l| operation(l, &rhs.data[0]))
            .collect::<Result<Vec<Scalar>, _>>()
            .map_err(|_| (span, "Operation failed"))?;
        Ok(Val::new(lhs.shape.clone(), data))
    } else if lhs.shape == rhs.shape {
        let data = lhs.data.iter()
            .zip(rhs.data.iter())
            .map(|(l, r)| operation(l, r))
            .collect::<Result<Vec<Scalar>, _>>()
            .map_err(|_| (span, "Operation failed"))?;
        Ok(Val::new(lhs.shape.clone(), data))
    } else {
        Err((
            Span::new(0, 0),
            "operands must be of the same shape or one must be scalar",
        ))
    }
}

fn apply_monadic_operation<F>(
    span: Span,
    arg: &Val,
    operation: F,
) -> Result<Val, (Span, &'static str)>
where
    F: Fn(&Scalar) -> Result<Scalar>,
{
    let data = arg.data.iter()
        .map(operation)
        .collect::<Result<Vec<Scalar>>>()
        .map_err(|_| (span, "Operation failed"))?;
    Ok(Val::new(arg.shape.clone(), data))
}

pub fn eval(
    lexer: &dyn NonStreamingLexer<DefaultLexerTypes<u32>>,
    e: Expr,
) -> Result<Val, (Span, &'static str)> {
    match e {
        Expr::Add { span, lhs, rhs } => {
            debug!("Dyadic Add");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let add_operation = |a: &Scalar, b: &Scalar| {
                a.checked_add(b)
                    .ok_or_eyre("Overflowed during addition of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, add_operation)
        }
        Expr::Sub { span, lhs, rhs } => {
            debug!("Dyadic Sub");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let sub_operation = |a: &Scalar, b: &Scalar| {
                a.checked_sub(b)
                    .ok_or_eyre("Overflowed during subtraction of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, sub_operation)
        }
        Expr::Mul { span, lhs, rhs } => {
            debug!("Dyadic Mul");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let mul_operation = |a: &Scalar, b: &Scalar| {
                a.checked_mul(b)
                    .ok_or_eyre("Overflowed during multiplication of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, mul_operation)
        }
        Expr::Div { span, lhs, rhs } => {
            debug!("Dyadic Div");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let div_operation = |a: &Scalar, b: &Scalar| {
                a.checked_div(b)
                    .ok_or_eyre("Overflowed during division of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, div_operation)
        }
        Expr::Power { span, lhs, rhs } => {
            debug!("Dyadic Power");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let pow_operation = |a: &Scalar, b: &Scalar| match TryInto::<usize>::try_into(*b) {
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

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |base: &Scalar, value: &Scalar| {
                value.log(base).ok_or_eyre(format!(
                    "Somehow failed to compute the logarithm of {base:?} and {value:?}: {span:?}"
                ))
            })
        }
        Expr::Min { span, lhs, rhs } => {
            debug!("Dyadic Min");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let min_operation = |a: &Scalar, b: &Scalar| {
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

            let max_operation = |a: &Scalar, b: &Scalar| {
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
            debug!("Dyadic Binomial");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            // APL: k ! n = C(n, k) = n! / (k! * (n-k)!)
            // lhs is k, rhs is n
            fn binomial(n: f64, k: f64) -> f64 {
                if k < 0.0 || k > n {
                    return 0.0;
                }
                let k = if k > n - k { n - k } else { k };
                let mut result = 1.0;
                for i in 0..k as i64 {
                    result *= (n - i as f64) / (i as f64 + 1.0);
                }
                result
            }

            let binomial_operation = |a: &Scalar, b: &Scalar| {
                let k = f64::from(*a);
                let n = f64::from(*b);
                Ok(Scalar::Float(binomial(n, k)))
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, binomial_operation)
        }
        Expr::Deal { span, lhs, rhs } => {
            debug!("Dyadic Deal");
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            if !lhs_eval.is_scalar() || !rhs_eval.is_scalar() {
                return Err((span, "Deal operation is only available for two scalars"));
            }

            let (lhs, rhs) = match (lhs_eval.data[0], rhs_eval.data[0]) {
                (Scalar::Integer(lhs), Scalar::Integer(rhs)) => (lhs, rhs),
                _ => return Err((span, "Deal arguments must be integers")),
            };

            let mut rng = rand::thread_rng();
            let data: Vec<Scalar> = (0..lhs)
                .map(|_| Scalar::Integer(rng.gen_range(0..=rhs)))
                .collect();
            Ok(Val::vector(data))
        }
        Expr::Residue { span, lhs, rhs } => {
            debug!("Dyadic Residue");
            // APL: B|A means A mod B (rhs mod lhs)
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;

            let residue_operation = |a: &Scalar, b: &Scalar| match (&a, &b) {
                (Scalar::Integer(a), Scalar::Integer(b)) => Ok(Scalar::Integer(b % a)),
                (Scalar::Float(a), Scalar::Integer(b)) => Ok(Scalar::Float(*b as f64 % a)),
                (Scalar::Integer(a), Scalar::Float(b)) => Ok(Scalar::Float(b % *a as f64)),
                (Scalar::Float(a), Scalar::Float(b)) => Ok(Scalar::Float(b % a)),
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, residue_operation)
        }
        Expr::IndexOf { span, lhs, rhs } => {
            debug!("Dyadic Index Of");
            // A ⍳ B — for each element of B, find its 1-based position in A.
            // If not found, returns 1 + length of A.
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;
            let _ = span;
            let not_found = lhs_eval.data.len() as i64 + 1;

            let data = rhs_eval.data.iter()
                .map(|needle| {
                    let pos = lhs_eval.data.iter()
                        .position(|hay| hay == needle)
                        .map(|i| i as i64 + 1)
                        .unwrap_or(not_found);
                    Scalar::Integer(pos)
                })
                .collect();

            Ok(Val::new(rhs_eval.shape, data))
        }
        Expr::IntervalIndex { span, lhs, rhs } => {
            debug!("Dyadic Interval Index");
            // A ⍸ B — for each element of B, count how many elements of A are ≤ it.
            // A must be sorted ascending.
            let lhs_eval = eval(lexer, *lhs)?;
            let rhs_eval = eval(lexer, *rhs)?;
            let _ = span;

            let data = rhs_eval.data.iter()
                .map(|val| {
                    let count = lhs_eval.data.iter().filter(|&a| a <= val).count();
                    Scalar::Integer(count as i64)
                })
                .collect();

            Ok(Val::new(rhs_eval.shape, data))
        }
        Expr::Conjugate { span, arg } => {
            debug!("Monadic Conjugate");
            let _ = span;
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
            let arg_eval = eval(lexer, *arg)?;

            fn direction_op(value: &Scalar) -> Result<Scalar> {
                match value.partial_cmp(&Scalar::Integer(0)) {
                    Some(std::cmp::Ordering::Less) => Ok(Scalar::Integer(-1)),
                    Some(std::cmp::Ordering::Equal) => Ok(Scalar::Integer(0)),
                    Some(std::cmp::Ordering::Greater) => Ok(Scalar::Integer(1)),
                    None => eyre::bail!("Comparison failed, possibly due to NaN"),
                }
            }

            apply_monadic_operation(span, &arg_eval, direction_op)
        }
        Expr::Reciprocal { span, arg } => {
            debug!("Monadic Reciprocal");
            let arg_eval = eval(lexer, *arg)?;

            let reciprocal_operation = |a: &Scalar| {
                Scalar::Integer(1)
                    .checked_div(a)
                    .ok_or_eyre(format!("Invalid operation for {a:?}"))
            };

            apply_monadic_operation(span, &arg_eval, reciprocal_operation)
        }
        Expr::Exp { span, arg } => {
            debug!("Monadic Exponential");
            let arg_eval = eval(lexer, *arg)?;

            let exp_operation = |a: &Scalar| match a {
                Scalar::Integer(val) => Ok(Scalar::Float((*val as f64).exp())),
                Scalar::Float(val) => Ok(Scalar::Float(val.exp())),
            };

            apply_monadic_operation(span, &arg_eval, exp_operation)
        }
        Expr::NaturalLog { span, arg } => {
            debug!("Monadic Natural Log");
            let arg_eval = eval(lexer, *arg)?;

            let nat_log_operation = |value: &Scalar| match value {
                Scalar::Integer(val) if *val > 0 => Ok(Scalar::Float((*val as f64).ln())),
                Scalar::Float(val) if *val > 0.0 => Ok(Scalar::Float(val.ln())),
                _ => eyre::bail!("logarithm undefined for non-positive values"),
            };

            apply_monadic_operation(span, &arg_eval, nat_log_operation)
        }
        Expr::PiMultiple { span, arg } => {
            debug!("Monadic Pi Multiple");
            let arg_eval = eval(lexer, *arg)?;

            let pi_multiple_operation = |a: &Scalar| match a {
                Scalar::Integer(i) => Ok(Scalar::Float(*i as f64 * std::f64::consts::PI)),
                Scalar::Float(f) => Ok(Scalar::Float(*f * std::f64::consts::PI)),
            };

            apply_monadic_operation(span, &arg_eval, pi_multiple_operation)
        }
        Expr::Factorial { span, arg } => {
            debug!("Monadic Factorial");
            let arg_eval = eval(lexer, *arg)?;

            let factorial_operation = |a: &Scalar| match a {
                Scalar::Integer(i) if *i >= 0 => {
                    let mut acc = 1;
                    for x in 1..=*i {
                        acc = acc
                            .checked_mul(&x)
                            .ok_or_else(|| eyre::eyre!("Factorial overflow"))?;
                    }
                    Ok(Scalar::Integer(acc))
                }
                Scalar::Float(f) if f.fract() == 0.0 && *f >= 0.0 => {
                    let mut acc = 1.0_f64;
                    for x in 1..=(*f as i64) {
                        acc *= x as f64;
                    }
                    Ok(Scalar::Float(acc))
                }
                _ => eyre::bail!("Factorial not defined for negative numbers or non-integers"),
            };

            apply_monadic_operation(span, &arg_eval, factorial_operation)
        }
        Expr::Roll { span, arg } => {
            debug!("Monadic Roll");
            let arg_eval = eval(lexer, *arg)?;

            let roll_operation = |limit: &Scalar| {
                let mut rng = rand::thread_rng();

                match limit {
                    Scalar::Integer(val) if *val == 0 => Ok(Scalar::Integer(rng.r#gen())),
                    Scalar::Integer(val) => Ok(Scalar::Integer(rng.gen_range(0..=*val))),
                    Scalar::Float(_) => {
                        eyre::bail!("Roll right argument must consist of non-negative integer(s)")
                    }
                }
            };

            apply_monadic_operation(span, &arg_eval, roll_operation)
        }
        Expr::Magnitude { span, arg } => {
            debug!("Monadic Magnitude");
            let arg_eval = eval(lexer, *arg)?;

            let magnitude_operation = |value: &Scalar| match value {
                Scalar::Integer(val) => Ok(Scalar::Integer(val.abs())),
                Scalar::Float(val) => Ok(Scalar::Float(val.abs())),
            };

            apply_monadic_operation(span, &arg_eval, magnitude_operation)
        }
        Expr::Ceil { span, arg } => {
            debug!("Monadic Ceiling");
            let arg_eval = eval(lexer, *arg)?;

            let ceil_operation = |a: &Scalar| match a {
                Scalar::Integer(i) => Ok(Scalar::Integer(*i)),
                Scalar::Float(f) => Ok(Scalar::Float(f.ceil())),
            };

            apply_monadic_operation(span, &arg_eval, ceil_operation)
        }
        Expr::Floor { span, arg } => {
            debug!("Monadic Floor");
            let arg_eval = eval(lexer, *arg)?;

            let floor_operation = |a: &Scalar| match a {
                Scalar::Integer(i) => Ok(Scalar::Integer(*i)),
                Scalar::Float(f) => Ok(Scalar::Float(f.floor())),
            };

            apply_monadic_operation(span, &arg_eval, floor_operation)
        }
        Expr::MonadicMax { span, arg } => {
            debug!("Monadic Maximum");
            let arg_eval = eval(lexer, *arg)?;

            arg_eval.data.iter()
                .max()
                .ok_or((span, "Cannot find max"))
                .map(|&num| Val::scalar(num))
        }
        Expr::MonadicMin { span, arg } => {
            debug!("Monadic Minimum");
            let arg_eval = eval(lexer, *arg)?;

            arg_eval.data.iter()
                .min()
                .ok_or((span, "Cannot find min"))
                .map(|&num| Val::scalar(num))
        }
        Expr::GenIndex { span, arg } => {
            debug!("Monadic Iota: generate index");
            let arg_eval = eval(lexer, *arg)?;

            if !arg_eval.is_scalar() {
                return Err((span, "Generate index only accepts a scalar integer"));
            }

            match arg_eval.data[0] {
                Scalar::Integer(i) if i >= 0 => {
                    let data: Vec<Scalar> = (1..=i).map(Scalar::Integer).collect();
                    Ok(Val::vector(data))
                }
                _ => Err((
                    span,
                    "Generate index only accepts non-negative integer values as right operand",
                )),
            }
        }
        Expr::Where { arg, .. } => {
            debug!("Monadic Where");
            let arg_eval = eval(lexer, *arg)?;

            let data: Vec<Scalar> = arg_eval.data.iter()
                .enumerate()
                .flat_map(|(index, val)| match val {
                    Scalar::Integer(i) if *i > 0 => vec![index as i64 + 1; *i as usize]
                        .into_iter()
                        .map(Scalar::Integer),
                    Scalar::Float(f) if *f > 0.0 => vec![index as i64 + 1; *f as usize]
                        .into_iter()
                        .map(Scalar::Integer),
                    _ => vec![].into_iter().map(Scalar::Integer),
                })
                .collect();

            Ok(Val::vector(data))
        }
        Expr::Reduce {
            span,
            operator,
            term,
        } => {
            debug!("Reduce");
            let term_eval = eval(lexer, *term)?;

            // APL reduce is a right-fold: f/ a b c d = a f (b f (c f d))
            let op_fn: fn(&Scalar, &Scalar) -> Option<Scalar> = match operator {
                Operator::Add => |a, b| a.checked_add(b),
                Operator::Subtract => |a, b| a.checked_sub(b),
                Operator::Multiply => |a, b| a.checked_mul(b),
                Operator::Divide => |a, b| a.checked_div(b),
            };

            let result = term_eval.data.iter().rev().copied().try_fold(None, |acc, n| {
                match acc {
                    None => Some(Some(n)),
                    Some(right) => op_fn(&n, &right).map(Some),
                }
            }).flatten();

            result
                .map(Val::scalar)
                .ok_or((span, "Arithmetic error or invalid operation in Reduce"))
        }
        Expr::ScalarFloat { span, .. } => {
            debug!("Scalar Float");
            lexer
                .span_str(span)
                .replace('¯', "-")
                .parse::<f64>()
                .map(|num| Val::scalar(Scalar::Float(num)))
                .map_err(|_| (span, "cannot be represented as a valid number"))
        }
        Expr::ScalarInteger { span, .. } => {
            debug!("Scalar Integer");
            lexer
                .span_str(span)
                .replace('¯', "-")
                .parse::<i64>()
                .map(|num| Val::scalar(Scalar::Integer(num)))
                .map_err(|_| (span, "cannot be represented as a valid number"))
        }
        Expr::Vector { span, elements } => {
            debug!("Vector");
            debug!(?elements, "Vector elements");

            let results: Vec<Result<Val, (Span, &'static str)>> =
                elements.into_iter().map(|elem| eval(lexer, elem)).collect();

            if let Some(err) = results.iter().find_map(|r| r.as_ref().err()) {
                error!(?span, "Error in vector evaluation at span: {:?}", err);
                return Err(*err);
            }

            let data: Vec<Scalar> = results
                .into_iter()
                .filter_map(Result::ok)
                .flat_map(|v| v.data)
                .collect();

            Ok(Val::vector(data))
        }
    }
}
