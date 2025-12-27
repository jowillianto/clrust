use crate::log::Context;

use super::prelude::Filter;

#[derive(Debug, Clone, Copy, Default)]
pub struct NoFilter;

impl Filter for NoFilter {
    fn allow(&self, _: &Context<'_>) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    Eq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone, Copy)]
pub struct LevelFilter {
    op: FilterOp,
    level: u8,
}

impl LevelFilter {
    pub fn equal_to(level: u8) -> Self {
        Self {
            op: FilterOp::Eq,
            level,
        }
    }

    pub fn less_than(level: u8) -> Self {
        Self {
            op: FilterOp::Lt,
            level,
        }
    }

    pub fn less_than_or_equal_to(level: u8) -> Self {
        Self {
            op: FilterOp::Lte,
            level,
        }
    }

    pub fn greater_than(level: u8) -> Self {
        Self {
            op: FilterOp::Gt,
            level,
        }
    }

    pub fn greater_than_or_equal_to(level: u8) -> Self {
        Self {
            op: FilterOp::Gte,
            level,
        }
    }
}

impl Filter for LevelFilter {
    fn allow(&self, ctx: &Context<'_>) -> bool {
        match self.op {
            FilterOp::Eq => ctx.level.value == self.level,
            FilterOp::Lt => ctx.level.value < self.level,
            FilterOp::Lte => ctx.level.value <= self.level,
            FilterOp::Gt => ctx.level.value > self.level,
            FilterOp::Gte => ctx.level.value >= self.level,
        }
    }
}
