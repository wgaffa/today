use std::marker::PhantomData;

use crate::{monoid::Monoid, semigroup::Semigroup};

use today_derive::Semigroup;

#[derive(Debug, Default, Clone, Copy)]
pub struct Build;

#[derive(Debug, Clone, Copy)]
pub struct Run;

#[derive(Debug, Clone, Copy)]
enum Selection<M, A> {
    Build(M),
    Run(A),
}

#[derive(Debug, Clone, Copy)]
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
    pub fn value(&self) -> &A {
        if let Selection::Run(ref x) = self.inner {
            x
        } else {
            panic!("Select in wrong state")
        }
    }

    pub fn get(self) -> A {
        if let Selection::Run(x) = self.inner {
            x
        } else {
            panic!("Select in wrong state")
        }
    }
}

impl<M, A> Select<Run, M, A>
where
    A: Default,
{
    pub fn take(&mut self) -> A {
        if let Selection::Run(ref mut x) = self.inner {
            std::mem::take(x)
        } else {
            panic!("Select in wrong state")
        }
    }
}

impl<M, A> From<Option<A>> for Select<Build, M, A>
where
    M: Monoid,
    Option<A>: Into<M>,
{
    fn from(value: Option<A>) -> Self {
        Self {
            inner: Selection::Build(value.into()),
            _phantom_data: PhantomData,
        }
    }
}

impl<M, A> From<A> for Select<Build, M, A>
where
    M: Monoid,
    A: Into<M>,
{
    fn from(value: A) -> Self {
        Self {
            inner: Selection::Build(value.into()),
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

impl<M, A> From<Select<Run, M, A>> for Select<Build, M, A>
where
    M: Monoid,
    A: Into<M>,
{
    fn from(value: Select<Run, M, A>) -> Self {
        let value = match value.inner {
            Selection::Build(_) => panic!("Select in wrong state"),
            Selection::Run(x) => x,
        };

        Self {
            inner: Selection::Build(value.into()),
            _phantom_data: PhantomData,
        }
    }
}

impl<M, A> From<Select<Build, M, A>> for Select<Run, M, A>
where
    M: Monoid + Into<A>,
{
    fn from(value: Select<Build, M, A>) -> Self {
        let value = match value.inner {
            Selection::Run(_) => panic!("Select in wrong state"),
            Selection::Build(x) => x,
        };

        Self {
            inner: Selection::Run(value.into()),
            _phantom_data: PhantomData,
        }
    }
}

impl<M: Semigroup, A> Semigroup for Select<Build, M, A> {
    fn combine(self, rhs: Self) -> Self {
        Self {
            inner: match (self.inner, rhs.inner) {
                (Selection::Build(left), Selection::Build(right)) => {
                    Selection::Build(left.combine(right))
                }
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

impl Semigroup for Build {
    fn combine(self, _rhs: Self) -> Self {
        self
    }
}

#[derive(Debug, Semigroup)]
pub struct Identity<T> {
    pub value: T,
}
