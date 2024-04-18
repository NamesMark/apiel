use conv::ConvUtil;
use eyre::Result;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub};

#[derive(Debug, Clone, Copy, PartialOrd)]
pub enum Val {
    Float(f64),
    Integer(i64),
}

impl Val {
    fn promote_pair(a: &Val, b: &Val) -> (Val, Val) {
        match (a, b) {
            (Val::Integer(i), Val::Integer(j)) => (Val::Integer(*i), Val::Integer(*j)),
            (Val::Float(f), Val::Float(g)) => (Val::Float(*f), Val::Float(*g)),
            (Val::Integer(i), Val::Float(f)) => (Val::Float(*i as f64), Val::Float(*f)),
            (Val::Float(f), Val::Integer(i)) => (Val::Float(*f), Val::Float(*i as f64)),
        }
    }
}

// #[derive(Debug, Clone)]
// enum ValArray {
//     Single(Val),
//     Array(Vec<ValArray>),
// }

// impl From<Val> for ValArray {
//     fn from(val: Val) -> Self {
//         Self::Single(val)
//     }
// }

impl TryFrom<Val> for usize {
    type Error = &'static str;

    fn try_from(value: Val) -> Result<Self, Self::Error> {
        match value {
            Val::Integer(val) => val
                .try_into()
                .map_err(|_| "Failed to convert i64 into usize"),
            Val::Float(val) => {
                if val.fract() == 0.0 && val >= 0.0 {
                    val.approx_as::<usize>()
                        .map_err(|_| "Failed to convert f64 into usize")
                } else {
                    Err("Float is not a whole number or is negative")
                }
            }
        }
    }
}

impl From<Val> for f64 {
    fn from(value: Val) -> Self {
        match value {
            Val::Integer(val) => val as f64,
            Val::Float(val) => val,
        }
    }
}

impl PartialEq for Val {
    fn eq(&self, other: &Val) -> bool {
        match (self, other) {
            (Val::Integer(i), Val::Integer(j)) => i == j,
            (Val::Float(f), Val::Float(g)) => f == g,
            (Val::Integer(i), Val::Float(f)) => *i as f64 == *f,
            (Val::Float(f), Val::Integer(i)) => *f == *i as f64,
        }
    }
}

impl Eq for Val {}

impl Ord for Val {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Val::Integer(i), Val::Integer(j)) => i.cmp(j),
            (Val::Float(f), Val::Float(g)) => {
                if f.is_nan() && g.is_nan() {
                    std::cmp::Ordering::Equal
                } else if f.is_nan() {
                    std::cmp::Ordering::Less
                } else if g.is_nan() {
                    std::cmp::Ordering::Greater
                } else {
                    f.partial_cmp(g).unwrap()
                }
            }
            (Val::Integer(i), Val::Float(f)) => (*i as f64).partial_cmp(f).unwrap_or_else(|| {
                if f.is_nan() {
                    std::cmp::Ordering::Greater
                } else {
                    (*i as f64).partial_cmp(f).unwrap()
                }
            }),
            (Val::Float(f), Val::Integer(i)) => f.partial_cmp(&(*i as f64)).unwrap_or_else(|| {
                if f.is_nan() {
                    std::cmp::Ordering::Less
                } else {
                    f.partial_cmp(&(*i as f64)).unwrap()
                }
            }),
        }
    }
}

impl std::ops::Add for Val {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let promoted = Self::promote_pair(&self, &other);

        match promoted {
            (Val::Integer(i), Val::Integer(j)) => Val::Integer(i + j),
            (Val::Float(f), Val::Float(g)) => Val::Float(f + g),
            // Panic here because can't return Result:
            _ => panic!("BUG: Unexpected type mismatch after promotion"),
        }
    }
}

