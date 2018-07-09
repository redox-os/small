use super::std;
use std::borrow::Borrow;

#[derive(Debug, Clone, Copy)]
enum Inner {
    Stack {
        data: [u8;23]
    },
    Heap {
        capacity: usize,
        data: *mut u8
    }
}

/// Inner is safe to send between threads
unsafe impl Send for Inner {}

/// Inner is safe to sync between threads
unsafe impl Sync for Inner {}

/// A string which stores up to 24 bytes on the stack
#[derive(Debug)]
pub struct String {
    len: usize,
    inner: Inner
}
impl String {
    /// Creates a new empty `String`.
    ///
    /// This will create a a string that starts on the stack. If you want to
    /// start on the heap or you know that the length of the string will be
    /// over 23 bytes, then consider using the [`with_capacity`] method so
    /// the string is allocated on the heap.
    ///
    /// [`with_capacity`]: #method.with_capacity
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// let s = String::new();
    /// ```
    #[inline]
    pub fn new() -> String {
        String {
            len: 0,
            inner: Inner::Stack {
                data: [0;23]
            }
        }
    }

    /// Creates a new empty `String` with a particular capacity on the heap.
    ///
    /// `String`s have an internal buffer to hold their data. The capacity is
    /// the length of that buffer, and can be queried with the [`capacity`]
    /// method. This method creates an empty `String`, but one with an initial
    /// buffer that can hold `capacity` bytes. This is useful when you may be
    /// appending a bunch of data to the `String`, reducing the number of
    /// reallocations it needs to do.
    ///
    /// [`capacity`]: #method.capacity
    ///
    /// If the given capacity is `0`, no allocation will occur, and this method
    /// is identical to the [`new`] method.
    ///
    /// [`new`]: #method.new
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// let mut s = String::with_capacity(10);
    ///
    /// // The String contains no chars, even though it has capacity for more
    /// assert_eq!(s.len(), 0);
    ///
    /// // These are all done without reallocating...
    /// let cap = s.capacity();
    /// for i in 0..10 {
    ///     s.push('a');
    /// }
    ///
    /// assert_eq!(s.capacity(), cap);
    ///
    /// // ...but this may make the vector reallocate
    /// s.push('a');
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> String {
        use std::alloc::{ alloc, Layout };
        String {
            len: 0,
            inner: Inner::Heap {
                capacity,
                data: unsafe {
                    alloc(Layout::from_size_align_unchecked(capacity, 32))
                }
            }
        }
    }

    /// Creates a `String` from a `std::string::String`
    ///
    /// This causes no allocations or deallocations, the old string is transformed
    /// into the new string
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// # extern crate small;
    /// let s = "Hello!".into();
    /// let new_s = small::String::from_string(s);
    ///
    /// assert_eq!(new_s.as_str(), "Hello!");
    /// ```
    #[inline]
    pub fn from_string(string: std::string::String) -> String {
        let mut string = string.into_bytes();
        let s = String {
            len: string.len(),
            inner: Inner::Heap {
                capacity: string.capacity(),
                data: string.as_mut_ptr()
            }
        };
        ::std::mem::forget(string);
        s
    }

