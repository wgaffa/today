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
        match self {
            Some(left) => match rhs {
                Some(right) => Some(left.combine(right)),
                None => Some(left),
            },
            None => match rhs {
                Some(right) => Some(right),
                None => None,
            },
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
