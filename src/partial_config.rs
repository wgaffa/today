use std::{
    marker::PhantomData,
    path::PathBuf,
};

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

#[derive(Debug)]
enum Selection<M, A> {
    Build(M),
    Run(A),
}

#[derive(Debug)]
pub struct Select<T, M, A> {
    inner: Selection<M, A>,
    _phantom_data: PhantomData<T>,
}

impl<T, M: Monoid, A> Default for Select<T, M, A> {
    fn default() -> Self {
        Self {
            inner: Selection::Build(Monoid::empty()),
            _phantom_data: PhantomData,
        }
    }
}

impl<M, A> Select<Build, M, A> {
    pub fn get(self) -> M {
        if let Selection::Build(x) = self.inner {
            x
        } else {
            panic!("Select in wrong state")
        }
    }
}

impl<M, A> Select<Run, M, A> {
    pub fn get(self) -> A {
        if let Selection::Run(x) = self.inner {
            x
        } else {
            panic!("Select in wrong state")
        }
    }
}

impl<M, A> From<M> for Select<Build, M, A> {
    fn from(value: M) -> Self {
        Self {
            inner: Selection::Build(value),
            _phantom_data: PhantomData,
        }
    }
}

impl<M: Monoid, A> From<A> for Select<Run, M, A> {
    fn from(val: A) -> Self {
        Self {
            inner: Selection::Run(val),
            _phantom_data: PhantomData,
        }
    }
}

impl<M: Semigroup, A> Semigroup for Select<Build, M, A> {
    fn combine(self, rhs: Self) -> Self {
        Self {
            inner: match (self.inner, rhs.inner) {
                (Selection::Build(left), Selection::Build(right)) =>
                    Selection::Build(left.combine(right)),
                _ => panic!("Select Build was in a wrong state to combine"),
            },
            _phantom_data: PhantomData,
        }
    }
}

impl<M: Monoid, A> Monoid for Select<Build, M, A> {
    fn empty() -> Self {
        Self {
            inner: Selection::Build(Monoid::empty()),
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
            verbose: self.verbose.get().0.into(),
            out_file: self.out_file.get().0.unwrap_or_default().into(),
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
