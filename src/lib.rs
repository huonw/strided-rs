#![feature(macro_rules)]

//! Strided slices.
//!
//! This library provides two types `Strided` and `MutStrided` as
//! generalised forms of `&[T]` and `&mut [T]` respectively, where the
//! elements are regularly spaced in memory, but not necessarily
//! immediately adjacently.
//!
//! For example, given an underlying array `[1, 2, 3, 4, 5]`, the
//! elements `[1, 3, 5]` are a strided slice with stride 2, and
//! `[1, 4]` has stride 3. Any slice can be regarded as a strided slice
//! with stride 1.
//!
//! This provides functionality through which one can safely and
//! efficiently manipulate every `n`th element of a slice (even a
//! mutable one) as close as possible to it being a conventional
//! slice. This releases one from worries about stride bookkeeping,
//! aliasing of `&mut` or any `unsafe` code.
//!
//! # Quick start
//!
//! The work-horse function is `.substrides(n)`, which returns an
//! iterator across a series of `n` new strided slices, each of which
//! points to every `n`th element, and each of which starts at the
//! next successive offset. For example, the following has `n = 3`.
//!
//! ```rust
//! use strided::MutStrided;
//!
//! let mut v = [1u8, 2, 3, 4, 5];
//! let mut all = MutStrided::new(&mut v);
//!
//! let mut substrides = all.substrides(3);
//!
//! let a = substrides.next().unwrap();
//! let b = substrides.next().unwrap();
//! let c = substrides.next().unwrap();
//! assert!(substrides.next().is_none()); // there was exactly 3.
//!
//! assert_eq!(a, MutStrided::new(&mut [1, 4]));
//! assert_eq!(b, MutStrided::new(&mut [2, 5]));
//! assert_eq!(c, MutStrided::new(&mut [3]));
//! ```
//!
//! The common case of `n = 2` has an abbreviation `substrides2`,
//! which takes the liberty of returns a tuple rather than an iterator
//! to make direct destructuring work. Continuing with the values
//! above, `left` and `right` point to alternate elements, starting at
//! index `0` and `1` of their parent slice respectively.
//!
//! ```rust
//! # use strided::MutStrided;
//! # let mut v = [1u8, 2, 3, 4, 5];
//! # let mut all = MutStrided::new(&mut v);
//! let (left, right) = all.reborrow().substrides2();
//!
//! assert_eq!(left, MutStrided::new(&mut [1, 3, 5]));
//! assert_eq!(right, MutStrided::new(&mut [2, 4]));
//! ```
//!
//! A lot of the conventional slice functionality is available, such
//! as indexing (both sugary and non-panicking), iterators and
//! slicing.
//!
//! ```rust
//! # use strided::MutStrided;
//! # let mut v = [1u8, 2, 3, 4, 5];
//! # let mut all = MutStrided::new(&mut v);
//! let (mut left, right) = all.reborrow().substrides2();
//! assert_eq!(left[2], 5);
//! assert!(right.get(10).is_none()); // out of bounds
//!
//! left[2] += 10;
//! match left.get_mut(0) {
//!     Some(val) => *val -= 3,
//!     None => {}
//! }
//!
//! assert_eq!(right.iter().fold(0, |sum, a| sum + *a), 2 + 4);
//! for val in left.iter_mut() {
//!     *val /= 2
//! }
//! ```
//!
//! Many of the methods of `MutStrided` take `self` by-value and so
//! consume ownership, this makes the `reborrow` method one of the
//! most important. It converts a `&'b mut MutStrided<'a, T>` to a
//! `MutStrided<'b, T>`, that is, allows temporarily viewing a strided
//! slices as one with a shorter lifetime. The temporary can then be
//! used with the consuming methods, and the parent slice can still be
//! used after that borrow has finished. For example, all of the
//! splitting and slicing methods on `MutStrided` consume ownership,
//! and so `reborrow` is necessary there to continue using, in this
//! case, `left`.
//!
//! ```rust
//! # use strided::MutStrided;
//! # let mut v = [1u8, 2, 3, 4, 5];
//! # let mut all = MutStrided::new(&mut v);
//! let (mut left, right) = all.reborrow().substrides2();
//! assert_eq!(left.reborrow().slice(1, 3), MutStrided::new(&mut [3, 5]));
//! assert_eq!(left.reborrow().slice_from(2), MutStrided::new(&mut [5]));
//! assert_eq!(left.reborrow().slice_to(2), MutStrided::new(&mut [1, 3]));
//!
//! assert_eq!(right.split_at(1),
//!            (MutStrided::new(&mut [2]), MutStrided::new(&mut [4])));
//! ```
//!
//! These contortions are necessary to ensure that `&mut`s cannot
//! alias, while still maintaining flexibility: leaving elements with
//! the maximum possible lifetime (i.e. that of the non-strided slices
//! which they lie in). Theoretically they are necessary with
//! `&mut []` too, but the compiler inserts implicit reborrows and so
//! one rarely needs to do them manually.
//!
//! The shared `Strided` is equivalent to `&[]` and only handles `&`
//! references, making ownership transfer and `reborrow` unnecessary,
//! so all its methods act identically to those on `&[]`.

#[cfg(test)] extern crate test;

pub use base::{Items, MutItems};

pub use mut_::Strided as MutStrided;
pub use mut_::Substrides as MutSubstrides;

pub use imm::Strided as Strided;
pub use imm::Substrides as Substrides;

macro_rules! eq {
    ($stride: expr, $expected: expr) => {
        eq!($stride, $expected, iter)
    };

    ($stride: expr, $expected: expr, $method: ident) => {{
        let e: &[_] = $expected;
        let mut _stride = $stride;
        assert_eq!(_stride.len(), e.len());
        let mut iter = _stride.$method();
        assert_eq!(iter.size_hint(),(e.len(), Some(e.len())));
        let vals = iter.by_ref().map(|s| *s).collect::<Vec<_>>();
        if vals.as_slice() != e {
            panic!("mismatched: {}, {}", vals, e);
        }
        assert_eq!(iter.size_hint(),(0, Some(0)));
    }}
}

mod base;
mod mut_;
mod imm;


#[cfg(test)]
mod bench {
    use super::Strided;
    use test::Bencher as B;
    use test;

    const N: uint = 100;

    #[bench]
    fn iter_slice(b: &mut B) {
        let v = Vec::from_fn(N, |i| i);
        b.iter(|| {
            test::black_box(&v);
            for e in v.iter() { test::black_box(e) }
        })
    }

    #[bench]
    fn iter_step_1(b: &mut B) {
        let v = Vec::from_fn(N, |i| i);
        let s = Strided::new(&*v);
        b.iter(|| {
            test::black_box(&s);
            for e in s.iter() { test::black_box(e) }
        })
    }

    #[bench]
    fn iter_step_13(b: &mut B) {
        let v = Vec::from_fn(N * 13, |i| i);
        let s = Strided::new(&*v);
        let s = s.substrides(13).next().unwrap();
        b.iter(|| {
            test::black_box(&s);
            for e in s.iter() { test::black_box(e) }
        })
    }
}
