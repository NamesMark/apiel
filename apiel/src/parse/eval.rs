use super::*;
use crate::parse::apiel_y::{Expr, Operator};
use eyre::{OptionExt, Result};
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub};
use rand::Rng;
use std::collections::HashMap;
use std::rc::Rc;
use tracing::{debug, error};
use val::{CheckedPow, Log, Scalar, Val};

#[derive(Debug, Clone)]
pub struct StoredDfn {
    pub body: Rc<Expr>,
    pub source: String, // original input line for correct span resolution
}

#[derive(Debug, Clone, Default)]
pub struct Env {
    pub vars: HashMap<String, Val>,
    pub fns: HashMap<String, StoredDfn>,
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }
}

fn eval_stored_dfn(stored: &StoredDfn, env: &mut Env) -> Result<Val, (Span, String)> {
    use crate::parse::apiel_l;
    let lexerdef = apiel_l::lexerdef();
    let lex = lexerdef.lexer(&stored.source);
    eval(&lex, (*stored.body).clone(), env).map_err(|(span, msg)| (span, msg.to_string()))
}

fn apply_dyadic_operation<F>(
    span: Span,
    lhs: &Val,
    rhs: &Val,
    operation: F,
) -> Result<Val, (Span, &'static str)>
where
    F: Fn(&Scalar, &Scalar) -> Result<Scalar>,
{
    // Treat 1-element arrays as scalars for broadcasting (standard APL behavior)
    let lhs_scalar = lhs.data.len() == 1;
    let rhs_scalar = rhs.data.len() == 1;
    if lhs_scalar && !rhs_scalar {
        let data = rhs
            .data
            .iter()
            .map(|r| operation(&lhs.data[0], r))
            .collect::<Result<Vec<Scalar>, _>>()
            .map_err(|_| (span, "Operation failed"))?;
        Ok(Val::new(rhs.shape.clone(), data))
    } else if rhs_scalar && !lhs_scalar {
        let data = lhs
            .data
            .iter()
            .map(|l| operation(l, &rhs.data[0]))
            .collect::<Result<Vec<Scalar>, _>>()
            .map_err(|_| (span, "Operation failed"))?;
        Ok(Val::new(lhs.shape.clone(), data))
    } else if lhs.shape == rhs.shape || (lhs_scalar && rhs_scalar) {
        let data = lhs
            .data
            .iter()
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
    let data = arg
        .data
        .iter()
        .map(operation)
        .collect::<Result<Vec<Scalar>>>()
        .map_err(|_| (span, "Operation failed"))?;
    Ok(Val::new(arg.shape.clone(), data))
}

fn get_operator_fn(op: Operator) -> fn(&Scalar, &Scalar) -> Option<Scalar> {
    match op {
        Operator::Add => |a, b| a.checked_add(b),
        Operator::Subtract => |a, b| a.checked_sub(b),
        Operator::Multiply => |a, b| a.checked_mul(b),
        Operator::Divide => |a, b| a.checked_div(b),
        Operator::Equal => |a, b| Some(Scalar::Integer(if a == b { 1 } else { 0 })),
        Operator::NotEqual => |a, b| Some(Scalar::Integer(if a != b { 1 } else { 0 })),
        Operator::LessThan => |a, b| Some(Scalar::Integer(if a < b { 1 } else { 0 })),
        Operator::GreaterThan => |a, b| Some(Scalar::Integer(if a > b { 1 } else { 0 })),
        Operator::LessEqual => |a, b| Some(Scalar::Integer(if a <= b { 1 } else { 0 })),
        Operator::GreaterEqual => |a, b| Some(Scalar::Integer(if a >= b { 1 } else { 0 })),
        Operator::Max => |a, b| Some(if a >= b { a.clone() } else { b.clone() }),
        Operator::Min => |a, b| Some(if a <= b { a.clone() } else { b.clone() }),
        Operator::And => |a, b| {
            let af: f64 = a.clone().into();
            let bf: f64 = b.clone().into();
            Some(Scalar::Integer(if af != 0.0 && bf != 0.0 { 1 } else { 0 }))
        },
        Operator::Or => |a, b| {
            let af: f64 = a.clone().into();
            let bf: f64 = b.clone().into();
            Some(Scalar::Integer(if af != 0.0 || bf != 0.0 { 1 } else { 0 }))
        },
        Operator::Nand => |a, b| {
            let af: f64 = a.clone().into();
            let bf: f64 = b.clone().into();
            Some(Scalar::Integer(if af != 0.0 && bf != 0.0 { 0 } else { 1 }))
        },
        Operator::Nor => |a, b| {
            let af: f64 = a.clone().into();
            let bf: f64 = b.clone().into();
            Some(Scalar::Integer(if af != 0.0 || bf != 0.0 { 0 } else { 1 }))
        },
        Operator::Power => |a, b| {
            let af: f64 = a.clone().into();
            let bf: f64 = b.clone().into();
            let result = af.powf(bf);
            if result.fract() == 0.0 && result.abs() < i64::MAX as f64 {
                Some(Scalar::Integer(result as i64))
            } else {
                Some(Scalar::Float(result))
            }
        },
        Operator::Log => |a, b| {
            let af: f64 = a.clone().into();
            let bf: f64 = b.clone().into();
            Some(Scalar::Float(bf.ln() / af.ln()))
        },
        Operator::Residue => |a, b| {
            let af: f64 = a.clone().into();
            let bf: f64 = b.clone().into();
            if af == 0.0 {
                Some(Scalar::Float(bf))
            } else {
                let r = bf % af;
                if r.fract() == 0.0 && r.abs() < i64::MAX as f64 {
                    Some(Scalar::Integer(r as i64))
                } else {
                    Some(Scalar::Float(r))
                }
            }
        },
        Operator::Binomial => |a, b| {
            let n: f64 = b.clone().into();
            let k: f64 = a.clone().into();
            let mut result = 1.0_f64;
            for i in 0..k as u64 {
                result *= (n - i as f64) / (i as f64 + 1.0);
            }
            if result.fract() == 0.0 && result.abs() < i64::MAX as f64 {
                Some(Scalar::Integer(result as i64))
            } else {
                Some(Scalar::Float(result))
            }
        },
    }
}

pub fn eval(
    lexer: &dyn NonStreamingLexer<DefaultLexerTypes<u32>>,
    e: Expr,
    env: &mut Env,
) -> Result<Val, (Span, &'static str)> {
    match e {
        Expr::Add { span, lhs, rhs } => {
            debug!("Dyadic Add");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let add_operation = |a: &Scalar, b: &Scalar| {
                a.checked_add(b)
                    .ok_or_eyre("Overflowed during addition of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, add_operation)
        }
        Expr::Sub { span, lhs, rhs } => {
            debug!("Dyadic Sub");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let sub_operation = |a: &Scalar, b: &Scalar| {
                a.checked_sub(b)
                    .ok_or_eyre("Overflowed during subtraction of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, sub_operation)
        }
        Expr::Mul { span, lhs, rhs } => {
            debug!("Dyadic Mul");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let mul_operation = |a: &Scalar, b: &Scalar| {
                a.checked_mul(b)
                    .ok_or_eyre("Overflowed during multiplication of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, mul_operation)
        }
        Expr::Div { span, lhs, rhs } => {
            debug!("Dyadic Div");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let div_operation = |a: &Scalar, b: &Scalar| {
                a.checked_div(b)
                    .ok_or_eyre("Overflowed during division of {a} and {b}")
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, div_operation)
        }
        Expr::Power { span, lhs, rhs } => {
            debug!("Dyadic Power");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let pow_operation = |a: &Scalar, b: &Scalar| match TryInto::<usize>::try_into(b.clone())
            {
                Ok(int_exp) => a.checked_pow(int_exp).ok_or_eyre(format!(
                    "Exponentiation overflow or invalid operation for {a:?} and {int_exp:?}"
                )),
                Err(_) => a.checked_powf(f64::from(b.clone())).ok_or_eyre(format!(
                    "Exponentiation overflow or invalid operation for {a:?} and {b:?}"
                )),
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, pow_operation)
        }
        Expr::Log { span, lhs, rhs } => {
            debug!("Dyadic Log");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            apply_dyadic_operation(
                span,
                &lhs_eval,
                &rhs_eval,
                |base: &Scalar, value: &Scalar| {
                    value.log(base).ok_or_eyre(format!(
                    "Somehow failed to compute the logarithm of {base:?} and {value:?}: {span:?}"
                ))
                },
            )
        }
        Expr::Min { span, lhs, rhs } => {
            debug!("Dyadic Min");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let min_operation = |a: &Scalar, b: &Scalar| {
                let result = match a.cmp(b) {
                    std::cmp::Ordering::Greater => b.clone(),
                    std::cmp::Ordering::Equal => b.clone(),
                    std::cmp::Ordering::Less => a.clone(),
                };
                Ok(result)
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, min_operation)
        }
        Expr::Max { span, lhs, rhs } => {
            debug!("Dyadic Max");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let max_operation = |a: &Scalar, b: &Scalar| {
                let result = match a.cmp(b) {
                    std::cmp::Ordering::Greater => a.clone(),
                    std::cmp::Ordering::Equal => a.clone(),
                    std::cmp::Ordering::Less => b.clone(),
                };
                Ok(result)
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, max_operation)
        }
        Expr::Binomial { span, lhs, rhs } => {
            debug!("Dyadic Binomial");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

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
                let k = f64::from(a.clone());
                let n = f64::from(b.clone());
                Ok(Scalar::Float(binomial(n, k)))
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, binomial_operation)
        }
        Expr::Deal { span, lhs, rhs } => {
            debug!("Dyadic Deal");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            if !lhs_eval.is_scalar() || !rhs_eval.is_scalar() {
                return Err((span, "Deal operation is only available for two scalars"));
            }

            let (lhs, rhs) = match (&lhs_eval.data[0], &rhs_eval.data[0]) {
                (Scalar::Integer(lhs), Scalar::Integer(rhs)) => (*lhs, *rhs),
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
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let residue_operation = |a: &Scalar, b: &Scalar| match (&a, &b) {
                (Scalar::Integer(a), Scalar::Integer(b)) => Ok(Scalar::Integer(b % a)),
                (Scalar::Float(a), Scalar::Integer(b)) => Ok(Scalar::Float(*b as f64 % a)),
                (Scalar::Integer(a), Scalar::Float(b)) => Ok(Scalar::Float(b % *a as f64)),
                (Scalar::Float(a), Scalar::Float(b)) => Ok(Scalar::Float(b % a)),
                _ => eyre::bail!("Residue not defined for character arguments"),
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, residue_operation)
        }
        Expr::IndexOf { span, lhs, rhs } => {
            debug!("Dyadic Index Of");
            // A ⍳ B — for each element of B, find its 1-based position in A.
            // If not found, returns 1 + length of A.
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let _ = span;
            let not_found = lhs_eval.data.len() as i64 + 1;

            let data = rhs_eval
                .data
                .iter()
                .map(|needle| {
                    let pos = lhs_eval
                        .data
                        .iter()
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
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let _ = span;

            let data = rhs_eval
                .data
                .iter()
                .map(|val| {
                    let count = lhs_eval.data.iter().filter(|&a| a <= val).count();
                    Scalar::Integer(count as i64)
                })
                .collect();

            Ok(Val::new(rhs_eval.shape, data))
        }
        Expr::Reshape { span, lhs, rhs } => {
            debug!("Dyadic Reshape");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let new_shape: Vec<usize> = lhs_eval
                .data
                .iter()
                .map(|s| {
                    usize::try_from(s.clone())
                        .map_err(|_| (span, "Reshape dimensions must be non-negative integers"))
                })
                .collect::<Result<Vec<usize>, _>>()?;

            let total: usize = new_shape.iter().product::<usize>().max(1);
            let data: Vec<Scalar> = rhs_eval.data.iter().cloned().cycle().take(total).collect();

            Ok(Val::new(new_shape, data))
        }
        Expr::Catenate { span, lhs, rhs } => {
            debug!("Dyadic Catenate");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let _ = span;

            let mut data = lhs_eval.data;
            data.extend(rhs_eval.data);
            Ok(Val::vector(data))
        }
        Expr::Rotate { span, lhs, rhs } => {
            debug!("Dyadic Rotate");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            if !lhs_eval.is_scalar() {
                return Err((span, "Rotate left argument must be a scalar integer"));
            }
            let n = match &lhs_eval.data[0] {
                Scalar::Integer(i) => *i,
                _ => return Err((span, "Rotate left argument must be an integer")),
            };

            let len = rhs_eval.data.len();
            if len == 0 {
                return Ok(rhs_eval);
            }
            let rot = ((n % len as i64) + len as i64) as usize % len;
            let mut data = rhs_eval.data;
            data.rotate_left(rot);
            Ok(Val::new(rhs_eval.shape, data))
        }
        Expr::Equal { span, lhs, rhs } => {
            debug!("Dyadic Equal");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |a, b| {
                Ok(Scalar::Integer(if a == b { 1 } else { 0 }))
            })
        }
        Expr::NotEqual { span, lhs, rhs } => {
            debug!("Dyadic Not Equal");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |a, b| {
                Ok(Scalar::Integer(if a != b { 1 } else { 0 }))
            })
        }
        Expr::LessThan { span, lhs, rhs } => {
            debug!("Dyadic Less Than");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |a, b| {
                Ok(Scalar::Integer(if a < b { 1 } else { 0 }))
            })
        }
        Expr::GreaterThan { span, lhs, rhs } => {
            debug!("Dyadic Greater Than");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |a, b| {
                Ok(Scalar::Integer(if a > b { 1 } else { 0 }))
            })
        }
        Expr::LessEqual { span, lhs, rhs } => {
            debug!("Dyadic Less Equal");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |a, b| {
                Ok(Scalar::Integer(if a <= b { 1 } else { 0 }))
            })
        }
        Expr::GreaterEqual { span, lhs, rhs } => {
            debug!("Dyadic Greater Equal");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |a, b| {
                Ok(Scalar::Integer(if a >= b { 1 } else { 0 }))
            })
        }
        Expr::And { span, lhs, rhs } => {
            debug!("Dyadic And");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |a, b| {
                let a = if *a != Scalar::Integer(0) { 1 } else { 0 };
                let b = if *b != Scalar::Integer(0) { 1 } else { 0 };
                Ok(Scalar::Integer(a & b))
            })
        }
        Expr::Or { span, lhs, rhs } => {
            debug!("Dyadic Or");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |a, b| {
                let a = if *a != Scalar::Integer(0) { 1 } else { 0 };
                let b = if *b != Scalar::Integer(0) { 1 } else { 0 };
                Ok(Scalar::Integer(a | b))
            })
        }
        Expr::Nand { span, lhs, rhs } => {
            debug!("Dyadic Nand");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |a, b| {
                let a = if *a != Scalar::Integer(0) { 1 } else { 0 };
                let b = if *b != Scalar::Integer(0) { 1 } else { 0 };
                Ok(Scalar::Integer(if a & b == 1 { 0 } else { 1 }))
            })
        }
        Expr::Nor { span, lhs, rhs } => {
            debug!("Dyadic Nor");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |a, b| {
                let a = if *a != Scalar::Integer(0) { 1 } else { 0 };
                let b = if *b != Scalar::Integer(0) { 1 } else { 0 };
                Ok(Scalar::Integer(if a | b == 1 { 0 } else { 1 }))
            })
        }
        Expr::Replicate { span, lhs, rhs } => {
            debug!("Dyadic Replicate");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            if lhs_eval.is_scalar() {
                // Scalar left: repeat each element n times
                let n = match lhs_eval.data[0] {
                    Scalar::Integer(i) if i >= 0 => i as usize,
                    _ => return Err((span, "Replicate count must be a non-negative integer")),
                };
                let data: Vec<Scalar> = rhs_eval
                    .data
                    .iter()
                    .flat_map(|v| std::iter::repeat_n(v.clone(), n))
                    .collect();
                Ok(Val::vector(data))
            } else {
                if lhs_eval.data.len() != rhs_eval.data.len() {
                    return Err((
                        span,
                        "Replicate: left and right arguments must have same length",
                    ));
                }
                let data: Vec<Scalar> = lhs_eval
                    .data
                    .iter()
                    .zip(rhs_eval.data.iter())
                    .flat_map(|(count, val)| {
                        let n = match count {
                            Scalar::Integer(i) => *i as usize,
                            Scalar::Float(f) => *f as usize,
                            _ => 0,
                        };
                        std::iter::repeat_n(val.clone(), n)
                    })
                    .collect();
                Ok(Val::vector(data))
            }
        }
        Expr::Expand { span, lhs, rhs } => {
            debug!("Dyadic Expand");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let mut data = Vec::new();
            let mut rhs_iter = rhs_eval.data.iter();
            for mask in &lhs_eval.data {
                let n = match mask {
                    Scalar::Integer(i) => *i,
                    Scalar::Float(f) => *f as i64,
                    _ => 0,
                };
                if n > 0 {
                    match rhs_iter.next() {
                        Some(v) => {
                            for _ in 0..n {
                                data.push(v.clone());
                            }
                        }
                        None => return Err((span, "Expand: not enough data elements")),
                    }
                } else {
                    data.push(Scalar::Integer(0));
                }
            }
            Ok(Val::vector(data))
        }
        Expr::Circular { span, lhs, rhs } => {
            debug!("Dyadic Circular");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let circular_op = |func: &Scalar, val: &Scalar| {
                let x = f64::from(val.clone());
                let result = match func {
                    Scalar::Integer(0) => (1.0 - x * x).sqrt(),
                    Scalar::Integer(1) => x.sin(),
                    Scalar::Integer(2) => x.cos(),
                    Scalar::Integer(3) => x.tan(),
                    Scalar::Integer(-1) => x.asin(),
                    Scalar::Integer(-2) => x.acos(),
                    Scalar::Integer(-3) => x.atan(),
                    Scalar::Integer(4) => (1.0 + x * x).sqrt(),
                    Scalar::Integer(5) => x.sinh(),
                    Scalar::Integer(6) => x.cosh(),
                    Scalar::Integer(7) => x.tanh(),
                    Scalar::Integer(-5) => x.asinh(),
                    Scalar::Integer(-6) => x.acosh(),
                    Scalar::Integer(-7) => x.atanh(),
                    _ => return Err(eyre::eyre!("Unknown circular function")),
                };
                Ok(Scalar::Float(result))
            };

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, circular_op)
        }
        Expr::Take { span, lhs, rhs } => {
            debug!("Dyadic Take");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            if !lhs_eval.is_scalar() {
                return Err((span, "Take left argument must be a scalar integer"));
            }
            let n = match lhs_eval.data[0] {
                Scalar::Integer(i) => i,
                _ => return Err((span, "Take left argument must be an integer")),
            };

            let len = rhs_eval.data.len();
            let abs_n = n.unsigned_abs() as usize;
            let mut data = if n >= 0 {
                let mut d: Vec<Scalar> = rhs_eval.data.iter().take(abs_n).cloned().collect();
                while d.len() < abs_n {
                    d.push(Scalar::Integer(0));
                }
                d
            } else {
                let skip = len.saturating_sub(abs_n);
                let mut d: Vec<Scalar> = rhs_eval.data.iter().skip(skip).cloned().collect();
                while d.len() < abs_n {
                    d.insert(0, Scalar::Integer(0));
                }
                d
            };
            let _ = &mut data;
            Ok(Val::vector(data))
        }
        Expr::Drop { span, lhs, rhs } => {
            debug!("Dyadic Drop");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            if !lhs_eval.is_scalar() {
                return Err((span, "Drop left argument must be a scalar integer"));
            }
            let n = match lhs_eval.data[0] {
                Scalar::Integer(i) => i,
                _ => return Err((span, "Drop left argument must be an integer")),
            };

            let len = rhs_eval.data.len();
            let data: Vec<Scalar> = if n >= 0 {
                let skip = (n as usize).min(len);
                rhs_eval.data.into_iter().skip(skip).collect()
            } else {
                let take = len.saturating_sub(n.unsigned_abs() as usize);
                rhs_eval.data.into_iter().take(take).collect()
            };
            Ok(Val::vector(data))
        }
        Expr::Assign { name, rhs, .. } => {
            debug!("Assignment");
            let val = eval(lexer, *rhs, env)?;
            env.vars.insert(name, val.clone());
            Ok(val)
        }
        Expr::ModifiedAssign {
            span,
            name,
            operator,
            rhs,
        } => {
            debug!("Modified Assign: {name}");
            let current = env
                .vars
                .get(&name)
                .cloned()
                .ok_or((span, "Undefined variable for modified assignment"))?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let op_fn = get_operator_fn(operator);
            let result = apply_dyadic_operation(span, &current, &rhs_eval, |a, b| {
                op_fn(a, b).ok_or_eyre("Modified assignment operation failed")
            })?;
            env.vars.insert(name, result.clone());
            Ok(result)
        }
        Expr::IndexedAssign {
            span,
            name,
            indices,
            rhs,
        } => {
            debug!("Indexed Assign: {name}");
            let mut current = env
                .vars
                .get(&name)
                .cloned()
                .ok_or((span, "Undefined variable for indexed assignment"))?;
            let idx_val = eval(lexer, *indices, env)?;
            let rhs_val = eval(lexer, *rhs, env)?;

            let idxs: Vec<usize> = idx_val
                .data
                .iter()
                .map(|s| {
                    let i: usize = s
                        .clone()
                        .try_into()
                        .map_err(|_| (span, "Index must be integer"))?;
                    if i < 1 || i > current.data.len() {
                        return Err((span, "Index out of bounds"));
                    }
                    Ok(i - 1) // 1-based to 0-based
                })
                .collect::<Result<Vec<_>, _>>()?;

            if rhs_val.is_scalar() {
                // Scalar: set all indexed positions to same value
                for &idx in &idxs {
                    current.data[idx] = rhs_val.data[0].clone();
                }
            } else {
                // Vector: must match length
                if rhs_val.data.len() != idxs.len() {
                    return Err((span, "Indexed assign: value length must match index count"));
                }
                for (i, &idx) in idxs.iter().enumerate() {
                    current.data[idx] = rhs_val.data[i].clone();
                }
            }

            env.vars.insert(name, current.clone());
            Ok(current)
        }
        Expr::OuterProduct {
            span,
            lhs,
            operator,
            rhs,
        } => {
            debug!("Outer Product");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let op_fn = get_operator_fn(operator);

            let rows = lhs_eval.data.len();
            let cols = rhs_eval.data.len();
            let mut data = Vec::with_capacity(rows * cols);
            for l in &lhs_eval.data {
                for r in &rhs_eval.data {
                    match op_fn(l, r) {
                        Some(v) => data.push(v),
                        None => return Err((span, "Outer product operation failed")),
                    }
                }
            }
            Ok(Val::new(vec![rows, cols], data))
        }
        Expr::Conjugate { span, arg } => {
            debug!("Monadic Conjugate");
            let _ = span;
            let arg_eval = eval(lexer, *arg, env)?;
            Ok(arg_eval)
        }
        Expr::Negate { span, arg } => {
            debug!("Monadic Negate");
            let arg_eval = eval(lexer, *arg, env)?;

            apply_monadic_operation(span, &arg_eval, |n| {
                n.checked_neg()
                    .ok_or_eyre(format!("Negation overflow or invalid operation for {n:?}"))
            })
        }
        Expr::Direction { span, arg } => {
            debug!("Monadic Direction");
            let arg_eval = eval(lexer, *arg, env)?;

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
            let arg_eval = eval(lexer, *arg, env)?;

            let reciprocal_operation = |a: &Scalar| {
                Scalar::Integer(1)
                    .checked_div(a)
                    .ok_or_eyre(format!("Invalid operation for {a:?}"))
            };

            apply_monadic_operation(span, &arg_eval, reciprocal_operation)
        }
        Expr::Exp { span, arg } => {
            debug!("Monadic Exponential");
            let arg_eval = eval(lexer, *arg, env)?;

            let exp_operation = |a: &Scalar| match a {
                Scalar::Integer(val) => Ok(Scalar::Float((*val as f64).exp())),
                Scalar::Float(val) => Ok(Scalar::Float(val.exp())),
                _ => eyre::bail!("Not defined for non-numeric types"),
            };

            apply_monadic_operation(span, &arg_eval, exp_operation)
        }
        Expr::NaturalLog { span, arg } => {
            debug!("Monadic Natural Log");
            let arg_eval = eval(lexer, *arg, env)?;

            let nat_log_operation = |value: &Scalar| match value {
                Scalar::Integer(val) if *val > 0 => Ok(Scalar::Float((*val as f64).ln())),
                Scalar::Float(val) if *val > 0.0 => Ok(Scalar::Float(val.ln())),
                _ => eyre::bail!("logarithm undefined for non-positive values"),
            };

            apply_monadic_operation(span, &arg_eval, nat_log_operation)
        }
        Expr::PiMultiple { span, arg } => {
            debug!("Monadic Pi Multiple");
            let arg_eval = eval(lexer, *arg, env)?;

            let pi_multiple_operation = |a: &Scalar| match a {
                Scalar::Integer(i) => Ok(Scalar::Float(*i as f64 * std::f64::consts::PI)),
                Scalar::Float(f) => Ok(Scalar::Float(*f * std::f64::consts::PI)),
                _ => eyre::bail!("Not defined for non-numeric types"),
            };

            apply_monadic_operation(span, &arg_eval, pi_multiple_operation)
        }
        Expr::Factorial { span, arg } => {
            debug!("Monadic Factorial");
            let arg_eval = eval(lexer, *arg, env)?;

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
            let arg_eval = eval(lexer, *arg, env)?;

            let roll_operation = |limit: &Scalar| {
                let mut rng = rand::thread_rng();

                match limit {
                    Scalar::Integer(val) if *val == 0 => Ok(Scalar::Integer(rng.r#gen())),
                    Scalar::Integer(val) => Ok(Scalar::Integer(rng.gen_range(0..=*val))),
                    _ => {
                        eyre::bail!("Roll right argument must consist of non-negative integer(s)")
                    }
                }
            };

            apply_monadic_operation(span, &arg_eval, roll_operation)
        }
        Expr::Magnitude { span, arg } => {
            debug!("Monadic Magnitude");
            let arg_eval = eval(lexer, *arg, env)?;

            let magnitude_operation = |value: &Scalar| match value {
                Scalar::Integer(val) => Ok(Scalar::Integer(val.abs())),
                Scalar::Float(val) => Ok(Scalar::Float(val.abs())),
                _ => eyre::bail!("Not defined for non-numeric types"),
            };

            apply_monadic_operation(span, &arg_eval, magnitude_operation)
        }
        Expr::Ceil { span, arg } => {
            debug!("Monadic Ceiling");
            let arg_eval = eval(lexer, *arg, env)?;

            let ceil_operation = |a: &Scalar| match a {
                Scalar::Integer(i) => Ok(Scalar::Integer(*i)),
                Scalar::Float(f) => Ok(Scalar::Float(f.ceil())),
                _ => eyre::bail!("Not defined for non-numeric types"),
            };

            apply_monadic_operation(span, &arg_eval, ceil_operation)
        }
        Expr::Floor { span, arg } => {
            debug!("Monadic Floor");
            let arg_eval = eval(lexer, *arg, env)?;

            let floor_operation = |a: &Scalar| match a {
                Scalar::Integer(i) => Ok(Scalar::Integer(*i)),
                Scalar::Float(f) => Ok(Scalar::Float(f.floor())),
                _ => eyre::bail!("Not defined for non-numeric types"),
            };

            apply_monadic_operation(span, &arg_eval, floor_operation)
        }
        Expr::MonadicMax { span, arg } => {
            debug!("Monadic Maximum");
            let arg_eval = eval(lexer, *arg, env)?;

            arg_eval
                .data
                .iter()
                .max()
                .ok_or((span, "Cannot find max"))
                .map(|num| Val::scalar(num.clone()))
        }
        Expr::MonadicMin { span, arg } => {
            debug!("Monadic Minimum");
            let arg_eval = eval(lexer, *arg, env)?;

            arg_eval
                .data
                .iter()
                .min()
                .ok_or((span, "Cannot find min"))
                .map(|num| Val::scalar(num.clone()))
        }
        Expr::GenIndex { span, arg } => {
            debug!("Monadic Iota: generate index");
            let arg_eval = eval(lexer, *arg, env)?;

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
            let arg_eval = eval(lexer, *arg, env)?;

            let data: Vec<Scalar> = arg_eval
                .data
                .iter()
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
        Expr::Shape { arg, .. } => {
            debug!("Monadic Shape");
            let arg_eval = eval(lexer, *arg, env)?;
            let data: Vec<Scalar> = arg_eval
                .shape
                .iter()
                .map(|&s| Scalar::Integer(s as i64))
                .collect();
            Ok(Val::vector(data))
        }
        Expr::Ravel { arg, .. } => {
            debug!("Monadic Ravel");
            let arg_eval = eval(lexer, *arg, env)?;
            Ok(Val::vector(arg_eval.data))
        }
        Expr::Reverse { arg, .. } => {
            debug!("Monadic Reverse");
            let arg_eval = eval(lexer, *arg, env)?;
            let mut data = arg_eval.data;
            data.reverse();
            Ok(Val::new(arg_eval.shape, data))
        }
        Expr::Transpose { arg, .. } => {
            debug!("Monadic Transpose");
            let arg_eval = eval(lexer, *arg, env)?;
            match arg_eval.shape.len() {
                0 | 1 => Ok(arg_eval),
                2 => {
                    let rows = arg_eval.shape[0];
                    let cols = arg_eval.shape[1];
                    let mut data = vec![Scalar::Integer(0); rows * cols];
                    for r in 0..rows {
                        for c in 0..cols {
                            data[c * rows + r] = arg_eval.data[r * cols + c].clone();
                        }
                    }
                    Ok(Val::new(vec![cols, rows], data))
                }
                _ => Err((Span::new(0, 0), "Transpose only supports rank 0, 1, or 2")),
            }
        }
        Expr::DyadicTranspose { span, lhs, rhs } => {
            debug!("Dyadic Transpose");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            // Parse permutation vector (1-based to 0-based)
            let perm: Vec<usize> = lhs_eval
                .data
                .iter()
                .map(|s| {
                    let v: usize = s
                        .clone()
                        .try_into()
                        .map_err(|_| (span, "Transpose perm must be integers"))?;
                    if v < 1 || v > rhs_eval.shape.len() {
                        return Err((span, "Transpose permutation out of range"));
                    }
                    Ok(v - 1)
                })
                .collect::<Result<Vec<_>, _>>()?;

            if perm.len() != rhs_eval.shape.len() {
                return Err((span, "Transpose permutation length must match array rank"));
            }

            let old_shape = &rhs_eval.shape;
            let rank = old_shape.len();

            // New shape: new_shape[perm[i]] = old_shape[i]
            let mut new_shape = vec![0usize; rank];
            for i in 0..rank {
                new_shape[perm[i]] = old_shape[i];
            }

            let total: usize = new_shape.iter().product();
            let mut new_data = vec![Scalar::Integer(0); total];

            // Compute strides for old shape
            let mut old_strides = vec![1usize; rank];
            for i in (0..rank - 1).rev() {
                old_strides[i] = old_strides[i + 1] * old_shape[i + 1];
            }

            // Compute strides for new shape
            let mut new_strides = vec![1usize; rank];
            for i in (0..rank - 1).rev() {
                new_strides[i] = new_strides[i + 1] * new_shape[i + 1];
            }

            // For each element in old array, compute its new position
            for old_flat in 0..total {
                // Convert flat index to multi-dimensional old index
                let mut old_idx = vec![0usize; rank];
                let mut remaining = old_flat;
                for i in 0..rank {
                    old_idx[i] = remaining / old_strides[i];
                    remaining %= old_strides[i];
                }

                // Apply permutation: new_idx[perm[i]] = old_idx[i]
                let mut new_idx = vec![0usize; rank];
                for i in 0..rank {
                    new_idx[perm[i]] = old_idx[i];
                }

                // Convert new multi-dimensional index to flat
                let new_flat: usize = new_idx
                    .iter()
                    .zip(new_strides.iter())
                    .map(|(&i, &s)| i * s)
                    .sum();

                new_data[new_flat] = rhs_eval.data[old_flat].clone();
            }

            Ok(Val::new(new_shape, new_data))
        }
        Expr::GradeUp { span, arg } => {
            debug!("Monadic Grade Up");
            let arg_eval = eval(lexer, *arg, env)?;
            let _ = span;
            let mut indices: Vec<usize> = (0..arg_eval.data.len()).collect();
            indices.sort_by(|&a, &b| arg_eval.data[a].cmp(&arg_eval.data[b]));
            let data: Vec<Scalar> = indices
                .iter()
                .map(|&i| Scalar::Integer(i as i64 + 1))
                .collect();
            Ok(Val::vector(data))
        }
        Expr::GradeDown { span, arg } => {
            debug!("Monadic Grade Down");
            let arg_eval = eval(lexer, *arg, env)?;
            let _ = span;
            let mut indices: Vec<usize> = (0..arg_eval.data.len()).collect();
            indices.sort_by(|&a, &b| arg_eval.data[b].cmp(&arg_eval.data[a]));
            let data: Vec<Scalar> = indices
                .iter()
                .map(|&i| Scalar::Integer(i as i64 + 1))
                .collect();
            Ok(Val::vector(data))
        }
        Expr::Reduce {
            span,
            operator,
            term,
        } => {
            debug!("Reduce");
            let term_eval = eval(lexer, *term, env)?;

            // APL reduce is a right-fold along the last axis:
            // f/ a b c d = a f (b f (c f d))
            let op_fn = get_operator_fn(operator);

            if term_eval.shape.len() <= 1 {
                // Vector or scalar: reduce all elements
                let result = term_eval
                    .data
                    .iter()
                    .rev()
                    .cloned()
                    .try_fold(None, |acc, n| match acc {
                        None => Some(Some(n)),
                        Some(right) => op_fn(&n, &right).map(Some),
                    })
                    .flatten();
                result
                    .map(Val::scalar)
                    .ok_or((span, "Arithmetic error or invalid operation in Reduce"))
            } else {
                // Higher-rank: reduce along last axis
                let last_dim = *term_eval.shape.last().unwrap();
                let row_count: usize = term_eval.data.len() / last_dim;
                let mut results = Vec::with_capacity(row_count);
                for i in 0..row_count {
                    let start = i * last_dim;
                    let row = &term_eval.data[start..start + last_dim];
                    let result = row
                        .iter()
                        .rev()
                        .cloned()
                        .try_fold(None::<Scalar>, |acc, n| match acc {
                            None => Some(Some(n)),
                            Some(right) => op_fn(&n, &right).map(Some),
                        })
                        .flatten()
                        .ok_or((span, "Arithmetic error in Reduce"))?;
                    results.push(result);
                }
                let new_shape = term_eval.shape[..term_eval.shape.len() - 1].to_vec();
                if new_shape.is_empty() {
                    Ok(Val::scalar(results.into_iter().next().unwrap()))
                } else {
                    Ok(Val::new(new_shape, results))
                }
            }
        }
        Expr::Scan {
            span,
            operator,
            term,
        } => {
            debug!("Scan");
            let term_eval = eval(lexer, *term, env)?;

            let op_fn = get_operator_fn(operator);

            // Scan: each element i is the right-fold reduce of prefix 0..=i
            let mut data = Vec::with_capacity(term_eval.data.len());
            for i in 0..term_eval.data.len() {
                let prefix = &term_eval.data[..=i];
                let result = prefix
                    .iter()
                    .rev()
                    .cloned()
                    .try_fold(None::<Scalar>, |acc, n| match acc {
                        None => Some(Some(n)),
                        Some(right) => op_fn(&n, &right).map(Some),
                    })
                    .flatten()
                    .ok_or((span, "Arithmetic error in Scan"))?;
                data.push(result);
            }
            Ok(Val::vector(data))
        }
        Expr::ReduceFirst {
            span,
            operator,
            term,
        } => {
            debug!("Reduce First Axis");
            let term_eval = eval(lexer, *term, env)?;
            let op_fn = get_operator_fn(operator);

            if term_eval.shape.len() <= 1 {
                // Vector: same as regular reduce
                let result = term_eval
                    .data
                    .iter()
                    .rev()
                    .cloned()
                    .try_fold(None, |acc, n| match acc {
                        None => Some(Some(n)),
                        Some(right) => op_fn(&n, &right).map(Some),
                    })
                    .flatten();
                result
                    .map(Val::scalar)
                    .ok_or((span, "Arithmetic error in ReduceFirst"))
            } else {
                // Higher-rank: reduce along FIRST axis (columns)
                let first_dim = term_eval.shape[0];
                let stride: usize = term_eval.data.len() / first_dim;
                let mut results = Vec::with_capacity(stride);
                for col in 0..stride {
                    let column: Vec<Scalar> = (0..first_dim)
                        .map(|row| term_eval.data[row * stride + col].clone())
                        .collect();
                    let result = column
                        .iter()
                        .rev()
                        .cloned()
                        .try_fold(None::<Scalar>, |acc, n| match acc {
                            None => Some(Some(n)),
                            Some(right) => op_fn(&n, &right).map(Some),
                        })
                        .flatten()
                        .ok_or((span, "Arithmetic error in ReduceFirst"))?;
                    results.push(result);
                }
                let new_shape = term_eval.shape[1..].to_vec();
                if new_shape.is_empty() {
                    Ok(Val::scalar(results.into_iter().next().unwrap()))
                } else {
                    Ok(Val::new(new_shape, results))
                }
            }
        }
        Expr::ScanFirst {
            span,
            operator,
            term,
        } => {
            debug!("Scan First Axis");
            let term_eval = eval(lexer, *term, env)?;
            let op_fn = get_operator_fn(operator);

            if term_eval.shape.len() <= 1 {
                // Vector: same as regular scan
                let mut data = Vec::with_capacity(term_eval.data.len());
                for i in 0..term_eval.data.len() {
                    let prefix = &term_eval.data[..=i];
                    let result = prefix
                        .iter()
                        .rev()
                        .cloned()
                        .try_fold(None::<Scalar>, |acc, n| match acc {
                            None => Some(Some(n)),
                            Some(right) => op_fn(&n, &right).map(Some),
                        })
                        .flatten()
                        .ok_or((span, "Arithmetic error in ScanFirst"))?;
                    data.push(result);
                }
                Ok(Val::vector(data))
            } else {
                // Higher-rank: scan along FIRST axis (columns)
                let first_dim = term_eval.shape[0];
                let stride: usize = term_eval.data.len() / first_dim;
                let mut data = term_eval.data.clone();
                for col in 0..stride {
                    for row in 1..first_dim {
                        let prev = data[(row - 1) * stride + col].clone();
                        let curr = data[row * stride + col].clone();
                        data[row * stride + col] =
                            op_fn(&prev, &curr).ok_or((span, "Arithmetic error in ScanFirst"))?;
                    }
                }
                Ok(Val::new(term_eval.shape.clone(), data))
            }
        }
        Expr::Membership { span: _, lhs, rhs } => {
            debug!("Dyadic Membership");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let data = lhs_eval
                .data
                .iter()
                .map(|l| {
                    let found = rhs_eval.data.iter().any(|r| l == r);
                    Scalar::Integer(if found { 1 } else { 0 })
                })
                .collect();
            Ok(Val::new(lhs_eval.shape.clone(), data))
        }
        Expr::IndexRead {
            span,
            array,
            indices,
        } => {
            debug!("Index Read");
            let arr = eval(lexer, *array, env)?;
            let idx_val = eval(lexer, *indices, env)?;
            let indices: Vec<usize> = idx_val
                .data
                .iter()
                .map(|s| {
                    let i: usize = s
                        .clone()
                        .try_into()
                        .map_err(|_| (span, "Index must be integer"))?;
                    if i < 1 || i > arr.data.len() {
                        return Err((span, "Index out of bounds"));
                    }
                    Ok(i - 1)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let data: Vec<Scalar> = indices.iter().map(|&i| arr.data[i].clone()).collect();
            if data.len() == 1 {
                Ok(Val::scalar(data.into_iter().next().unwrap()))
            } else {
                Ok(Val::vector(data))
            }
        }
        Expr::DfnReduce { span, body, term } => {
            debug!("Dfn Reduce");
            let term_eval = eval(lexer, *term, env)?;
            if term_eval.data.len() < 2 {
                return Ok(term_eval);
            }
            let body_rc = Rc::new(*body);
            // Right fold: f/ a b c = a f (b f c)
            let mut acc = Val::scalar(term_eval.data.last().unwrap().clone());
            for i in (0..term_eval.data.len() - 1).rev() {
                let left = match &term_eval.data[i] {
                    Scalar::Nested(v) => (**v).clone(),
                    s => Val::scalar(s.clone()),
                };
                let stored = StoredDfn {
                    body: Rc::clone(&body_rc),
                    source: lexer.span_str(span).to_string(),
                };
                let mut dfn_env = env.clone();
                dfn_env.vars.insert("⍺".to_string(), left);
                dfn_env.vars.insert("⍵".to_string(), acc);
                dfn_env.fns.insert("∇".to_string(), stored);
                acc = eval(lexer, (*body_rc).clone(), &mut dfn_env)?;
            }
            Ok(acc)
        }
        Expr::DfnReduceFirst { span, body, term } => {
            debug!("Dfn Reduce First");
            let term_eval = eval(lexer, *term, env)?;
            // For vectors, same as DfnReduce
            // For matrices, reduce along first axis (column-wise)
            if term_eval.shape.len() <= 1 {
                let body_rc = Rc::new(*body);
                if term_eval.data.len() < 2 {
                    return Ok(term_eval);
                }
                let mut acc = Val::scalar(term_eval.data.last().unwrap().clone());
                for i in (0..term_eval.data.len() - 1).rev() {
                    let left = match &term_eval.data[i] {
                        Scalar::Nested(v) => (**v).clone(),
                        s => Val::scalar(s.clone()),
                    };
                    let stored = StoredDfn {
                        body: Rc::clone(&body_rc),
                        source: lexer.span_str(span).to_string(),
                    };
                    let mut dfn_env = env.clone();
                    dfn_env.vars.insert("⍺".to_string(), left);
                    dfn_env.vars.insert("⍵".to_string(), acc);
                    dfn_env.fns.insert("∇".to_string(), stored);
                    acc = eval(lexer, (*body_rc).clone(), &mut dfn_env)?;
                }
                Ok(acc)
            } else {
                // Higher-rank: reduce along first axis
                let body_rc = Rc::new(*body);
                let first_dim = term_eval.shape[0];
                let stride: usize = term_eval.data.len() / first_dim;
                let cell_shape = term_eval.shape[1..].to_vec();
                // Start with last row
                let mut acc_data = term_eval.data[(first_dim - 1) * stride..].to_vec();
                for row in (0..first_dim - 1).rev() {
                    let row_data = &term_eval.data[row * stride..(row + 1) * stride];
                    let left = Val::new(cell_shape.clone(), row_data.to_vec());
                    let right = Val::new(cell_shape.clone(), acc_data);
                    let stored = StoredDfn {
                        body: Rc::clone(&body_rc),
                        source: lexer.span_str(span).to_string(),
                    };
                    let mut dfn_env = env.clone();
                    dfn_env.vars.insert("⍺".to_string(), left);
                    dfn_env.vars.insert("⍵".to_string(), right);
                    dfn_env.fns.insert("∇".to_string(), stored);
                    let result = eval(lexer, (*body_rc).clone(), &mut dfn_env)?;
                    acc_data = result.data;
                }
                Ok(Val::new(cell_shape, acc_data))
            }
        }
        Expr::StringArray { span: _, elements } => {
            debug!("String Array");
            let data: Vec<Scalar> = elements
                .into_iter()
                .map(|e| {
                    let val = eval(lexer, e, env)?;
                    Ok(Scalar::Nested(Box::new(val)))
                })
                .collect::<Result<Vec<_>, (Span, &'static str)>>()?;
            Ok(Val::vector(data))
        }
        Expr::Variable { span, name } => {
            debug!("Variable: {name}");
            env.vars
                .get(&name)
                .cloned()
                .ok_or((span, "Undefined variable"))
        }
        Expr::Omega { span } => env
            .vars
            .get("⍵")
            .cloned()
            .ok_or((span, "⍵ used outside of a dfn")),
        Expr::Alpha { span } => env
            .vars
            .get("⍺")
            .cloned()
            .ok_or((span, "⍺ used outside of a dfn")),
        Expr::MonadicDfn { span, body, rhs } => {
            debug!("Monadic Dfn");
            let rhs_val = eval(lexer, *rhs, env)?;
            let body_rc = Rc::new(*body);
            let stored = StoredDfn {
                body: Rc::clone(&body_rc),
                source: lexer.span_str(span).to_string(),
            };
            let mut dfn_env = env.clone();
            dfn_env.vars.insert("⍵".to_string(), rhs_val);
            dfn_env.fns.insert("∇".to_string(), stored);
            eval(lexer, (*body_rc).clone(), &mut dfn_env)
        }
        Expr::RankOp {
            span,
            body,
            rank,
            arg,
        } => {
            debug!("Rank Operator");
            let rank_val = eval(lexer, *rank, env)?;
            let k: usize = rank_val.data[0]
                .clone()
                .try_into()
                .map_err(|_| (span, "Rank must be a non-negative integer"))?;
            let arg_val = eval(lexer, *arg, env)?;
            let n = arg_val.shape.len();
            let body_rc = Rc::new(*body);
            if k >= n {
                // Apply to entire array
                let stored = StoredDfn {
                    body: Rc::clone(&body_rc),
                    source: lexer.span_str(span).to_string(),
                };
                let mut dfn_env = env.clone();
                dfn_env.vars.insert("⍵".to_string(), arg_val);
                dfn_env.fns.insert("∇".to_string(), stored);
                return eval(lexer, (*body_rc).clone(), &mut dfn_env);
            }
            let frame_shape = arg_val.shape[..n - k].to_vec();
            let cell_shape = arg_val.shape[n - k..].to_vec();
            let cell_size: usize = cell_shape.iter().product();
            let num_cells: usize = frame_shape.iter().product();
            let mut results = Vec::new();
            let mut result_cell_shape: Option<Vec<usize>> = None;
            for i in 0..num_cells {
                let start = i * cell_size;
                let cell_data = arg_val.data[start..start + cell_size].to_vec();
                let cell = Val::new(cell_shape.clone(), cell_data);
                let stored = StoredDfn {
                    body: Rc::clone(&body_rc),
                    source: lexer.span_str(span).to_string(),
                };
                let mut dfn_env = env.clone();
                dfn_env.vars.insert("⍵".to_string(), cell);
                dfn_env.fns.insert("∇".to_string(), stored);
                let result = eval(lexer, (*body_rc).clone(), &mut dfn_env)?;
                if result_cell_shape.is_none() {
                    result_cell_shape = Some(result.shape.clone());
                }
                results.extend(result.data);
            }
            let rcs = result_cell_shape.unwrap_or_default();
            let mut final_shape = frame_shape;
            final_shape.extend_from_slice(&rcs);
            Ok(Val::new(final_shape, results))
        }
        Expr::AtOp {
            span,
            body,
            indices,
            arg,
        } => {
            debug!("At Operator");
            let idx_val = eval(lexer, *indices, env)?;
            let mut arg_val = eval(lexer, *arg, env)?;
            let body_rc = Rc::new(*body);

            // Convert 1-based indices to 0-based
            let idxs: Vec<usize> = idx_val
                .data
                .iter()
                .map(|s| {
                    let i: usize = s
                        .clone()
                        .try_into()
                        .map_err(|_| (span, "At index must be integer"))?;
                    if i < 1 || i > arg_val.data.len() {
                        return Err((span, "At index out of bounds"));
                    }
                    Ok(i - 1)
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Apply function to each indexed element
            for &idx in &idxs {
                let elem = Val::scalar(arg_val.data[idx].clone());
                let stored = StoredDfn {
                    body: Rc::clone(&body_rc),
                    source: lexer.span_str(span).to_string(),
                };
                let mut dfn_env = env.clone();
                dfn_env.vars.insert("⍵".to_string(), elem);
                dfn_env.fns.insert("∇".to_string(), stored);
                let result = eval(lexer, (*body_rc).clone(), &mut dfn_env)?;
                arg_val.data[idx] = result.data[0].clone();
            }

            Ok(arg_val)
        }
        Expr::KeyOp { span, body, arg } => {
            debug!("Key Operator");
            let arg_val = eval(lexer, *arg, env)?;
            let body_rc = Rc::new(*body);

            // Find unique keys and their indices (1-based)
            let mut keys: Vec<Scalar> = Vec::new();
            let mut groups: Vec<Vec<Scalar>> = Vec::new();

            for (i, s) in arg_val.data.iter().enumerate() {
                if let Some(pos) = keys.iter().position(|k| k == s) {
                    groups[pos].push(Scalar::Integer((i + 1) as i64));
                } else {
                    keys.push(s.clone());
                    groups.push(vec![Scalar::Integer((i + 1) as i64)]);
                }
            }

            // Apply f to each group
            let mut results = Vec::new();
            for (key, indices) in keys.iter().zip(groups.iter()) {
                let stored = StoredDfn {
                    body: Rc::clone(&body_rc),
                    source: lexer.span_str(span).to_string(),
                };
                let mut dfn_env = env.clone();
                dfn_env
                    .vars
                    .insert("⍺".to_string(), Val::scalar(key.clone()));
                dfn_env
                    .vars
                    .insert("⍵".to_string(), Val::vector(indices.clone()));
                dfn_env.fns.insert("∇".to_string(), stored);
                let result = eval(lexer, (*body_rc).clone(), &mut dfn_env)?;
                results.extend(result.data);
            }

            Ok(Val::vector(results))
        }
        Expr::PowerOp {
            span,
            body,
            count,
            arg,
        } => {
            debug!("Power Operator (dfn)");
            let count_val = eval(lexer, *count, env)?;
            let n: usize = count_val.data[0]
                .clone()
                .try_into()
                .map_err(|_| (span, "Power operator count must be a non-negative integer"))?;
            let mut current = eval(lexer, *arg, env)?;
            let body_rc = Rc::new(*body);
            for _ in 0..n {
                let stored = StoredDfn {
                    body: Rc::clone(&body_rc),
                    source: lexer.span_str(span).to_string(),
                };
                let mut dfn_env = env.clone();
                dfn_env.vars.insert("⍵".to_string(), current);
                dfn_env.fns.insert("∇".to_string(), stored);
                current = eval(lexer, (*body_rc).clone(), &mut dfn_env)?;
            }
            Ok(current)
        }
        Expr::DyadicDfn {
            span,
            lhs,
            body,
            rhs,
        } => {
            debug!("Dyadic Dfn");
            let lhs_val = eval(lexer, *lhs, env)?;
            let rhs_val = eval(lexer, *rhs, env)?;
            let body_rc = Rc::new(*body);
            let stored = StoredDfn {
                body: Rc::clone(&body_rc),
                source: lexer.span_str(span).to_string(),
            };
            let mut dfn_env = env.clone();
            dfn_env.vars.insert("⍺".to_string(), lhs_val);
            dfn_env.vars.insert("⍵".to_string(), rhs_val);
            dfn_env.fns.insert("∇".to_string(), stored);
            eval(lexer, (*body_rc).clone(), &mut dfn_env)
        }
        Expr::ComposeDfn { span: _, f, g, arg } => {
            debug!("Compose (monadic)");
            let arg_val = eval(lexer, *arg, env)?;
            // First apply g monadically
            let mut g_env = env.clone();
            g_env.vars.insert("⍵".to_string(), arg_val);
            let g_result = eval(lexer, *g, &mut g_env)?;
            // Then apply f monadically
            let mut f_env = env.clone();
            f_env.vars.insert("⍵".to_string(), g_result);
            eval(lexer, *f, &mut f_env)
        }
        Expr::ComposeDyadicDfn {
            span: _,
            lhs,
            f,
            g,
            arg,
        } => {
            debug!("Compose (dyadic)");
            let lhs_val = eval(lexer, *lhs, env)?;
            let arg_val = eval(lexer, *arg, env)?;
            // Apply g monadically to right arg
            let mut g_env = env.clone();
            g_env.vars.insert("⍵".to_string(), arg_val);
            let g_result = eval(lexer, *g, &mut g_env)?;
            // Apply f dyadically with left and g's result
            let mut f_env = env.clone();
            f_env.vars.insert("⍺".to_string(), lhs_val);
            f_env.vars.insert("⍵".to_string(), g_result);
            eval(lexer, *f, &mut f_env)
        }
        Expr::OverDfn { span: _, f, g, arg } => {
            debug!("Over (monadic)");
            let arg_val = eval(lexer, *arg, env)?;
            // Apply g monadically to arg
            let mut g_env = env.clone();
            g_env.vars.insert("⍵".to_string(), arg_val);
            let g_result = eval(lexer, *g, &mut g_env)?;
            // Apply f monadically to g's result
            let mut f_env = env.clone();
            f_env.vars.insert("⍵".to_string(), g_result);
            eval(lexer, *f, &mut f_env)
        }
        Expr::OverDyadicDfn {
            span: _,
            lhs,
            f,
            g,
            arg,
        } => {
            debug!("Over (dyadic)");
            let lhs_val = eval(lexer, *lhs, env)?;
            let arg_val = eval(lexer, *arg, env)?;
            // Apply g to BOTH arguments
            let g_clone = (*g).clone();
            let mut g_env_l = env.clone();
            g_env_l.vars.insert("⍵".to_string(), lhs_val);
            let g_lhs = eval(lexer, *g, &mut g_env_l)?;
            let mut g_env_r = env.clone();
            g_env_r.vars.insert("⍵".to_string(), arg_val);
            let g_rhs = eval(lexer, g_clone, &mut g_env_r)?;
            // Apply f dyadically to the two results
            let mut f_env = env.clone();
            f_env.vars.insert("⍺".to_string(), g_lhs);
            f_env.vars.insert("⍵".to_string(), g_rhs);
            eval(lexer, *f, &mut f_env)
        }
        Expr::SelfCall { span, arg } => {
            debug!("Self-reference ∇");
            let arg_val = eval(lexer, *arg, env)?;
            let stored = env
                .fns
                .get("∇")
                .cloned()
                .ok_or((span, "∇ used outside of a dfn"))?;
            let mut self_env = env.clone();
            self_env.vars.insert("⍵".to_string(), arg_val);
            eval_stored_dfn(&stored, &mut self_env)
                .map_err(|(_span, msg)| (span, Box::leak(msg.into_boxed_str()) as &'static str))
        }
        Expr::DfnGuard {
            cond, result, rest, ..
        } => {
            debug!("Dfn Guard");
            let cond_val = eval(lexer, *cond, env)?;
            let is_true = match cond_val.data.first() {
                Some(Scalar::Integer(1)) => true,
                Some(Scalar::Float(f)) if *f == 1.0 => true,
                _ => false,
            };
            if is_true {
                eval(lexer, *result, env)
            } else {
                eval(lexer, *rest, env)
            }
        }
        Expr::DfnStatements { first, rest, .. } => {
            debug!("Dfn Statements");
            eval(lexer, *first, env)?;
            eval(lexer, *rest, env)
        }
        Expr::AssignDfn { span, name, body } => {
            debug!("Assign Dfn");
            let stored = StoredDfn {
                body: Rc::new(*body),
                source: lexer.span_str(span).to_string(),
            };
            env.fns.insert(name, stored);
            Ok(Val::scalar(Scalar::Integer(0)))
        }
        Expr::NamedMonadic { span, name, rhs } => {
            debug!("Named Monadic: {name}");
            let stored = env
                .fns
                .get(&name)
                .cloned()
                .ok_or((span, "Undefined function"))?;
            let rhs_val = eval(lexer, *rhs, env)?;
            let mut dfn_env = env.clone();
            dfn_env.vars.insert("⍵".to_string(), rhs_val);
            dfn_env.fns.insert("∇".to_string(), stored.clone());
            eval_stored_dfn(&stored, &mut dfn_env)
                .map_err(|(_span, msg)| (span, Box::leak(msg.into_boxed_str()) as &'static str))
        }
        Expr::NamedDyadic {
            span,
            lhs,
            name,
            rhs,
        } => {
            debug!("Named Dyadic: {name}");
            let stored = env
                .fns
                .get(&name)
                .cloned()
                .ok_or((span, "Undefined function"))?;
            let lhs_val = eval(lexer, *lhs, env)?;
            let rhs_val = eval(lexer, *rhs, env)?;
            let mut dfn_env = env.clone();
            dfn_env.vars.insert("⍺".to_string(), lhs_val);
            dfn_env.vars.insert("⍵".to_string(), rhs_val);
            dfn_env.fns.insert("∇".to_string(), stored.clone());
            eval_stored_dfn(&stored, &mut dfn_env)
                .map_err(|(_span, msg)| (span, Box::leak(msg.into_boxed_str()) as &'static str))
        }
        Expr::Enclose { arg, .. } => {
            debug!("Monadic Enclose");
            let arg_eval = eval(lexer, *arg, env)?;
            Ok(Val::scalar(Scalar::Nested(Box::new(arg_eval))))
        }
        Expr::First { span, arg } => {
            debug!("Monadic First / Disclose");
            let arg_eval = eval(lexer, *arg, env)?;
            let _ = span;
            match arg_eval.data.into_iter().next() {
                Some(Scalar::Nested(v)) => Ok(*v),
                Some(s) => Ok(Val::scalar(s)),
                None => Ok(Val::scalar(Scalar::Integer(0))),
            }
        }
        Expr::Partition { span, lhs, rhs } => {
            debug!("Dyadic Partition");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            if lhs_eval.data.len() != rhs_eval.data.len() {
                return Err((span, "Partition: arguments must have same length"));
            }

            let mut groups: Vec<Scalar> = Vec::new();
            let mut current: Vec<Scalar> = Vec::new();
            let mut in_group = false;

            for (mask, val) in lhs_eval.data.iter().zip(rhs_eval.data.iter()) {
                let m = match mask {
                    Scalar::Integer(i) => *i,
                    Scalar::Float(f) => *f as i64,
                    _ => 0,
                };
                if m > 0 {
                    if !in_group {
                        if !current.is_empty() {
                            groups.push(Scalar::Nested(Box::new(Val::vector(current))));
                            current = Vec::new();
                        }
                        in_group = true;
                    }
                    current.push(val.clone());
                } else {
                    if !current.is_empty() {
                        groups.push(Scalar::Nested(Box::new(Val::vector(current))));
                        current = Vec::new();
                    }
                    in_group = false;
                }
            }
            if !current.is_empty() {
                groups.push(Scalar::Nested(Box::new(Val::vector(current))));
            }
            Ok(Val::vector(groups))
        }
        Expr::PartitionedEnclose { span, lhs, rhs } => {
            debug!("Partitioned Enclose");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            if lhs_eval.data.len() != rhs_eval.data.len() {
                return Err((
                    span,
                    "Partitioned enclose: left and right must be same length",
                ));
            }

            let mut partitions: Vec<Vec<Scalar>> = Vec::new();
            let mut current: Option<Vec<Scalar>> = None;

            for (mask, elem) in lhs_eval.data.iter().zip(rhs_eval.data.iter()) {
                let m: f64 = mask.clone().into();
                if m >= 1.0 {
                    // Start new partition (save current if exists)
                    if let Some(part) = current.take() {
                        partitions.push(part);
                    }
                    current = Some(vec![elem.clone()]);
                } else if m == 0.0 && current.is_some() {
                    // Continue current partition
                    current.as_mut().unwrap().push(elem.clone());
                }
                // If m == 0 and no current partition, element is dropped
            }

            // Don't forget the last partition
            if let Some(part) = current {
                partitions.push(part);
            }

            let data: Vec<Scalar> = partitions
                .into_iter()
                .map(|p| Scalar::Nested(Box::new(Val::vector(p))))
                .collect();

            Ok(Val::vector(data))
        }
        Expr::MonadicEach { span, func, arg } => {
            debug!("Monadic Each: {func}");
            let arg_eval = eval(lexer, *arg, env)?;

            let apply_to_val = |v: &Val| -> Result<Val, (Span, &'static str)> {
                match func.as_str() {
                    "shape" => {
                        let data: Vec<Scalar> =
                            v.shape.iter().map(|&s| Scalar::Integer(s as i64)).collect();
                        Ok(Val::vector(data))
                    }
                    "reverse" => {
                        let mut data = v.data.clone();
                        data.reverse();
                        Ok(Val::new(v.shape.clone(), data))
                    }
                    "iota" => {
                        if let Some(Scalar::Integer(n)) = v.data.first() {
                            let data: Vec<Scalar> = (1..=*n).map(Scalar::Integer).collect();
                            Ok(Val::vector(data))
                        } else {
                            Err((span, "Iota each: elements must be integers"))
                        }
                    }
                    _ => Err((span, "Unknown each function")),
                }
            };

            let data: Vec<Scalar> = arg_eval
                .data
                .iter()
                .map(|elem| match elem {
                    Scalar::Nested(v) => apply_to_val(v).map(|r| Scalar::Nested(Box::new(r))),
                    Scalar::Integer(n) => apply_to_val(&Val::scalar(Scalar::Integer(*n)))
                        .map(|r| Scalar::Nested(Box::new(r))),
                    _ => Err((span, "Each: unsupported element type")),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Val::vector(data))
        }
        Expr::ReduceEach {
            span,
            operator,
            term,
        } => {
            debug!("Reduce Each");
            let term_eval = eval(lexer, *term, env)?;
            let op_fn = get_operator_fn(operator);

            let data: Vec<Scalar> = term_eval
                .data
                .iter()
                .map(|elem| {
                    let inner = match elem {
                        Scalar::Nested(v) => &v.data,
                        _ => return Err((span, "Reduce each: elements must be nested")),
                    };
                    inner
                        .iter()
                        .rev()
                        .cloned()
                        .try_fold(None::<Scalar>, |acc, n| match acc {
                            None => Some(Some(n)),
                            Some(right) => op_fn(&n, &right).map(Some),
                        })
                        .flatten()
                        .ok_or((span, "Reduce each: operation failed"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Val::vector(data))
        }
        Expr::DyadicEach {
            span,
            lhs,
            operator,
            rhs,
        } => {
            debug!("Dyadic Each");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let op_fn = get_operator_fn(operator);

            // Element-wise application
            let apply = |a: &Scalar, b: &Scalar| -> Result<Scalar, (Span, &'static str)> {
                op_fn(a, b).ok_or((span, "Dyadic each: operation failed"))
            };

            if lhs_eval.is_scalar() {
                let data: Vec<Scalar> = rhs_eval
                    .data
                    .iter()
                    .map(|r| apply(&lhs_eval.data[0], r))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Val::new(rhs_eval.shape, data))
            } else if rhs_eval.is_scalar() {
                let data: Vec<Scalar> = lhs_eval
                    .data
                    .iter()
                    .map(|l| apply(l, &rhs_eval.data[0]))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Val::new(lhs_eval.shape, data))
            } else {
                let data: Vec<Scalar> = lhs_eval
                    .data
                    .iter()
                    .zip(rhs_eval.data.iter())
                    .map(|(l, r)| apply(l, r))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Val::new(lhs_eval.shape, data))
            }
        }
        Expr::Unique { arg, .. } => {
            debug!("Monadic Unique");
            let arg_eval = eval(lexer, *arg, env)?;
            let mut seen = Vec::new();
            for v in &arg_eval.data {
                if !seen.contains(v) {
                    seen.push(v.clone());
                }
            }
            Ok(Val::vector(seen))
        }
        Expr::Not { span, arg } => {
            debug!("Monadic Not");
            let arg_eval = eval(lexer, *arg, env)?;
            apply_monadic_operation(span, &arg_eval, |a| {
                Ok(Scalar::Integer(if *a == Scalar::Integer(0) {
                    1
                } else {
                    0
                }))
            })
        }
        Expr::Union { span, lhs, rhs } => {
            debug!("Dyadic Union");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let _ = span;
            let mut data = lhs_eval.data;
            for v in &rhs_eval.data {
                if !data.contains(v) {
                    data.push(v.clone());
                }
            }
            Ok(Val::vector(data))
        }
        Expr::Intersection { span, lhs, rhs } => {
            debug!("Dyadic Intersection");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let _ = span;
            let data: Vec<Scalar> = lhs_eval
                .data
                .iter()
                .filter(|v| rhs_eval.data.contains(v))
                .cloned()
                .collect();
            Ok(Val::vector(data))
        }
        Expr::Without { span, lhs, rhs } => {
            debug!("Dyadic Without");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let _ = span;
            let data: Vec<Scalar> = lhs_eval
                .data
                .iter()
                .filter(|v| !rhs_eval.data.contains(v))
                .cloned()
                .collect();
            Ok(Val::vector(data))
        }
        Expr::Decode { span, lhs, rhs } => {
            debug!("Dyadic Decode");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            if !lhs_eval.is_scalar() {
                return Err((span, "Decode: left argument must be a scalar base"));
            }
            let base = f64::from(lhs_eval.data[0].clone());
            let result = rhs_eval
                .data
                .iter()
                .fold(0.0_f64, |acc, v| acc * base + f64::from(v.clone()));
            Ok(Val::scalar(Scalar::Integer(result as i64)))
        }
        Expr::Encode { span, lhs, rhs } => {
            debug!("Dyadic Encode");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            if !rhs_eval.is_scalar() {
                return Err((span, "Encode: right argument must be a scalar"));
            }
            let mut n = f64::from(rhs_eval.data[0].clone()) as i64;
            let bases: Vec<i64> = lhs_eval
                .data
                .iter()
                .map(|v| f64::from(v.clone()) as i64)
                .collect();
            let mut data: Vec<Scalar> = vec![Scalar::Integer(0); bases.len()];
            for i in (0..bases.len()).rev() {
                if bases[i] != 0 {
                    data[i] = Scalar::Integer(n % bases[i]);
                    n /= bases[i];
                }
            }
            Ok(Val::vector(data))
        }
        Expr::InnerProduct {
            span,
            lhs,
            f,
            g,
            rhs,
        } => {
            debug!("Inner Product");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let f_fn = get_operator_fn(f);
            let g_fn = get_operator_fn(g);

            match (lhs_eval.shape.len(), rhs_eval.shape.len()) {
                (1, 1) => {
                    // Vector inner product: +/ lhs × rhs
                    if lhs_eval.data.len() != rhs_eval.data.len() {
                        return Err((span, "Inner product: lengths must match"));
                    }
                    let products: Vec<Scalar> = lhs_eval
                        .data
                        .iter()
                        .zip(rhs_eval.data.iter())
                        .map(|(a, b)| g_fn(a, b).ok_or((span, "Inner product g failed")))
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = products
                        .iter()
                        .rev()
                        .cloned()
                        .try_fold(None::<Scalar>, |acc, n| match acc {
                            None => Some(Some(n)),
                            Some(right) => f_fn(&n, &right).map(Some),
                        })
                        .flatten()
                        .ok_or((span, "Inner product f failed"))?;
                    Ok(Val::scalar(result))
                }
                (2, 2) => {
                    // Matrix inner product
                    let m = lhs_eval.shape[0];
                    let k = lhs_eval.shape[1];
                    let n = rhs_eval.shape[1];
                    if k != rhs_eval.shape[0] {
                        return Err((span, "Inner product: inner dimensions must match"));
                    }
                    let mut data = Vec::with_capacity(m * n);
                    for i in 0..m {
                        for j in 0..n {
                            let products: Vec<Scalar> = (0..k)
                                .map(|p| {
                                    g_fn(&lhs_eval.data[i * k + p], &rhs_eval.data[p * n + j])
                                        .ok_or((span, "Inner product g failed"))
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            let result = products
                                .iter()
                                .cloned()
                                .reduce(|a, b| f_fn(&a, &b).unwrap_or(a))
                                .ok_or((span, "Inner product f failed"))?;
                            data.push(result);
                        }
                    }
                    Ok(Val::new(vec![m, n], data))
                }
                _ => Err((span, "Inner product: only rank 1 and 2 supported")),
            }
        }
        Expr::Index { span, lhs, rhs } => {
            debug!("Dyadic Index");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            let data: Vec<Scalar> = lhs_eval
                .data
                .iter()
                .map(|idx| {
                    let i = f64::from(idx.clone()) as usize;
                    if i == 0 || i > rhs_eval.data.len() {
                        Err((span, "Index out of bounds"))
                    } else {
                        Ok(rhs_eval.data[i - 1].clone())
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Val::vector(data))
        }
        Expr::MatrixInverse { span, arg } => {
            debug!("Matrix Inverse");
            let arg_eval = eval(lexer, *arg, env)?;
            if arg_eval.shape.len() != 2 {
                return Err((span, "Matrix inverse requires a rank-2 array"));
            }
            let n = arg_eval.shape[0];
            if n != arg_eval.shape[1] {
                return Err((span, "Matrix inverse requires a square matrix"));
            }
            // Gauss-Jordan elimination
            let mut m: Vec<f64> = arg_eval.data.iter().map(|s| f64::from(s.clone())).collect();
            let mut inv = vec![0.0_f64; n * n];
            for i in 0..n {
                inv[i * n + i] = 1.0;
            }

            for col in 0..n {
                let pivot_row = (col..n)
                    .max_by(|&a, &b| {
                        m[a * n + col]
                            .abs()
                            .partial_cmp(&m[b * n + col].abs())
                            .unwrap()
                    })
                    .unwrap();
                if m[pivot_row * n + col].abs() < 1e-12 {
                    return Err((span, "Matrix is singular"));
                }
                for j in 0..n {
                    m.swap(col * n + j, pivot_row * n + j);
                    inv.swap(col * n + j, pivot_row * n + j);
                }
                let pivot = m[col * n + col];
                for j in 0..n {
                    m[col * n + j] /= pivot;
                    inv[col * n + j] /= pivot;
                }
                for i in 0..n {
                    if i != col {
                        let factor = m[i * n + col];
                        for j in 0..n {
                            m[i * n + j] -= factor * m[col * n + j];
                            inv[i * n + j] -= factor * inv[col * n + j];
                        }
                    }
                }
            }
            let data: Vec<Scalar> = inv.into_iter().map(Scalar::Float).collect();
            Ok(Val::new(vec![n, n], data))
        }
        Expr::MatrixDivide { span, lhs, rhs } => {
            debug!("Matrix Divide");
            // B ⌹ A means solve Ax = B, i.e. x = A⁻¹ B
            let b_eval = eval(lexer, *lhs, env)?;
            let a_eval = eval(lexer, *rhs, env)?;

            if a_eval.shape.len() != 2 {
                return Err((span, "Matrix divide: right argument must be a matrix"));
            }
            let n = a_eval.shape[0];
            if n != a_eval.shape[1] {
                return Err((span, "Matrix divide: right argument must be square"));
            }

            // Build augmented matrix [A | B]
            let b_cols = if b_eval.shape.len() == 2 {
                b_eval.shape[1]
            } else if b_eval.shape.len() <= 1 {
                1
            } else {
                return Err((
                    span,
                    "Matrix divide: left argument must be vector or matrix",
                ));
            };
            let b_data: Vec<f64> = b_eval.data.iter().map(|s| f64::from(s.clone())).collect();
            let mut aug: Vec<f64> = vec![0.0; n * (n + b_cols)];
            for i in 0..n {
                for j in 0..n {
                    aug[i * (n + b_cols) + j] = f64::from(a_eval.data[i * n + j].clone());
                }
                for j in 0..b_cols {
                    let b_idx = if b_eval.shape.len() == 2 {
                        i * b_cols + j
                    } else {
                        i
                    };
                    if b_idx < b_data.len() {
                        aug[i * (n + b_cols) + n + j] = b_data[b_idx];
                    }
                }
            }

            // Gauss-Jordan elimination
            let w = n + b_cols;
            for col in 0..n {
                let pivot_row = (col..n)
                    .max_by(|&a, &b| {
                        aug[a * w + col]
                            .abs()
                            .partial_cmp(&aug[b * w + col].abs())
                            .unwrap()
                    })
                    .unwrap();
                if aug[pivot_row * w + col].abs() < 1e-12 {
                    return Err((span, "Matrix divide: singular matrix"));
                }
                for j in 0..w {
                    aug.swap(col * w + j, pivot_row * w + j);
                }
                let pivot = aug[col * w + col];
                for j in 0..w {
                    aug[col * w + j] /= pivot;
                }
                for i in 0..n {
                    if i != col {
                        let factor = aug[i * w + col];
                        for j in 0..w {
                            aug[i * w + j] -= factor * aug[col * w + j];
                        }
                    }
                }
            }

            // Extract solution
            let mut data = Vec::with_capacity(n * b_cols);
            for i in 0..n {
                for j in 0..b_cols {
                    data.push(Scalar::Float(aug[i * w + n + j]));
                }
            }
            if b_cols == 1 {
                Ok(Val::vector(data))
            } else {
                Ok(Val::new(vec![n, b_cols], data))
            }
        }
        Expr::Left { span: _, lhs, rhs } => {
            debug!("Dyadic Left");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let _rhs_eval = eval(lexer, *rhs, env)?;
            Ok(lhs_eval)
        }
        Expr::Right { span: _, lhs, rhs } => {
            debug!("Dyadic Right");
            let _lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            Ok(rhs_eval)
        }
        Expr::LeftIdentity { span: _, arg } => {
            debug!("Monadic Left (identity)");
            eval(lexer, *arg, env)
        }
        Expr::RightIdentity { span: _, arg } => {
            debug!("Monadic Right (identity)");
            eval(lexer, *arg, env)
        }
        Expr::Tally { span: _, arg } => {
            debug!("Monadic Tally");
            let arg_eval = eval(lexer, *arg, env)?;
            let tally = if arg_eval.shape.is_empty() {
                1
            } else {
                arg_eval.shape[0]
            };
            Ok(Val::scalar(Scalar::Integer(tally as i64)))
        }
        Expr::Depth { span: _, arg } => {
            debug!("Monadic Depth");
            let arg_eval = eval(lexer, *arg, env)?;
            Ok(Val::scalar(Scalar::Integer(arg_eval.depth() as i64)))
        }
        Expr::Match { span: _, lhs, rhs } => {
            debug!("Dyadic Match");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            Ok(Val::scalar(Scalar::Integer(
                if lhs_eval.matches_val(&rhs_eval) {
                    1
                } else {
                    0
                },
            )))
        }
        Expr::NotMatch { span: _, lhs, rhs } => {
            debug!("Dyadic Not Match");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            Ok(Val::scalar(Scalar::Integer(
                if lhs_eval.matches_val(&rhs_eval) {
                    0
                } else {
                    1
                },
            )))
        }
        Expr::Find { span: _, lhs, rhs } => {
            debug!("Dyadic Find");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let pattern = &lhs_eval.data;
            let data = &rhs_eval.data;
            let plen = pattern.len();
            let dlen = data.len();
            let mut result = vec![Scalar::Integer(0); dlen];
            if plen > 0 && plen <= dlen {
                for i in 0..=(dlen - plen) {
                    if data[i..i + plen]
                        .iter()
                        .zip(pattern.iter())
                        .all(|(a, b)| a == b)
                    {
                        result[i] = Scalar::Integer(1);
                    }
                }
            }
            Ok(Val::new(rhs_eval.shape.clone(), result))
        }
        Expr::StringLiteral { span } => {
            debug!("String Literal");
            let raw = lexer.span_str(span);
            // Strip surrounding quotes
            let inner = &raw[1..raw.len() - 1];
            let data: Vec<Scalar> = inner.chars().map(Scalar::Char).collect();
            if data.is_empty() {
                Ok(Val::vector(vec![]))
            } else {
                Ok(Val::vector(data))
            }
        }
        Expr::Commute {
            span,
            lhs,
            operator,
            rhs,
        } => {
            debug!("Dyadic Commute");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let op_fn = get_operator_fn(operator);
            // Swap: apply as (rhs op lhs) instead of (lhs op rhs)
            apply_dyadic_operation(span, &rhs_eval, &lhs_eval, |a, b| {
                op_fn(a, b).ok_or_eyre("Commute operation failed")
            })
        }
        Expr::Selfie {
            span,
            operator,
            arg,
        } => {
            debug!("Monadic Selfie");
            let arg_eval = eval(lexer, *arg, env)?;
            let op_fn = get_operator_fn(operator);
            // Apply as (arg op arg)
            apply_dyadic_operation(span, &arg_eval, &arg_eval, |a, b| {
                op_fn(a, b).ok_or_eyre("Selfie operation failed")
            })
        }
        Expr::Split { span: _, arg } => {
            debug!("Monadic Split");
            let arg_eval = eval(lexer, *arg, env)?;
            if arg_eval.shape.len() <= 1 {
                // Vector or scalar: each element becomes a nested scalar
                let data = arg_eval
                    .data
                    .into_iter()
                    .map(|s| Scalar::Nested(Box::new(Val::scalar(s))))
                    .collect::<Vec<_>>();
                Ok(Val::vector(data))
            } else {
                let rows = arg_eval.shape[0];
                let cell_shape = arg_eval.shape[1..].to_vec();
                let cell_size: usize = cell_shape.iter().product();
                let data = (0..rows)
                    .map(|i| {
                        let start = i * cell_size;
                        let cell = Val::new(
                            cell_shape.clone(),
                            arg_eval.data[start..start + cell_size].to_vec(),
                        );
                        Scalar::Nested(Box::new(cell))
                    })
                    .collect();
                Ok(Val::vector(data))
            }
        }
        Expr::Mix { span, arg } => {
            debug!("Monadic Mix");
            let arg_eval = eval(lexer, *arg, env)?;
            let cells: Vec<Val> = arg_eval
                .data
                .iter()
                .filter_map(|s| match s {
                    Scalar::Nested(v) => Some((**v).clone()),
                    _ => None,
                })
                .collect();
            if cells.is_empty() {
                return Ok(arg_eval);
            }
            let cell_shape = cells[0].shape.clone();
            let mut shape = vec![cells.len()];
            shape.extend_from_slice(&cell_shape);
            let data = cells.into_iter().flat_map(|v| v.data).collect();
            let _ = span;
            Ok(Val::new(shape, data))
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

            let results: Vec<Result<Val, (Span, &'static str)>> = elements
                .into_iter()
                .map(|elem| eval(lexer, elem, env))
                .collect();

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
