use conv::ConvUtil;
use eyre::Result;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub};

#[derive(Debug, Clone, Copy)]
pub enum Scalar {
    Float(f64),
    Integer(i64),
}

impl Scalar {
    fn promote_pair(a: &Scalar, b: &Scalar) -> (Scalar, Scalar) {
        match (a, b) {
            (Scalar::Integer(i), Scalar::Integer(j)) => (Scalar::Integer(*i), Scalar::Integer(*j)),
            (Scalar::Float(f), Scalar::Float(g)) => (Scalar::Float(*f), Scalar::Float(*g)),
            (Scalar::Integer(i), Scalar::Float(f)) => (Scalar::Float(*i as f64), Scalar::Float(*f)),
            (Scalar::Float(f), Scalar::Integer(i)) => (Scalar::Float(*f), Scalar::Float(*i as f64)),
        }
    }
}

impl TryFrom<Scalar> for usize {
    type Error = &'static str;

    fn try_from(value: Scalar) -> Result<Self, Self::Error> {
        match value {
            Scalar::Integer(val) => val
                .try_into()
                .map_err(|_| "Failed to convert i64 into usize"),
            Scalar::Float(val) => {
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

impl From<Scalar> for f64 {
    fn from(value: Scalar) -> Self {
        match value {
            Scalar::Integer(val) => val as f64,
            Scalar::Float(val) => val,
        }
    }
}

impl PartialEq for Scalar {
    fn eq(&self, other: &Scalar) -> bool {
        match (self, other) {
            (Scalar::Integer(i), Scalar::Integer(j)) => i == j,
            (Scalar::Float(f), Scalar::Float(g)) => f == g,
            (Scalar::Integer(i), Scalar::Float(f)) => *i as f64 == *f,
            (Scalar::Float(f), Scalar::Integer(i)) => *f == *i as f64,
        }
    }
}

impl Eq for Scalar {}

impl PartialOrd for Scalar {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Scalar {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Scalar::Integer(i), Scalar::Integer(j)) => i.cmp(j),
            (Scalar::Float(f), Scalar::Float(g)) => {
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
            (Scalar::Integer(i), Scalar::Float(f)) => (*i as f64).partial_cmp(f).unwrap_or_else(|| {
                if f.is_nan() {
                    std::cmp::Ordering::Greater
                } else {
                    (*i as f64).partial_cmp(f).unwrap()
                }
            }),
            (Scalar::Float(f), Scalar::Integer(i)) => f.partial_cmp(&(*i as f64)).unwrap_or_else(|| {
                if f.is_nan() {
                    std::cmp::Ordering::Less
                } else {
                    f.partial_cmp(&(*i as f64)).unwrap()
                }
            }),
        }
    }
}

impl std::ops::Add for Scalar {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let promoted = Self::promote_pair(&self, &other);

        match promoted {
            (Scalar::Integer(i), Scalar::Integer(j)) => Scalar::Integer(i + j),
            (Scalar::Float(f), Scalar::Float(g)) => Scalar::Float(f + g),
            _ => panic!("BUG: Unexpected type mismatch after promotion"),
        }
    }
}

impl CheckedAdd for Scalar {
    fn checked_add(&self, other: &Self) -> Option<Self> {
        let promoted_result = Self::promote_pair(self, other);
        match promoted_result {
            (Scalar::Integer(i), Scalar::Integer(j)) => i.checked_add(j).map(Scalar::Integer),
            (Scalar::Float(f), Scalar::Float(g)) => Some(Scalar::Float(f + g)),
            _ => None,
        }
    }
}

impl std::ops::Sub for Scalar {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let promoted = Self::promote_pair(&self, &other);

        match promoted {
            (Scalar::Integer(i), Scalar::Integer(j)) => Scalar::Integer(i - j),
            (Scalar::Float(f), Scalar::Float(g)) => Scalar::Float(f - g),
            _ => panic!("BUG: Unexpected type mismatch after promotion"),
        }
    }
}

impl CheckedSub for Scalar {
    fn checked_sub(&self, other: &Self) -> Option<Self> {
        let promoted_result = Self::promote_pair(self, other);
        match promoted_result {
            (Scalar::Integer(i), Scalar::Integer(j)) => i.checked_sub(j).map(Scalar::Integer),
            (Scalar::Float(f), Scalar::Float(g)) => Some(Scalar::Float(f - g)),
            _ => None,
        }
    }
}

impl std::ops::Mul for Scalar {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        let promoted = Self::promote_pair(&self, &other);

