//! Testing utilities

#[cfg(loom)]
pub(crate) mod loom_oneshot;

#[cfg(test)]
mod basic_scheduler;
