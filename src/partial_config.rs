use crate::{
    semigroup::Semigroup,
    monoid::{Sum, Monoid},
};

use today_derive::*;

#[derive(Default, Semigroup, Monoid, Debug)]
pub struct PartialConfig {
    pub verbose: Option<Sum<u32>>,
}