    /// Shortens this `String` to the specified length.
    ///
    /// If `new_len` is greater than the string's current length, this has no
    /// effect.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the string
    ///
    /// # Panics
    ///
    /// Panics if `new_len` does not lie on a [`char`] boundary.
    ///
    /// [`char`]: ../../std/primitive.char.html
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// let mut s = String::from("hello");
    ///
    /// s.truncate(2);
    ///
    /// assert_eq!("he", s.as_str());
    /// ```
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len() {
            assert!(self.is_char_boundary(new_len));
            self.len = new_len;
        }
    }

    /// The length of the string in bytes
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// The capacity of the string in bytes before reallocation
    #[inline]
    pub fn capacity(&self) -> usize {
        match self.inner {
            Inner::Stack { .. } => {
                23
            },
            Inner::Heap { capacity, .. } => {
                capacity
            }
        }
    }

    /// Returns true if this string is allocated on the heap
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// # extern crate small;
    /// # use small::String;
    /// let stack = String::new();
    /// let heap = String::with_capacity(32);
    ///
    /// assert!(!stack.overflowed());
    /// assert!(heap.overflowed());
    /// ```
    #[inline]
    pub fn overflowed(&self) -> bool {
        match self.inner {
            Inner::Stack { .. } => false,
            Inner::Heap { .. } => true
        }
    }

    /// Removes the last character from the string buffer and returns it.
    ///
    /// Returns [`None`] if this `String` is empty.
    ///
    /// [`None`]: ../../std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// let mut s = String::from("foo");
    ///
    /// assert_eq!(s.pop(), Some('o'));
    /// assert_eq!(s.pop(), Some('o'));
    /// assert_eq!(s.pop(), Some('f'));
    ///
    /// assert_eq!(s.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.chars().rev().next()?;
        let newlen = self.len() - ch.len_utf8();
        self.len = newlen;
        Some(ch)
    }

    /// Removes a [`char`] from this `String` at a byte position and returns it.
    ///
    /// This is an `O(n)` operation, as it requires copying every element in the
    /// buffer.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is larger than or equal to the `String`'s length,
    /// or if it does not lie on a [`char`] boundary.
    ///
    /// [`char`]: ../../std/primitive.char.html
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// let mut s = String::from("foo");
    ///
    /// assert_eq!(s.remove(0), 'f');
    /// assert_eq!(s.remove(1), 'o');
    /// assert_eq!(s.remove(0), 'o');
    /// ```
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        use std::ptr;
        let ch = match self[idx..].chars().next() {
            Some(ch) => ch,
            None => panic!("cannot remove a char from the end of a string"),
        };

        let next = idx + ch.len_utf8();
        let len = self.len();
        unsafe {
            ptr::copy(self.as_ptr().offset(next as isize),
                      self.as_mut_ptr().offset(idx as isize),
                      len - next);
            self.len = len - (next - idx);
        }
        ch
    }

    fn as_ptr(&self) -> *const u8 {
        match &self.inner {
            Inner::Stack { ref data } => {
                data as *const _ as _
            },
            Inner::Heap { capacity: _, ref data } => {
                *data
            }
        }
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        match &mut self.inner {
            Inner::Stack { ref mut data } => {
                data as *mut _ as _
            },
            Inner::Heap { capacity: _, ref data } => {
                *data
            }
        }
    }

    /// Retains only the characters specified by the predicate.
    ///
    /// In other words, remove all characters `c` such that `f(c)` returns `false`.
    /// This method operates in place and preserves the order of the retained
    /// characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// let mut s = String::from("f_o_ob_ar");
    ///
    /// s.retain(|c| c != '_');
    ///
    /// assert_eq!(s.as_str(), "foobar");
    /// ```
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
        where F: FnMut(char) -> bool
    {
        use std::ptr;
        let len = self.len();
        let mut del_bytes = 0;
        let mut idx = 0;

        while idx < len {
            let ch = unsafe {
                self.slice_unchecked(idx, len).chars().next().unwrap()
            };
            let ch_len = ch.len_utf8();

            if !f(ch) {
                del_bytes += ch_len;
            } else if del_bytes > 0 {
                unsafe {
                    ptr::copy(self.as_ptr().offset(idx as isize),
                              self.as_mut_ptr().offset((idx - del_bytes) as isize),
                              ch_len);
                }
            }

            // Point idx to the next char
            idx += ch_len;
        }

        if del_bytes > 0 {
            self.len = len - del_bytes;
        }
    }

    /// The byte representation of the string
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        match self.inner {
            Inner::Stack { ref data } => {
                &data[..self.len()]
            },
            Inner::Heap { data, .. } => {
                unsafe {
                    &::std::slice::from_raw_parts(data, self.len())
                }
            }
        }
    }

    /// The mutable byte representation of the string
    #[inline]
    pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        let len = self.len();
        match &mut self.inner {
            Inner::Stack { ref mut data } => {
                data
            },
            Inner::Heap { capacity: _, data } => {
                ::std::slice::from_raw_parts_mut(*data, len)
            }
        }
    }

    /// As a `str`
    #[inline]
    pub fn as_str(&self) -> &str {
        self
    }

    /// Push a `str` onto the end of the string
    #[inline]
    pub fn push_str(&mut self, item: &str) {
        // we match &mut self.inner so we don't copy the byte array
        match (&mut self.inner, self.len + item.len()) {
            (Inner::Stack { data }, 0...23) => {
                data[self.len..][..item.len()].copy_from_slice(item.as_bytes());
            },
            (Inner::Heap { capacity, ref data }, x) => {
                if x > *capacity {
                    self.grow();
                }
                unsafe {
                    ::std::ptr::copy_nonoverlapping(item.as_ptr(), data.add(self.len), item.len())
                }
            },
            (Inner::Stack { ref data }, _) => {
                use std::alloc::{ alloc, Layout };
                unsafe {
                    let d = alloc(Layout::from_size_align_unchecked(32, 32));
                    ::std::ptr::copy_nonoverlapping(data.as_ptr(), d, self.len);
                    ::std::ptr::copy_nonoverlapping(item.as_ptr(), d.add(self.len), item.len());
                    self.inner = Inner::Heap {
                        capacity: 32,
                        data: d
                    };
                }
            }
        }
        self.len += item.len();
    }

    /// Push a character onto the end of the string
    #[inline]
    pub fn push(&mut self, item: char) {
        let ch_len = item.len_utf8();
        let mut chs = [0; 4];
        item.encode_utf8(&mut chs);
        // we match &mut self.inner so we don't copy the byte array
        match (&mut self.inner, self.len + ch_len) {
            (Inner::Stack { data }, 0...23) => {
                data[self.len..][..ch_len].copy_from_slice(&chs[..ch_len]);
            },
            (Inner::Heap { capacity, ref data }, x) => {
                if x > *capacity {
                    self.grow();
                }
                unsafe {
                    ::std::ptr::copy_nonoverlapping(chs.as_ptr(), data.add(self.len), ch_len)
                }
            },
            (Inner::Stack { ref data }, _) => {
                use std::alloc::{ alloc, Layout };
                unsafe {
                    let d = alloc(Layout::from_size_align_unchecked(32, 32));
                    ::std::ptr::copy_nonoverlapping(data.as_ptr(), d, self.len);
                    ::std::ptr::copy_nonoverlapping(chs.as_ptr(), d.add(self.len), ch_len);
                    self.inner = Inner::Heap {
                        capacity: 32,
                        data: d
                    };
                }
            }
        }
        self.len += ch_len;
    }

    /// Converts a vector of bytes to a `String`
    #[inline]
    pub fn from_utf8(mut vec: std::vec::Vec<u8>) -> Result<String, FromUtf8Error> {
        use std::str;
        match str::from_utf8(&vec) {
            Ok(..) => {
                let (capacity, data, len) = (vec.capacity(), vec.as_mut_ptr(), vec.len());
                Ok(String {
                    len,
                    inner: Inner::Heap {
                        capacity,
                        data
                    }
                })
            },
            Err(e) => {
                Err(FromUtf8Error {
                    bytes: vec,
                    error: e
                })
            }
        }
    }

    /// Converts a vector of bytes to a `String` without checking that the
    /// string contains valid UTF-8.
    ///
    /// See the safe version, [`from_utf8`], for more details.
    ///
    /// [`from_utf8`]: struct.String.html#method.from_utf8
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed
    /// to it are valid UTF-8. If this constraint is violated, it may cause
    /// memory unsafety issues with future users of the `String`, as the rest of
    /// the standard library assumes that `String`s are valid UTF-8.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// // some bytes, in a vector
    /// let sparkle_heart = vec![240, 159, 146, 150];
    ///
    /// let sparkle_heart = unsafe {
    ///     String::from_utf8_unchecked(sparkle_heart)
    /// };
    ///
    /// assert_eq!("💖", sparkle_heart.as_str());
    /// ```
    #[inline]
    pub unsafe fn from_utf8_unchecked(mut vec: std::vec::Vec<u8>) -> String {
        let (capacity, data, len) = (vec.capacity(), vec.as_mut_ptr(), vec.len());
        let s = String {
            len,
            inner: Inner::Heap {
                capacity,
                data
            }
        };
        ::std::mem::forget(vec);
        s
    }

    /// Returns the bytes that were attempted to convert to a `String`.
    ///
    /// This method is carefully constructed to avoid allocation. It will
    /// consume the error, moving out the bytes, so that a copy of the bytes
    /// does not need to be made.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// // some invalid bytes, in a vector
    /// let bytes = vec![0, 159];
    ///
    /// let value = String::from_utf8(bytes);
    ///
    /// assert_eq!(vec![0, 159], value.unwrap_err().into_bytes());
    /// ```
    #[inline]
    pub fn into_bytes(self) -> std::vec::Vec<u8> {
        let v = match &self.inner {
            Inner::Stack { ref data } => {
                let mut v = ::std::vec::Vec::new();
                v.extend_from_slice(data);
                v
            },
            Inner::Heap { ref capacity, ref data } => {
                unsafe {
                    ::std::vec::Vec::from_raw_parts(*data, self.len(), *capacity)
                }
            }
        };
        ::std::mem::forget(self);
        v
    }

    /// Converts a `String` into a mutable string slice.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// let mut s = String::from("foobar");
    /// let s_mut_str = s.as_mut_str();
    ///
    /// s_mut_str.make_ascii_uppercase();
    ///
    /// assert_eq!("FOOBAR", s_mut_str);
    /// ```
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        self
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        use std::alloc::{ dealloc, Layout };
        let len = self.len();
        if let Inner::Heap { ref mut capacity, ref mut data } = &mut self.inner {
            unsafe {
                dealloc(data.add(len), Layout::from_size_align_unchecked(*capacity - len, 32))
            }
            *capacity = len;
        }
    }

    #[inline]
    fn grow(&mut self) {
        use std::alloc::{ handle_alloc_error, realloc, Layout };
        if let Inner::Heap { ref mut capacity, ref mut data } = &mut self.inner {
            unsafe {
                let layout = Layout::from_size_align_unchecked(*capacity, 32);
                let d = realloc(*data, layout, *capacity*2);
                if d.is_null() {
                    handle_alloc_error(layout);
                }
                *data = d;
                *capacity *= 2;
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0);
    }
}

