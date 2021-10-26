use std::path::PathBuf;

use today_derive::*;

#[derive(Default, Semigroup, Monoid, Debug)]
pub struct PartialConfig {
    pub config_path: Last<PathBuf>,
    pub verbose: Sum<u32>,
}

pub trait Semigroup {
    fn combine(self, rhs: Self) -> Self;
}

pub trait Monoid: Semigroup {
    fn empty() -> Self;
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
pub struct Sum<T: num_traits::Num>(T);

impl<T: num_traits::Num + PartialEq> PartialEq<T> for Sum<T> {
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl<T: num_traits::Num + PartialOrd> PartialOrd<T> for Sum<T> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<T: num_traits::Num> From<T> for Sum<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[derive(Debug, Semigroup)]
pub struct Product<T: num_traits::Num>(T);

impl<T: Semigroup + num_traits::Num> Monoid for Product<T> {
    fn empty() -> Self {
        Self( num_traits::identities::One::one() )
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
    ( $init:expr => $($x:expr;)+ ) => {
        $init$(
            .combine($x.into())
        )*
    };

    ( $init:expr => $($x:expr);+ ) => {
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
    fn combine_macro() {
        let x = combine!{
            Last::from(53) => None; 42; {let b = None; b};
        };

        assert_eq!(x.0, Some(42));
    }
}