impl CheckedAdd for Val {
    fn checked_add(&self, other: &Self) -> Option<Self> {
        let promoted_result = Self::promote_pair(self, other);
        match promoted_result {
            (Val::Integer(i), Val::Integer(j)) => i.checked_add(j).map(Val::Integer),
            (Val::Float(f), Val::Float(g)) => Some(Val::Float(f + g)),
            _ => None,
        }
    }
}

impl std::ops::Sub for Val {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let promoted = Self::promote_pair(&self, &other);

        match promoted {
            (Val::Integer(i), Val::Integer(j)) => Val::Integer(i - j),
            (Val::Float(f), Val::Float(g)) => Val::Float(f - g),
            // Panic here because can't return Result:
            _ => panic!("BUG: Unexpected type mismatch after promotion"),
        }
    }
}

impl CheckedSub for Val {
    fn checked_sub(&self, other: &Self) -> Option<Self> {
        let promoted_result = Self::promote_pair(self, other);
        match promoted_result {
            (Val::Integer(i), Val::Integer(j)) => i.checked_sub(j).map(Val::Integer),
            (Val::Float(f), Val::Float(g)) => Some(Val::Float(f - g)),
            _ => None,
        }
    }
}

impl std::ops::Mul for Val {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        let promoted = Self::promote_pair(&self, &other);

        match promoted {
            (Val::Integer(i), Val::Integer(j)) => Val::Integer(i * j),
            (Val::Float(f), Val::Float(g)) => Val::Float(f * g),
            // Panic here because can't return Result:
            _ => panic!("BUG: Unexpected type mismatch after promotion"),
        }
    }
}

impl CheckedMul for Val {
    fn checked_mul(&self, other: &Self) -> Option<Self> {
        let promoted_result = Self::promote_pair(self, other);
        match promoted_result {
            (Val::Integer(i), Val::Integer(j)) => i.checked_mul(j).map(Val::Integer),
            (Val::Float(f), Val::Float(g)) => Some(Val::Float(f * g)),
            _ => None,
        }
    }
}

impl std::ops::Div for Val {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        let promoted = Self::promote_pair(&self, &other);

        match promoted {
            (Val::Integer(i), Val::Integer(j)) => Val::Float(i as f64 / j as f64),
            (Val::Float(f), Val::Float(g)) => Val::Float(f / g),
            // Panic here because can't return Result:
            _ => panic!("BUG: Unexpected type mismatch after promotion"),
        }
    }
}

impl CheckedDiv for Val {
    fn checked_div(&self, other: &Self) -> Option<Self> {
        let promoted_result = Self::promote_pair(self, other);
        match promoted_result {
            (Val::Integer(i), Val::Integer(j)) => Some(Val::Float(i as f64 / j as f64)),
            (Val::Float(f), Val::Float(g)) => Some(Val::Float(f / g)),
            _ => None,
        }
    }
}

pub trait CheckedPow: Sized {
    fn checked_pow(&self, power: usize) -> Option<Self>;
    fn checked_powf(&self, other: f64) -> Option<Self>;
}

impl CheckedPow for Val {
    fn checked_pow(&self, other: usize) -> Option<Self> {
        match self {
            Val::Integer(i) => i.checked_pow(other as u32).map(Val::Integer),
            Val::Float(f) => Some(Val::Float(num_traits::pow::pow(*f, other))),
        }
    }

    fn checked_powf(&self, other: f64) -> Option<Self> {
        match self {
            Val::Integer(i) => Some(Val::Float((*i as f64).powf(other))),
            Val::Float(f) => Some(Val::Float(f.powf(other))),
        }
    }
}

pub trait Log: Sized {
    fn log(&self, base: &Self) -> Option<Self>;
}

impl Log for Val {
    fn log(&self, base: &Self) -> Option<Self> {
        Some(Val::Float(f64::from(*self).log(f64::from(*base))))
    }
}

impl CheckedNeg for Val {
    fn checked_neg(&self) -> Option<Self> {
        match self {
            Val::Integer(i) => i.checked_neg().map(Val::Integer),
            Val::Float(f) => Some(Val::Float(-f)),
        }
    }
}
