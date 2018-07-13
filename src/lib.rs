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

mod alloc {
    use std::mem;

    /// These are internal rust allocation functions. They're not supposed to be
    /// exposed by the compiler, but they do exist as symbols so we can use them
    /// to control our allocations without having to go through a [`Box`] or a
    /// [`Vec`] as you would otherwise.
    ///
    /// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
    /// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
    extern "Rust" {
        fn __rust_alloc(size: usize, align: usize) -> *mut u8;
        fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize);
        fn __rust_realloc(ptr: *mut u8,
                          old_size: usize,
                          align: usize,
                          new_size: usize) -> *mut u8;
    }

    /// Allocate `count` number of `T` on the heap.
    ///
    /// Returns
    /// -------
    /// A null pointer on failure, a valid pointer on success
    #[inline]
    pub fn alloc<T>(count: usize) -> *mut T {
        unsafe {
            __rust_alloc(mem::size_of::<T>()*count, mem::align_of::<T>()) as _
        }
    }

    /// Deallocate `ptr` of count `count`
    #[inline]
    pub unsafe fn dealloc<T>(ptr: *mut T, count: usize) {
        __rust_dealloc(ptr as _,
                       mem::size_of::<T>()*count,
                       mem::align_of::<T>());
    }

    /// Reallocate `ptr` with count `old_count` to be of size `new_count`
    #[inline]
    pub unsafe fn realloc<T>(ptr: *mut T, old_count: usize, count: usize) -> *mut T {
        __rust_realloc(ptr as _,
                       mem::size_of::<T>()*old_count,
                       mem::align_of::<T>(),
                       mem::size_of::<T>()*count) as _
    }
}
