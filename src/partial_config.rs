use std::marker::PhantomData;
use std::path::PathBuf;

use crate::{
    semigroup::Semigroup,
    monoid::{Last, Sum, Monoid},
};

use today_derive::*;

#[derive(Default, Semigroup, Monoid, Debug)]
pub struct PartialConfig {
    pub verbose: Option<Sum<u32>>,
}

#[derive(Debug, Default)]
pub struct Build;

#[derive(Debug)]
pub struct Run;

#[derive(Debug, Default)]
pub struct Select<T, M, A> {
    inner: M,
    val: A,
    _phantom_data: PhantomData<T>,
}

impl<M, A> Select<Build, M, A> {
    pub fn get(self) -> M {
        self.inner
    }
}

impl<M, A> Select<Run, M, A> {
    pub fn get(self) -> A {
        self.val
    }
}

impl<M, A: Default> From<M> for Select<Build, M, A> {
    fn from(value: M) -> Self {
        Self {
            inner: value,
            val: Default::default(),
            _phantom_data: PhantomData,
        }
    }
}

impl<M: Monoid, A> From<A> for Select<Run, M, A> {
    fn from(val: A) -> Self {
        Self {
            inner: Monoid::empty(),
            val,
            _phantom_data: PhantomData,
        }
    }
}

impl<M: Semigroup, A: Default> Semigroup for Select<Build, M, A> {
    fn combine(self, rhs: Self) -> Self {
        Self {
            inner: self.inner.combine(rhs.inner),
            val: Default::default(),
            _phantom_data: PhantomData,
        }
    }
}

impl<M: Monoid, A: Default> Monoid for Select<Build, M, A> {
    fn empty() -> Self {
        Self {
            inner: Monoid::empty(),
            val: Default::default(),
            _phantom_data: PhantomData,
        }
    }
}

#[macro_export]
macro_rules! config {
    ($($der:meta),+ $name:ident { $(,)? }) => {
        $(#[$der])*
        pub struct $name<T> {
            _phantom_data: PhantomData<T>,
        }
    };
    ($($der:meta),+ $name:ident { $($i:ident : $m:ty => $t:ty),* $(,)? }) => {
        $(#[$der])*
        pub struct $name<T> {
            $(
                pub $i: Select<T, $m, $t>,
            )*
        }
    };
    ($($tail:tt)*) => {
        $crate::config!(
            derive(Debug)
            $($tail)*
        );
    };
}

config!(
    Config {
        verbose: Sum<i32> => i32,
        out_file: Last<PathBuf> => PathBuf,
    }
);

#[macro_export]
macro_rules! config_builder {
    ($t:ident { $($field:ident => $e:expr),* $(,)? }) => {
        impl $t<Build> {
            pub fn build(self) -> $t<Run> {
                $t {
                    $(
                        $field: {
                            let tmp = $e;
                            tmp(self.$field.inner)
                        },
                    )*
                }
            }
        }
    };
}

pub trait Builder {
    type Item;
    fn build(self) -> Self::Item;
}

impl Builder for Config<Build> {
    type Item = Config<Run>;
    fn build(self) -> Self::Item {
        Config {
            verbose: self.verbose.inner.0.into(),
            out_file: self.out_file.inner.0.unwrap_or_default().into(),
        }
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

semigroup_default!(Config<Build>: verbose, out_file);
monoid_default!(Config<Build>: verbose, out_file);

impl<T> Semigroup for PhantomData<T> {
    fn combine(self, _rhs: Self) -> Self {
        self
    }
}

impl<T> Monoid for PhantomData<T> {
    fn empty() -> Self {
        Self
    }
}
