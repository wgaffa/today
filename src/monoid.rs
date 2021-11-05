use std::marker::PhantomData;
use today_derive::*;

use crate::semigroup::Semigroup;

pub trait Monoid: Semigroup {
    fn empty() -> Self;
}

impl<T: Semigroup> Monoid for Option<T> {
    fn empty() -> Self {
        None
    }
}

#[derive(Debug)]
pub struct Last<T>(pub Option<T>);

impl<T> Default for Last<T> {
    fn default() -> Self {
        Self(None)
    }
}

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

impl<T> From<Last<T>> for Option<T> {
    fn from(value: Last<T>) -> Self {
        value.0
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

#[derive(Debug, Semigroup, Monoid, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sum<T>(pub T);

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
pub struct Product<T>(pub T);

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

impl_monoid_for_default!(
    usize, isize, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);

impl<T> Monoid for PhantomData<T> {
    fn empty() -> Self {
        Self
    }
}

#[macro_export]
macro_rules! monoid_default {
    ($t:ty : $($i:ident),*) => {
        impl Monoid for $t {
            fn empty() -> Self {
                Self {
                    $(
                        $i: Monoid::empty(),
                    )*
                }
            }
        }
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
        let sum: Option<Sum<i32>> = crate::combine!(
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
        let x = crate::combine!{
            Last::from(53) => None, 42, {let b = None; b},
        };

        assert_eq!(x.0, Some(42));
    }

    #[test]
    fn last_to_option_conversion() {
        let last = Last::from(42);
        let res: Option<i32> = last.into();

        assert_eq!(res, Some(42));
    }
}