        match promoted {
            (Scalar::Integer(i), Scalar::Integer(j)) => Scalar::Integer(i * j),
            (Scalar::Float(f), Scalar::Float(g)) => Scalar::Float(f * g),
            _ => panic!("BUG: Unexpected type mismatch after promotion"),
        }
    }
}

impl CheckedMul for Scalar {
    fn checked_mul(&self, other: &Self) -> Option<Self> {
        let promoted_result = Self::promote_pair(self, other);
        match promoted_result {
            (Scalar::Integer(i), Scalar::Integer(j)) => i.checked_mul(j).map(Scalar::Integer),
            (Scalar::Float(f), Scalar::Float(g)) => Some(Scalar::Float(f * g)),
            _ => None,
        }
    }
}

impl std::ops::Div for Scalar {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        let promoted = Self::promote_pair(&self, &other);

        match promoted {
            (Scalar::Integer(i), Scalar::Integer(j)) => Scalar::Float(i as f64 / j as f64),
            (Scalar::Float(f), Scalar::Float(g)) => Scalar::Float(f / g),
            _ => panic!("BUG: Unexpected type mismatch after promotion"),
        }
    }
}

impl CheckedDiv for Scalar {
    fn checked_div(&self, other: &Self) -> Option<Self> {
        let promoted_result = Self::promote_pair(self, other);
        match promoted_result {
            (Scalar::Integer(i), Scalar::Integer(j)) => Some(Scalar::Float(i as f64 / j as f64)),
            (Scalar::Float(f), Scalar::Float(g)) => Some(Scalar::Float(f / g)),
            _ => None,
        }
    }
}

pub trait CheckedPow: Sized {
    fn checked_pow(&self, power: usize) -> Option<Self>;
    fn checked_powf(&self, other: f64) -> Option<Self>;
}

impl CheckedPow for Scalar {
    fn checked_pow(&self, other: usize) -> Option<Self> {
        match self {
            Scalar::Integer(i) => i.checked_pow(other as u32).map(Scalar::Integer),
            Scalar::Float(f) => Some(Scalar::Float(num_traits::pow::pow(*f, other))),
        }
    }

    fn checked_powf(&self, other: f64) -> Option<Self> {
        match self {
            Scalar::Integer(i) => Some(Scalar::Float((*i as f64).powf(other))),
            Scalar::Float(f) => Some(Scalar::Float(f.powf(other))),
        }
    }
}

pub trait Log: Sized {
    fn log(&self, base: &Self) -> Option<Self>;
}

impl Log for Scalar {
    fn log(&self, base: &Self) -> Option<Self> {
        Some(Scalar::Float(f64::from(*self).log(f64::from(*base))))
    }
}

impl CheckedNeg for Scalar {
    fn checked_neg(&self) -> Option<Self> {
        match self {
            Scalar::Integer(i) => i.checked_neg().map(Scalar::Integer),
            Scalar::Float(f) => Some(Scalar::Float(-f)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Val {
    pub shape: Vec<usize>,
    pub data: Vec<Scalar>,
}

impl Val {
    pub fn scalar(s: Scalar) -> Self {
        Val { shape: vec![], data: vec![s] }
    }

    pub fn vector(data: Vec<Scalar>) -> Self {
        let len = data.len();
        Val { shape: vec![len], data }
    }

    pub fn new(shape: Vec<usize>, data: Vec<Scalar>) -> Self {
        Val { shape, data }
    }

    pub fn is_scalar(&self) -> bool {
        self.shape.is_empty()
    }

    pub fn from_f64s(values: &[f64]) -> Self {
        let data: Vec<Scalar> = values.iter().map(|&v| {
            if v.fract() == 0.0 && v.abs() < i64::MAX as f64 {
                Scalar::Integer(v as i64)
            } else {
                Scalar::Float(v)
            }
        }).collect();
        if data.len() == 1 {
            Val::scalar(data[0])
        } else {
            Val::vector(data)
        }
    }
}
