//! Animated inderminate loading spinners.
//!
//! Code from the [iced loading spinners example](https://github.com/iced-rs/iced/tree/master/examples/loading_spinners)
//! but using [`iced::Animation`]

pub mod circular;
pub mod linear;

pub use circular::{Circular, circular};
pub use linear::{Linear, linear};