impl AsRef<str> for String {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsRef<[u8]> for String {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Default for String {
    #[inline]
    fn default() -> String {
        String::new()
    }
}

impl Borrow<str> for String {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl ::std::ops::Deref for String {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        match self.inner {
            Inner::Stack { ref data } => {
                unsafe {
                    ::std::str::from_utf8_unchecked(&data[..self.len()])
                }
            }
            _ => {
                unsafe {
                    ::std::str::from_utf8_unchecked(self.as_bytes())
                }
            }
        }
    }
}

impl ::std::ops::DerefMut for String {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        let len = self.len();
        match self.inner {
            Inner::Stack { ref mut data } => {
                unsafe {
                    ::std::str::from_utf8_unchecked_mut(&mut data[..len])
                }
            }
            _ => {
                unsafe {
                    ::std::str::from_utf8_unchecked_mut(self.as_mut_bytes())
                }
            }
        }
    }
}

impl Clone for String {
    #[inline]
    fn clone(&self) -> Self {
        String {
            len: self.len(),
            inner: match self.inner {
                stack @ Inner::Stack { .. } => stack,
                Inner::Heap { capacity, data } => {
                    use std::alloc::{ alloc, Layout };
                    use std::ptr;
                    Inner::Heap {
                        capacity,
                        data: {
                            unsafe {
                                let d = alloc(Layout::from_size_align_unchecked(capacity, 32));
                                ptr::copy_nonoverlapping(data, d, self.len());
                                d
                            }
                        }
                    }
                }
            }
        }
    }
}

impl std::hash::Hash for String {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, hs: &mut H) {
        (**self).hash(hs)
    }
}

