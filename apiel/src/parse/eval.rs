use std::collections::HashMap;
use std::rc::Rc;
use super::*;
use crate::parse::apiel_y::{Expr, Operator};
use val::{Scalar, Val, CheckedPow, Log};
use eyre::{OptionExt, Result};
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub};
use rand::Rng;
use tracing::{debug, error};

#[derive(Debug, Clone)]
pub struct StoredDfn {
    pub body: Rc<Expr>,
    pub source: String,  // original input line for correct span resolution
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

fn eval_stored_dfn(
    stored: &StoredDfn,
    env: &mut Env,
) -> Result<Val, (Span, String)> {
    use crate::parse::apiel_l;
    let lexerdef = apiel_l::lexerdef();
    let lex = lexerdef.lexer(&stored.source);
    eval(&lex, (*stored.body).clone(), env)
        .map_err(|(span, msg)| (span, msg.to_string()))
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

fn get_operator_fn(op: Operator) -> fn(&Scalar, &Scalar) -> Option<Scalar> {
    match op {
        Operator::Add => |a, b| a.checked_add(b),
        Operator::Subtract => |a, b| a.checked_sub(b),
        Operator::Multiply => |a, b| a.checked_mul(b),
        Operator::Divide => |a, b| a.checked_div(b),
        Operator::Equal => |a, b| Some(Scalar::Integer(if a == b { 1 } else { 0 })),
        Operator::LessThan => |a, b| Some(Scalar::Integer(if a < b { 1 } else { 0 })),
        Operator::GreaterThan => |a, b| Some(Scalar::Integer(if a > b { 1 } else { 0 })),
        Operator::Max => |a, b| Some(if a >= b { *a } else { *b }),
        Operator::Min => |a, b| Some(if a <= b { *a } else { *b }),
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
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            apply_dyadic_operation(span, &lhs_eval, &rhs_eval, |base: &Scalar, value: &Scalar| {
                value.log(base).ok_or_eyre(format!(
                    "Somehow failed to compute the logarithm of {base:?} and {value:?}: {span:?}"
                ))
            })
        }
        Expr::Min { span, lhs, rhs } => {
            debug!("Dyadic Min");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

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
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

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
                let k = f64::from(*a);
                let n = f64::from(*b);
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
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

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
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
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
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let _ = span;

            let data = rhs_eval.data.iter()
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

            let new_shape: Vec<usize> = lhs_eval.data.iter()
                .map(|s| usize::try_from(*s).map_err(|_| (span, "Reshape dimensions must be non-negative integers")))
                .collect::<Result<Vec<usize>, _>>()?;

            let total: usize = new_shape.iter().product::<usize>().max(1);
            let data: Vec<Scalar> = rhs_eval.data.iter()
                .copied()
                .cycle()
                .take(total)
                .collect();

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
            let n = match lhs_eval.data[0] {
                Scalar::Integer(i) => i,
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
                let data: Vec<Scalar> = rhs_eval.data.iter()
                    .flat_map(|v| std::iter::repeat_n(*v, n))
                    .collect();
                Ok(Val::vector(data))
            } else {
                if lhs_eval.data.len() != rhs_eval.data.len() {
                    return Err((span, "Replicate: left and right arguments must have same length"));
                }
                let data: Vec<Scalar> = lhs_eval.data.iter()
                    .zip(rhs_eval.data.iter())
                    .flat_map(|(count, val)| {
                        let n = match count {
                            Scalar::Integer(i) => *i as usize,
                            Scalar::Float(f) => *f as usize,
                        };
                        std::iter::repeat_n(*val, n)
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
                };
                if n > 0 {
                    match rhs_iter.next() {
                        Some(&v) => {
                            for _ in 0..n { data.push(v); }
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
                let x = f64::from(*val);
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
                let mut d: Vec<Scalar> = rhs_eval.data.iter().copied().take(abs_n).collect();
                while d.len() < abs_n { d.push(Scalar::Integer(0)); }
                d
            } else {
                let skip = if abs_n <= len { len - abs_n } else { 0 };
                let mut d: Vec<Scalar> = rhs_eval.data.iter().copied().skip(skip).collect();
                while d.len() < abs_n { d.insert(0, Scalar::Integer(0)); }
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
        Expr::OuterProduct { span, lhs, operator, rhs } => {
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

            apply_monadic_operation(span, &arg_eval, |&n| {
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
                    Scalar::Float(_) => {
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
            };

            apply_monadic_operation(span, &arg_eval, magnitude_operation)
        }
        Expr::Ceil { span, arg } => {
            debug!("Monadic Ceiling");
            let arg_eval = eval(lexer, *arg, env)?;

            let ceil_operation = |a: &Scalar| match a {
                Scalar::Integer(i) => Ok(Scalar::Integer(*i)),
                Scalar::Float(f) => Ok(Scalar::Float(f.ceil())),
            };

            apply_monadic_operation(span, &arg_eval, ceil_operation)
        }
        Expr::Floor { span, arg } => {
            debug!("Monadic Floor");
            let arg_eval = eval(lexer, *arg, env)?;

            let floor_operation = |a: &Scalar| match a {
                Scalar::Integer(i) => Ok(Scalar::Integer(*i)),
                Scalar::Float(f) => Ok(Scalar::Float(f.floor())),
            };

            apply_monadic_operation(span, &arg_eval, floor_operation)
        }
        Expr::MonadicMax { span, arg } => {
            debug!("Monadic Maximum");
            let arg_eval = eval(lexer, *arg, env)?;

            arg_eval.data.iter()
                .max()
                .ok_or((span, "Cannot find max"))
                .map(|&num| Val::scalar(num))
        }
        Expr::MonadicMin { span, arg } => {
            debug!("Monadic Minimum");
            let arg_eval = eval(lexer, *arg, env)?;

            arg_eval.data.iter()
                .min()
                .ok_or((span, "Cannot find min"))
                .map(|&num| Val::scalar(num))
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
        Expr::Shape { arg, .. } => {
            debug!("Monadic Shape");
            let arg_eval = eval(lexer, *arg, env)?;
            let data: Vec<Scalar> = arg_eval.shape.iter()
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
                            data[c * rows + r] = arg_eval.data[r * cols + c];
                        }
                    }
                    Ok(Val::new(vec![cols, rows], data))
                }
                _ => Err((Span::new(0, 0), "Transpose only supports rank 0, 1, or 2")),
            }
        }
        Expr::GradeUp { span, arg } => {
            debug!("Monadic Grade Up");
            let arg_eval = eval(lexer, *arg, env)?;
            let _ = span;
            let mut indices: Vec<usize> = (0..arg_eval.data.len()).collect();
            indices.sort_by(|&a, &b| arg_eval.data[a].cmp(&arg_eval.data[b]));
            let data: Vec<Scalar> = indices.iter().map(|&i| Scalar::Integer(i as i64 + 1)).collect();
            Ok(Val::vector(data))
        }
        Expr::GradeDown { span, arg } => {
            debug!("Monadic Grade Down");
            let arg_eval = eval(lexer, *arg, env)?;
            let _ = span;
            let mut indices: Vec<usize> = (0..arg_eval.data.len()).collect();
            indices.sort_by(|&a, &b| arg_eval.data[b].cmp(&arg_eval.data[a]));
            let data: Vec<Scalar> = indices.iter().map(|&i| Scalar::Integer(i as i64 + 1)).collect();
            Ok(Val::vector(data))
        }
        Expr::Reduce {
            span,
            operator,
            term,
        } => {
            debug!("Reduce");
            let term_eval = eval(lexer, *term, env)?;

            // APL reduce is a right-fold: f/ a b c d = a f (b f (c f d))
            let op_fn = get_operator_fn(operator);

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
                let result = prefix.iter().rev().copied().try_fold(None::<Scalar>, |acc, n| {
                    match acc {
                        None => Some(Some(n)),
                        Some(right) => op_fn(&n, &right).map(Some),
                    }
                }).flatten()
                    .ok_or((span, "Arithmetic error in Scan"))?;
                data.push(result);
            }
            Ok(Val::vector(data))
        }
        Expr::Variable { span, name } => {
            debug!("Variable: {name}");
            env.vars.get(&name).cloned().ok_or((span, "Undefined variable"))
        }
        Expr::Omega { span } => {
            env.vars.get("⍵").cloned().ok_or((span, "⍵ used outside of a dfn"))
        }
        Expr::Alpha { span } => {
            env.vars.get("⍺").cloned().ok_or((span, "⍺ used outside of a dfn"))
        }
        Expr::MonadicDfn { span, body, rhs } => {
            debug!("Monadic Dfn");
            let rhs_val = eval(lexer, *rhs, env)?;
            let body_rc = Rc::new(*body);
            let stored = StoredDfn { body: Rc::clone(&body_rc), source: lexer.span_str(span).to_string() };
            let mut dfn_env = env.clone();
            dfn_env.vars.insert("⍵".to_string(), rhs_val);
            dfn_env.fns.insert("∇".to_string(), stored);
            eval(lexer, (*body_rc).clone(), &mut dfn_env)
        }
        Expr::DyadicDfn { span, lhs, body, rhs } => {
            debug!("Dyadic Dfn");
            let lhs_val = eval(lexer, *lhs, env)?;
            let rhs_val = eval(lexer, *rhs, env)?;
            let body_rc = Rc::new(*body);
            let stored = StoredDfn { body: Rc::clone(&body_rc), source: lexer.span_str(span).to_string() };
            let mut dfn_env = env.clone();
            dfn_env.vars.insert("⍺".to_string(), lhs_val);
            dfn_env.vars.insert("⍵".to_string(), rhs_val);
            dfn_env.fns.insert("∇".to_string(), stored);
            eval(lexer, (*body_rc).clone(), &mut dfn_env)
        }
        Expr::SelfCall { span, arg } => {
            debug!("Self-reference ∇");
            let arg_val = eval(lexer, *arg, env)?;
            let stored = env.fns.get("∇").cloned()
                .ok_or((span, "∇ used outside of a dfn"))?;
            let mut self_env = env.clone();
            self_env.vars.insert("⍵".to_string(), arg_val);
            eval_stored_dfn(&stored, &mut self_env)
                .map_err(|(_span, msg)| (span, Box::leak(msg.into_boxed_str()) as &'static str))
        }
        Expr::DfnGuard { cond, result, rest, .. } => {
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
            let stored = StoredDfn { body: Rc::new(*body), source: lexer.span_str(span).to_string() };
            env.fns.insert(name, stored);
            Ok(Val::scalar(Scalar::Integer(0)))
        }
        Expr::NamedMonadic { span, name, rhs } => {
            debug!("Named Monadic: {name}");
            let stored = env.fns.get(&name).cloned()
                .ok_or((span, "Undefined function"))?;
            let rhs_val = eval(lexer, *rhs, env)?;
            let mut dfn_env = env.clone();
            dfn_env.vars.insert("⍵".to_string(), rhs_val);
            dfn_env.fns.insert("∇".to_string(), stored.clone());
            eval_stored_dfn(&stored, &mut dfn_env)
                .map_err(|(_span, msg)| (span, Box::leak(msg.into_boxed_str()) as &'static str))
        }
        Expr::NamedDyadic { span, lhs, name, rhs } => {
            debug!("Named Dyadic: {name}");
            let stored = env.fns.get(&name).cloned()
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
        Expr::First { span, arg } => {
            debug!("Monadic First");
            let arg_eval = eval(lexer, *arg, env)?;
            let _ = span;
            Ok(Val::scalar(arg_eval.data.into_iter().next().unwrap_or(Scalar::Integer(0))))
        }
        Expr::Unique { arg, .. } => {
            debug!("Monadic Unique");
            let arg_eval = eval(lexer, *arg, env)?;
            let mut seen = Vec::new();
            for v in &arg_eval.data {
                if !seen.contains(v) { seen.push(*v); }
            }
            Ok(Val::vector(seen))
        }
        Expr::Not { span, arg } => {
            debug!("Monadic Not");
            let arg_eval = eval(lexer, *arg, env)?;
            apply_monadic_operation(span, &arg_eval, |a| {
                Ok(Scalar::Integer(if *a == Scalar::Integer(0) { 1 } else { 0 }))
            })
        }
        Expr::Union { span, lhs, rhs } => {
            debug!("Dyadic Union");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let _ = span;
            let mut data = lhs_eval.data;
            for v in &rhs_eval.data {
                if !data.contains(v) { data.push(*v); }
            }
            Ok(Val::vector(data))
        }
        Expr::Intersection { span, lhs, rhs } => {
            debug!("Dyadic Intersection");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let _ = span;
            let data: Vec<Scalar> = lhs_eval.data.iter()
                .filter(|v| rhs_eval.data.contains(v))
                .copied()
                .collect();
            Ok(Val::vector(data))
        }
        Expr::Without { span, lhs, rhs } => {
            debug!("Dyadic Without");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;
            let _ = span;
            let data: Vec<Scalar> = lhs_eval.data.iter()
                .filter(|v| !rhs_eval.data.contains(v))
                .copied()
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
            let base = f64::from(lhs_eval.data[0]);
            let result = rhs_eval.data.iter().fold(0.0_f64, |acc, v| {
                acc * base + f64::from(*v)
            });
            Ok(Val::scalar(Scalar::Integer(result as i64)))
        }
        Expr::Encode { span, lhs, rhs } => {
            debug!("Dyadic Encode");
            let lhs_eval = eval(lexer, *lhs, env)?;
            let rhs_eval = eval(lexer, *rhs, env)?;

            if !rhs_eval.is_scalar() {
                return Err((span, "Encode: right argument must be a scalar"));
            }
            let mut n = f64::from(rhs_eval.data[0]) as i64;
            let bases: Vec<i64> = lhs_eval.data.iter().map(|v| f64::from(*v) as i64).collect();
            let mut data: Vec<Scalar> = vec![Scalar::Integer(0); bases.len()];
            for i in (0..bases.len()).rev() {
                if bases[i] != 0 {
                    data[i] = Scalar::Integer(n % bases[i]);
                    n /= bases[i];
                }
            }
            Ok(Val::vector(data))
        }
        Expr::InnerProduct { span, lhs, f, g, rhs } => {
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
                    let products: Vec<Scalar> = lhs_eval.data.iter().zip(rhs_eval.data.iter())
                        .map(|(a, b)| g_fn(a, b).ok_or((span, "Inner product g failed")))
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = products.iter().rev().copied().try_fold(None::<Scalar>, |acc, n| {
                        match acc {
                            None => Some(Some(n)),
                            Some(right) => f_fn(&n, &right).map(Some),
                        }
                    }).flatten().ok_or((span, "Inner product f failed"))?;
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
                                .map(|p| g_fn(&lhs_eval.data[i * k + p], &rhs_eval.data[p * n + j])
                                    .ok_or((span, "Inner product g failed")))
                                .collect::<Result<Vec<_>, _>>()?;
                            let result = products.iter().copied().reduce(|a, b| f_fn(&a, &b).unwrap_or(a))
                                .ok_or((span, "Inner product f failed"))?;
                            data.push(result);
                        }
                    }
                    Ok(Val::new(vec![m, n], data))
                }
                _ => Err((span, "Inner product: only rank 1 and 2 supported")),
            }
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
                elements.into_iter().map(|elem| eval(lexer, elem, env)).collect();

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
