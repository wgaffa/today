use std::marker::PhantomData;

pub trait Semigroup {
    fn combine(self, rhs: Self) -> Self;
}

#[macro_export]
macro_rules! combine {
    ( $init:expr => $($x:expr),+ $(,)? ) => {
        $init$(
            .combine($x.into())
        )*
    };
}

impl<T: Semigroup> Semigroup for Option<T> {
    fn combine(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Some(left), Some(right)) => Some(left.combine(right)),
            (left, right) => left.or(right),
        }
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

impl_semigroup_with_addition!(
    usize, isize, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64
);

impl Semigroup for bool {
    fn combine(self, rhs: Self) -> Self {
        self || rhs
    }
}

impl<T> Semigroup for PhantomData<T> {
    fn combine(self, _rhs: Self) -> Self {
        self
    }
}

#[macro_export]
macro_rules! semigroup_default {
    ($t:ty : $($i:ident),*) => {
        impl Semigroup for $t {
            fn combine(self, rhs: Self) -> Self {
                Self {
                    $(
                        $i: self.$i.combine(rhs.$i),
                    )*
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monoid::{Last, Sum};
    use test_case::test_case;

    #[test_case(Some(5), Some(7) => Some(12))]
    #[test_case(Some(5), None => Some(5))]
    #[test_case(None, Some(7) => Some(7))]
    #[test_case(None, None => None)]
    fn option_combine(left: Option<u32>, right: Option<u32>) -> Option<u32> {
        left.combine(right)
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
        let x = crate::combine! {
            Last::from(53) => None, 42, {let b = None; b},
        };

        assert_eq!(x.0, Some(42));
    }
}