impl std::ops::Index<std::ops::Range<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: std::ops::Range<usize>) -> &str {
        &self[..][index]
    }
}

impl std::ops::Index<std::ops::RangeTo<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: std::ops::RangeTo<usize>) -> &str {
        &self[..][index]
    }
}

impl std::ops::Index<std::ops::RangeFrom<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: std::ops::RangeFrom<usize>) -> &str {
        &self[..][index]
    }
}

impl std::ops::Index<std::ops::RangeFull> for String {
    type Output = str;

    #[inline]
    fn index(&self, _index: std::ops::RangeFull) -> &str {
        self
    }
}

impl std::ops::Index<std::ops::RangeInclusive<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: std::ops::RangeInclusive<usize>) -> &str {
        ::std::ops::Index::index(&**self, index)
    }
}

impl std::ops::Index<std::ops::RangeToInclusive<usize>> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: std::ops::RangeToInclusive<usize>) -> &str {
        ::std::ops::Index::index(&**self, index)
    }
}

impl std::ops::IndexMut<std::ops::Range<usize>> for String {
    #[inline]
    fn index_mut(&mut self, index: std::ops::Range<usize>) -> &mut str {
        &mut self[..][index]
    }
}

impl std::ops::IndexMut<std::ops::RangeTo<usize>> for String {
    #[inline]
    fn index_mut(&mut self, index: std::ops::RangeTo<usize>) -> &mut str {
        &mut self[..][index]
    }
}

