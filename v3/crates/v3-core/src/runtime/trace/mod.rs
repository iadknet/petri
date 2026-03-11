//! Runtime trace module split by responsibility:
//! - `domain`: passive sampler data records
//! - `recording`: active recording lifecycle state

pub mod domain;
pub mod recording;
