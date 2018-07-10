//!
//! # Small
//!
//! Small is a crate devoted to "small" types - those that store additional
//! information on the stack while they can before expanding to the heap.
//! This leads to better performance for lower amounts of storage as you do not
//! have to wait for allocation between insertions and deletions.
//!
//! Currently, only the [`String`] type has been implemented. This is based
//! upon similar implementations of `String` in various libc++ libraries.
//! `String` stores 23 bytes of data on the stack, however once it begins to
//! use the heap to store data, it never returns to using the stack.
//!
//! [`String`]: string/struct.String.html

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
#[cfg_attr(test, macro_use)]
extern crate alloc;

#[cfg(not(feature = "std"))]
mod std {
    pub use alloc::{boxed, string, vec};
    pub use core::*;
}

pub mod string;
pub use string::String;