impl std::ops::IndexMut<std::ops::RangeFrom<usize>> for String {
    #[inline]
    fn index_mut(&mut self, index: std::ops::RangeFrom<usize>) -> &mut str {
        &mut self[..][index]
    }
}

impl std::ops::IndexMut<std::ops::RangeFull> for String {
    #[inline]
    fn index_mut(&mut self, _index: std::ops::RangeFull) -> &mut str {
        self
    }
}

impl std::ops::IndexMut<std::ops::RangeInclusive<usize>> for String {
    #[inline]
    fn index_mut(&mut self, index: std::ops::RangeInclusive<usize>) -> &mut str {
        std::ops::IndexMut::index_mut(&mut **self, index)
    }
}

impl std::ops::IndexMut<std::ops::RangeToInclusive<usize>> for String {
    #[inline]
    fn index_mut(&mut self, index: std::ops::RangeToInclusive<usize>) -> &mut str {
        std::ops::IndexMut::index_mut(&mut **self, index)
    }
}

impl From<std::string::String> for String {
    #[inline]
    fn from(item: std::string::String) -> String {
        use std::mem;
        let mut v = item.into_bytes();
        let (capacity, data, len) = (v.capacity(), v.as_mut_ptr(), v.len());
        mem::forget(v);
        String {
            len,
            inner: Inner::Heap {
                capacity,
                data
            }
        }
    }
}

impl<'a> From<&'a str> for String {
    #[inline]
    fn from(item: &str) -> String {
        String {
            len: item.len(),
            inner: match item.len() {
                0...23 => {
                    Inner::Stack {
                        data: {
                            let mut d = [0u8;23];
                            d[..item.len()].copy_from_slice(item.as_bytes());
                            d
                        }
                    }
                },
                len @ _ => {
                    use std::alloc::{ alloc, Layout };
                    use std::ptr;
                    let capacity = match len.checked_next_power_of_two() {
                        Some(x) => x,
                        None => len
                    };
                    Inner::Heap {
                        capacity,
                        data: {
                            unsafe {
                                let d = alloc(Layout::from_size_align_unchecked(capacity, 32));
                                ptr::copy_nonoverlapping(item.as_ptr(), d, len);
                                d
                            }
                        }
                    }
                }
            }
        }
    }
}

