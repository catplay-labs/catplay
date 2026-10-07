use crate::{DecodeError, EncodeError};

/// Values that can appear as operands in a statically generated predicate.
pub trait PredicateValue {
    fn predicate_integer(&self) -> Option<i128>;
}

pub fn decode_length<T: PredicateValue>(value: &T, field: &'static str) -> Result<usize, DecodeError> {
    value
        .predicate_integer()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or(DecodeError::InvalidFieldValue { field })
}

pub fn encode_length<T: PredicateValue>(value: &T, field: &'static str) -> Result<usize, EncodeError> {
    value
        .predicate_integer()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or(EncodeError::InvalidFieldValue { field })
}

macro_rules! predicate_integer {
    ($($ty:ty),* $(,)?) => { $(
        impl PredicateValue for $ty { fn predicate_integer(&self) -> Option<i128> { Some(*self as i128) } }
    )* };
}
predicate_integer!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);
impl PredicateValue for bool {
    fn predicate_integer(&self) -> Option<i128> {
        Some(i128::from(*self))
    }
}
impl<T: PredicateValue> PredicateValue for Option<T> {
    fn predicate_integer(&self) -> Option<i128> {
        self.as_ref().and_then(PredicateValue::predicate_integer)
    }
}
impl<T: PredicateValue + ?Sized> PredicateValue for &T {
    fn predicate_integer(&self) -> Option<i128> {
        (*self).predicate_integer()
    }
}
impl<const N: usize> PredicateValue for [u8; N] {
    fn predicate_integer(&self) -> Option<i128> {
        None
    }
}

pub mod predicate_eval {
    use super::PredicateValue;

    pub fn eq<L: PredicateValue, R: PredicateValue>(left: &L, right: &R) -> bool {
        compare(left, right, |a, b| a == b)
    }
    pub fn ne<L: PredicateValue, R: PredicateValue>(left: &L, right: &R) -> bool {
        compare(left, right, |a, b| a != b)
    }
    pub fn lt<L: PredicateValue, R: PredicateValue>(left: &L, right: &R) -> bool {
        compare(left, right, |a, b| a < b)
    }
    pub fn le<L: PredicateValue, R: PredicateValue>(left: &L, right: &R) -> bool {
        compare(left, right, |a, b| a <= b)
    }
    pub fn gt<L: PredicateValue, R: PredicateValue>(left: &L, right: &R) -> bool {
        compare(left, right, |a, b| a > b)
    }
    pub fn ge<L: PredicateValue, R: PredicateValue>(left: &L, right: &R) -> bool {
        compare(left, right, |a, b| a >= b)
    }
    pub fn truthy<T: PredicateValue>(value: &T) -> bool {
        value.predicate_integer().is_some_and(|v| v != 0)
    }
    fn compare<L: PredicateValue, R: PredicateValue>(left: &L, right: &R, op: impl FnOnce(i128, i128) -> bool) -> bool {
        match (left.predicate_integer(), right.predicate_integer()) {
            (Some(a), Some(b)) => op(a, b),
            _ => false,
        }
    }
}
