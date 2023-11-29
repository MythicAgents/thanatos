#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

//! Implementation of the `pwd` command.

#[cfg(not(feature = "load"))]
pub mod base;

#[cfg(feature = "load")]
pub mod load;