impl From<std::boxed::Box<str>> for String {
    #[inline]
    fn from(item: std::boxed::Box<str>) -> String {
        item.into()
    }
}

#[cfg(feature = "std")]
impl std::net::ToSocketAddrs for String {
    type Iter = std::option::IntoIter<std::net::SocketAddr>;
    #[inline]
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (&self).to_socket_addrs()
    }
}

#[derive(Clone, Copy)]
pub enum ParseError {}

impl std::str::FromStr for String {
    type Err = ParseError;
    #[inline]
    fn from_str(s: &str) -> Result<String, ParseError> {
        Ok(String::from(s))
    }
}

impl<'a> std::ops::Add<&'a str> for String {
    type Output = String;
    #[inline]
    fn add(mut self, other: &'a str) -> String {
        self.push_str(other);
        self
    }
}

impl<'a> std::ops::AddAssign<&'a str> for String {
    #[inline]
    fn add_assign(&mut self, rhs: &'a str) {
        self.push_str(rhs);
    }
}

impl PartialEq for String {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        self.as_str() == rhs.as_str()
    }
}
impl Eq for String { }

impl PartialOrd for String {
    #[inline]
    fn partial_cmp(&self, rhs: &Self) -> Option<::std::cmp::Ordering> {
        self.as_str().partial_cmp(rhs.as_str())
    }
}
impl Ord for String {
    #[inline]
    fn cmp(&self, rhs: &Self) -> ::std::cmp::Ordering {
        self.as_str().cmp(rhs.as_str())
    }
}

impl std::fmt::Write for String {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        Ok(self.push_str(s))
    }

    fn write_char(&mut self, c: char) -> std::fmt::Result {
        Ok(self.push(c))
    }
}

impl std::fmt::Display for String {
    #[inline]
    fn fmt(&self, fm: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        (self as &str).fmt(fm)
    }
}

impl Drop for String {
    #[inline]
    fn drop(&mut self) {
        use std::alloc::{ dealloc, Layout };
        if let Inner::Heap { capacity, data } = self.inner {
            unsafe {
                dealloc(data, Layout::from_size_align_unchecked(capacity, 32))
            }
        }
    }
}

