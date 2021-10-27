use std::path::PathBuf;

use today_derive::*;

#[derive(Default, Semigroup, Monoid, Debug)]
pub struct PartialConfig {
    pub config_path: Last<PathBuf>,
    pub verbose: Option<Sum<u32>>,
}

pub trait Semigroup {
    fn combine(self, rhs: Self) -> Self;
}

pub trait Monoid: Semigroup {
    fn empty() -> Self;
}

impl<T: Semigroup> Semigroup for Option<T> {
    fn combine(self, rhs: Self) -> Self {
        match self {
            Some(left) => match rhs {
                Some(right) => Some(left.combine(right)),
                None => Some(left),
            },
            None => match rhs {
                Some(right) => Some(right),
                None => None,
            }
        }
    }
}

impl<T: Semigroup> Monoid for Option<T> {
    fn empty() -> Self {
        None
    }
}

#[derive(Debug, Default)]
pub struct Last<T>(Option<T>);

impl<T> From<T> for Last<T> {
    fn from(value: T) -> Self {
        Self(Some(value))
    }
}

impl<T> From<Option<T>> for Last<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T> Semigroup for Last<T> {
    fn combine(self, rhs: Self) -> Self {
        Self(rhs.0.or(self.0))
    }
}

impl<T> Monoid for Last<T> {
    fn empty() -> Self {
        Self(None)
    }
}

#[derive(Debug, Default, Semigroup, Monoid, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sum<T>(T);

impl<T: PartialEq> PartialEq<T> for Sum<T> {
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl<T: PartialOrd> PartialOrd<T> for Sum<T> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<T> From<T> for Sum<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[derive(Debug, Semigroup)]
pub struct Product<T>(T);

impl<T: Semigroup + num_traits::Num> Monoid for Product<T> {
    fn empty() -> Self {
        Self( num_traits::identities::One::one() )
    }
}

impl<T> From<T> for Product<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

macro_rules! impl_semigroup_with_addition {
    ( $($x:ty),* ) => {
        $(
            impl Semigroup for $x {
                fn combine(self, rhs: Self) -> Self {
                    self + rhs
                }
            }
        )*
    };
}

macro_rules! impl_monoid_for_default {
    ( $($x:ty),* ) => {
        $(
            impl Monoid for $x {
                fn empty() -> Self {
                    <$x as Default>::default()
                }
            }
        )*
    };
}

impl_semigroup_with_addition!(
    usize, isize, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);

impl_monoid_for_default!(
    usize, isize, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);

#[macro_export]
macro_rules! combine {
    ( $init:expr => $($x:expr),+ $(,)? ) => {
        $init$(
            .combine($x.into())
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_with_into() {
        let x = Last::empty()
            .combine(53.into())
            .combine(None.into())
            .combine(42.into());

        assert_eq!(x.0, Some(42));
    }

    #[test]
    fn sum_test() {
        let nums = vec![10, 24, 3, 7, 42];
        let sum = nums.into_iter().fold(Sum::empty(), |acc, x| acc.combine(Sum::from(x)));

        assert_eq!(sum, 86);
    }

    #[test]
    fn option_sum() {
        let sum = None
            .combine(Some(Sum::from(10)))
            .combine(None)
            .combine(Some(Sum::from(5)))
            .combine(Some(Sum::from(7)))
            .combine(None)
            .combine(Some(Sum::from(42)))
            .combine(None);

        assert_eq!(sum.unwrap(), 64);
    }

    #[test]
    fn option_combine_macro() {
        let sum: Option<Sum<i32>> = combine!(
            None =>
            Sum::from(10),
            None,
            Sum::from(5),
            Sum::from(7),
            None,
            Sum::from(42),
            None,
        );

        assert_eq!(sum.unwrap(), 64);
    }

    #[test]
    fn combine_macro() {
        let x = combine!{
            Last::from(53) => None, 42, {let b = None; b},
        };

        assert_eq!(x.0, Some(42));
    }
}
