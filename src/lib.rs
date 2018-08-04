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
    pub use core::*;
    pub use alloc::{alloc, boxed, string, vec};
}

pub mod string;
pub use string::String;

mod allocate {
    use std::{
        alloc::{Layout, alloc as std_alloc, dealloc as std_dealloc, realloc as std_realloc},
        mem,
    };

    /// Allocate `count` number of `T` on the heap.
    ///
    /// Returns
    /// -------
    /// A null pointer on failure, a valid pointer on success
    #[inline(always)]
    pub fn alloc<T>(count: usize) -> *mut T {
        unsafe {
            std_alloc(Layout::from_size_align_unchecked(mem::size_of::<T>() * count, mem::align_of::<T>())) as _
        }
    }

    /// Deallocate `ptr` of count `count`
    #[inline(always)]
    pub unsafe fn dealloc<T>(ptr: *mut T, count: usize) {
        std_dealloc(ptr as *mut u8, Layout::from_size_align_unchecked(mem::size_of::<T>() * count, mem::align_of::<T>()));
    }

    /// Reallocate `ptr` with count `old_count` to be of size `new_count`
    #[inline(always)]
    pub unsafe fn realloc<T>(ptr: *mut T, old_count: usize, count: usize) -> *mut T {
        std_realloc(ptr as *mut u8, Layout::from_size_align_unchecked(mem::size_of::<T>() * old_count, mem::align_of::<T>()), mem::size_of::<T>() * count) as _
    }
}