#[derive(Debug)]
pub struct FromUtf8Error {
    bytes: std::vec::Vec<u8>,
    error: std::str::Utf8Error,
}
impl FromUtf8Error {
    /// Returns a slice of [`u8`]s bytes that were attempted to convert to a `String`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// // some invalid bytes, in a vector
    /// let bytes = vec![0, 159];
    ///
    /// let value = String::from_utf8(bytes);
    ///
    /// assert_eq!(&[0, 159], value.unwrap_err().as_bytes());
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..]
    }

    /// Returns the bytes that were attempted to convert to a `String`.
    ///
    /// This method is carefully constructed to avoid allocation. It will
    /// consume the error, moving out the bytes, so that a copy of the bytes
    /// does not need to be made.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// // some invalid bytes, in a vector
    /// let bytes = vec![0, 159];
    ///
    /// let value = String::from_utf8(bytes);
    ///
    /// assert_eq!(vec![0, 159], value.unwrap_err().into_bytes());
    /// ```
    #[inline]
    pub fn into_bytes(self) -> std::vec::Vec<u8> {
        self.bytes
    }

    /// Fetch a `Utf8Error` to get more details about the conversion failure.
    ///
    /// The [`Utf8Error`] type provided by [`std::str`] represents an error that may
    /// occur when converting a slice of [`u8`]s to a [`&str`]. In this sense, it's
    /// an analogue to `FromUtf8Error`. See its documentation for more details
    /// on using it.
    ///
    /// [`Utf8Error`]: ../../std/str/struct.Utf8Error.html
    /// [`std::str`]: ../../std/str/index.html
    /// [`u8`]: ../../std/primitive.u8.html
    /// [`&str`]: ../../std/primitive.str.html
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # extern crate small;
    /// use small::String;
    /// // some invalid bytes, in a vector
    /// let bytes = vec![0, 159];
    ///
    /// let error = String::from_utf8(bytes).unwrap_err().utf8_error();
    ///
    /// // the first byte is invalid here
    /// assert_eq!(1, error.valid_up_to());
    /// ```
    #[inline]
    pub fn utf8_error(&self) -> std::str::Utf8Error {
        self.error
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn size() {
        assert_eq!(::std::mem::size_of::<String>(), 32)
    }
    #[test]
    fn str_under_24() {
        assert_eq!("hello", String::from("hello").as_str())
    }
    #[test]
    fn str_over_24() {
        assert_eq!("abcdefghijklmnopqrstuvwxyz",
                   String::from("abcdefghijklmnopqrstuvwxyz").as_str())
    }
    #[test]
    fn sort() {
        let mut v = vec![
            String::from("c"), String::from("b"), String::from("a"),
            String::from("d"), String::from("e")
        ];
        v.sort();
        assert_eq!(v,
                   [String::from("a"), String::from("b"),
                    String::from("c"), String::from("d"),
                    String::from("e")]);
    }
    #[test]
    fn clone_stack() {
        let a = super::String::from("hello");
        assert_eq!(a.clone(), a);
    }
    #[test]
    fn clone_heap() {
        let a = super::String::from("abcdefghijklmnopqrstuvwxyz");
        assert_eq!(a.clone(), a);
    }
    #[test]
    fn push_stack() {
        let mut a = super::String::from("hell");
        a.push('o');
        assert_eq!("hello", a.as_str())
    }
    #[test]
    fn push_stack_to_heap() {
        let mut a = super::String::from("abcdefghijklmnopqrstuvw");
        a.push('x');
        assert_eq!("abcdefghijklmnopqrstuvwx", a.as_str())
    }
    #[test]
    fn push_heap() {
        let mut a = super::String::from("abcdefghijklmnopqrstuvwxy");
        a.push('z');
        assert_eq!("abcdefghijklmnopqrstuvwxyz", a.as_str())
    }
    #[test]
    fn push_str_stack() {
        let mut a = super::String::from("h");
        a.push_str("ello");
        assert_eq!("hello", a.as_str())
    }
    #[test]
    fn push_str_heap() {
        let mut a = super::String::from("abcdefghijklmnopqrstuvwxyz");
        a.push_str(" hello");
        assert_eq!("abcdefghijklmnopqrstuvwxyz hello", a.as_str())
    }
    #[test]
    fn push_str_stack_to_heap() {
        let mut a = super::String::from("abcdefghijkl");
        a.push_str("mnopqrstuvwxyz hello");
        assert_eq!("abcdefghijklmnopqrstuvwxyz hello", a.as_str())
    }
    #[test]
    fn grow_heap() {
        let mut a = super::String::from("abcdefghijklmnopqrstuvwxyz");
        a.push_str(" hello thing");
        assert_eq!("abcdefghijklmnopqrstuvwxyz hello thing", a.as_str())
    }
    #[test]
    fn into_bytes_stack() {
        let a = super::String::from("hello");
        assert_eq!(a.into_bytes(), vec![104, 101, 108, 108, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }
    #[test]
    fn into_bytes_heap() {
        let a = super::String::from("abcdefghijklmnopqrstuvwxyz");
        assert_eq!(a.into_bytes(), vec![97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122])
    }
    #[test]
    fn clear_the_string() {
        let mut a = super::String::from("abcdefghijklmnopqrstuvwxyz");
        let original_capacity = a.capacity();
        a.clear();
        assert_eq!(a.as_str(), "");
        assert_eq!(a.capacity(), original_capacity);
    }
}
