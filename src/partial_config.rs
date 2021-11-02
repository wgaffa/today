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

#[derive(Debug)]
pub struct Build;

#[derive(Debug)]
pub struct Run;

#[derive(Debug)]
pub struct Select<T, M, A> {
    inner: M,
    val: A,
    _phantom_data: PhantomData<T>,
}

impl<M, A> Select<Build, M, A> {
    pub fn get(&self) -> &M {
        &self.inner
    }
}

impl<M, A> Select<Run, M, A> {
    pub fn get(&self) -> &A {
        &self.val
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

#[derive(Debug)]
pub struct Config<T> {
    pub verbose: Select<T, Sum<i32>, i32>,
    pub out_file: Select<T, Last<PathBuf>, PathBuf>,
}

impl Config<Build> {
    pub fn build(self) -> Config<Run> {
        Config {
            verbose: self.verbose.inner.0.into(),
            out_file: self.out_file.inner.0.unwrap_or_default().into(),
        }
    }
}

impl Semigroup for Config<Build> {
    fn combine(self, rhs: Self) -> Self {
        Self {
            verbose: self.verbose.combine(rhs.verbose),
            out_file: self.out_file.combine(rhs.out_file),
        }
    }
}

impl Monoid for Config<Build> {
    fn empty() -> Self {
        Self {
            verbose: Monoid::empty(),
            out_file: Monoid::empty(),
        }
    }
}

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
