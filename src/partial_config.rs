use std::{marker::PhantomData, path::PathBuf};

use crate::{
    monoid::{Last, Monoid, Sum},
    semigroup::Semigroup,
};

use today_derive::*;

#[derive(Default, Semigroup, Monoid, Debug)]
pub struct PartialConfig {
    pub verbose: Option<Sum<u32>>,
}

#[derive(Debug)]
pub struct Build;
pub struct Run;

impl Semigroup for Build {
    fn combine(self, rhs: Self) -> Self {
        self
    }
}

pub trait Runner<F, A> {}

#[derive(Debug, Semigroup)]
pub struct Identity<T> {
    pub value: T,
}

#[derive(Debug, Semigroup)]
pub struct Select<F, A>
where
    F: Semigroup,
{
    runner: F,
    accesser: A,
}

#[derive(Debug, Semigroup)]
pub struct Config<T> {
    debug: Select<Identity<i32>, i32>,
    verbose: Select<Sum<i32>, i32>,
    _phantom_data: PhantomData<T>,
}

impl<T> Semigroup for PhantomData<T> {
    fn combine(self, _: Self) -> Self {
        self
    }
}
