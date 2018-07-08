#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
#[cfg_attr(test, macro_use)]
extern crate alloc;

#[cfg(not(feature = "std"))]
mod std {
    pub use core::*;
    pub use alloc::{alloc, string, vec};
}

pub mod string;
pub use string::String;

extern crate smallvec;
pub use smallvec::SmallVec as Vec;
