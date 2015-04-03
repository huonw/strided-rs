//! Strided slices.
//!
//! This library provides two types `Stride` and `MutStride` as
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
//! The work-horse functions are `.substrides(n)` and
//! `.substrides_mut(n)`, which return an iterator across a series of
//! `n` new strided slices (shared and mutable, respectively), each of
//! which points to every `n`th element, and each of which starts at
//! the next successive offset. For example, the following has
//! `n = 3`.
//!
//! ```rust
//! use strided::MutStride;
//!
//! let mut v = [1u8, 2, 3, 4, 5];
//! let mut all = MutStride::new(&mut v);
//!
//! let mut substrides = all.substrides_mut(3);
//!
//! let a = substrides.next().unwrap();
//! let b = substrides.next().unwrap();
//! let c = substrides.next().unwrap();
//! assert!(substrides.next().is_none()); // there was exactly 3.
//!
//! assert_eq!(a, MutStride::new(&mut [1, 4]));
//! assert_eq!(b, MutStride::new(&mut [2, 5]));
//! assert_eq!(c, MutStride::new(&mut [3]));
//! ```
//!
//! The common case of `n = 2` has an abbreviation `substrides2`
//! (resp. `substrides2_mut`), which takes the liberty of returns a
//! tuple rather than an iterator to make direct destructuring
//! work. Continuing with the values above, `left` and `right` point
//! to alternate elements, starting at index `0` and `1` of their
//! parent slice respectively.
//!
//! ```rust
//! # use strided::MutStride;
//! # let mut v = [1u8, 2, 3, 4, 5];
//! # let mut all = MutStride::new(&mut v);
//! let (left, right) = all.substrides2_mut();
//!
//! assert_eq!(left, MutStride::new(&mut [1, 3, 5]));
//! assert_eq!(right, MutStride::new(&mut [2, 4]));
//! ```
//!
//! A lot of the conventional slice functionality is available, such
//! as indexing, iterators and slicing.
//!
//! ```rust
//! # use strided::MutStride;
//! # let mut v = [1u8, 2, 3, 4, 5];
//! # let mut all = MutStride::new(&mut v);
//! let (mut left, right) = all.substrides2_mut();
//! assert_eq!(left[2], 5);
//! assert!(right.get(10).is_none()); // out of bounds
//!
//! left[2] += 10;
//! match left.get_mut(0) {
//!     Some(val) => *val -= 1,
//!     None => {}
//! }
//!
//! assert_eq!(right.iter().fold(0, |sum, a| sum + *a), 2 + 4);
//! for val in left.iter_mut() {
//!     *val /= 2
//! }
//! ```
//!
//! ## Ownership and `reborrow`
//!
//! `MutStride` has a method `reborrow` which has signature
//!
//! ```rust,ignore
//! impl<'a, T> MutStride<'a, T> {
//!     pub fn reborrow<'b>(&'b mut self) -> MutStride<'b, T> { ... }
//! }
//! ```
//!
//! That is, it allows temporarily viewing a strided slices as one
//! with a shorter lifetime. This method is key because many of the
//! methods on `MutStride` take `self` by-value and so consume
//! ownership... which is rather unfortunate if one wants to use a
//! strided slice multiple times.
//!
//! The temporary returned by `reborrow` can be used with the
//! consuming methods, which allows the parent slice to continuing
//! being used after that temporary has disappeared. For example, all
//! of the splitting and slicing methods on `MutStride` consume
//! ownership, and so `reborrow` is necessary there to continue using,
//! in this case, `left`.
//!
//! ```rust
//! # use strided::MutStride;
//! # let mut v = [1u8, 2, 3, 4, 5];
//! # let mut all = MutStride::new(&mut v);
//! let (mut left, right) = all.substrides2_mut();
//! assert_eq!(left.reborrow().slice_mut(1, 3), MutStride::new(&mut [3, 5]));
//! assert_eq!(left.reborrow().slice_from_mut(2), MutStride::new(&mut [5]));
//! assert_eq!(left.reborrow().slice_to_mut(2), MutStride::new(&mut [1, 3]));
//!
//! // no reborrow:
//! assert_eq!(right.split_at_mut(1),
//!            (MutStride::new(&mut [2]), MutStride::new(&mut [4])));
//! // println!("{}", right); // error: use of moved value `right`.
//! ```
//!
//! These contortions are necessary to ensure that `&mut`s cannot
//! alias, while still maintaining flexibility: leaving elements with
//! the maximum possible lifetime (i.e. that of the non-strided slices
//! which they lie in). Theoretically they are necessary with
//! `&mut []` too, but the compiler inserts implicit reborrows and so
//! one rarely needs to do them manually.
//!
//! In practice, one should only need to insert `reborrow`s if the
//! compiler complains about the use of a moved value.
//!
//! The shared `Stride` is equivalent to `&[]` and only handles `&`
//! references, making ownership transfer and `reborrow` unnecessary,
//! so all its methods act identically to those on `&[]`.
//!
//! # Example
//!
//! The [fast Fourier transform
//! (FFT)](https://en.wikipedia.org/wiki/Fast_Fourier_transform) is a
//! signal processing algorithm that performs a discrete Fourier
//! transform (DFT) of length `n` in `O(n log n)` time. A DFT breaks a
//! waveform into the sum of sines and cosines, and is an important
//! part of many other algorithms due to certain nice properties of
//! the Fourier transform.
//!
//! The first FFT algorithm was the [Cooley-Tukey
//! algorithm](https://en.wikipedia.org/wiki/Cooley-Tukey_FFT_algorithm). The
//! decimation-in-time variant works by computing the FFT of
//! equal-length subarrays of equally spaced elements and then
//! combining these together into the desired result. This sort of
//! spacing is exactly the striding provided by this library, and
//! hence this library can be used to create an FFT algorithm in a
//! very natural way.
//!
//! Below is an implementation of the radix-2 case, that is, when the
//! length `n` is a power of two. In this case, only two strided
//! subarrays are necessary: exactly the alternating ones provided by
//! `substrides2`. Note the use of `reborrow` to allow `start` and
//! `end` to be used for the recursive `fft` calls and then again
//! later in the loop.
//!
//! ```rust
//! # #![allow(unstable)]
//! extern crate strided;
//! extern crate num; // https://github.com/rust-lang/num
//! use std::num::Int;
//! use std::f64;
//! use num::complex::{Complex, Complex64};
//! use strided::{MutStride, Stride};
//!
//! /// Writes the forward DFT of `input` to `output`.
//! fn fft(input: Stride<Complex64>, mut output: MutStride<Complex64>) {
//!     // check it's a power of two.
//!     assert!(input.len() == output.len() && input.len().count_ones() == 1);
//!
//!     // base case: the DFT of a single element is itself.
//!     if input.len() == 1 {
//!         output[0] = input[0];
//!         return
//!     }
//!
//!     // split the input into two arrays of alternating elements ("decimate in time")
//!     let (evens, odds) = input.substrides2();
//!     // break the output into two halves (front and back, not alternating)
//!     let (mut start, mut end) = output.split_at_mut(input.len() / 2);
//!
//!     // recursively perform two FFTs on alternating elements of the input, writing the
//!     // results into the first and second half of the output array respectively.
//!     fft(evens, start.reborrow());
//!     fft(odds, end.reborrow());
//!
//!     // exp(-2πi/N)
//!     let twiddle = Complex::from_polar(&1.0, &(-2.0 * f64::consts::PI / input.len() as f64));
//!
//!     let mut factor = Complex::new(1., 0.);
//!
//!     // combine the subFFTs with the relations:
//!     //   X_k       = E_k + exp(-2πki/N) * O_k
//!     //   X_{k+N/2} = E_k - exp(-2πki/N) * O_k
//!     for (even, odd) in start.iter_mut().zip(end.iter_mut()) {
//!         let twiddled = factor * *odd;
//!         let e = *even;
//!
//!         *even = e + twiddled;
//!         *odd = e - twiddled;
//!         factor = factor * twiddle;
//!     }
//! }
//!
//! fn main() {
//!     let a = [Complex::new(2., 0.), Complex::new(1., 0.),
//!              Complex::new(2., 0.), Complex::new(1., 0.)];
//!     let mut b = [Complex::new(0., 0.); 4];
//!
//!     fft(Stride::new(&a), MutStride::new(&mut b));
//!     println!("forward: {:?} -> {:?}", a.as_slice(), b.as_slice());
//! }
//! ```
//!
//! The above definitely has complexity `O(n log n)`, but it has a
//! much larger constant factor than an optimised library like
//! [FFTW](http://www.fftw.org/). (Strictly speaking `output` does not
//! need to be a strided slice, since it is never split into
//! alternating elements.)

#![feature(core)]
#![cfg_attr(test, feature(test))]

#[cfg(test)] extern crate test;

pub use base::{Items, MutItems};

pub use mut_::Stride as MutStride;
pub use mut_::Substrides as MutSubstrides;

pub use imm::Stride as Stride;
pub use imm::Substrides as Substrides;


pub use traits::{Strided, MutStrided};

#[cfg(test)]
mod common_tests;

mod base;
mod mut_;
mod imm;
mod traits;

#[cfg(test)]
mod bench {
    use super::Stride;
    use test::Bencher as B;
    use test;

    const N: usize = 100;

    #[bench]
    fn iter_slice(b: &mut B) {
        let v = (0..N).collect::<Vec<_>>();
        b.iter(|| {
            test::black_box(&v);
            for e in v.iter() { test::black_box(e); }
        })
    }

    #[bench]
    fn iter_step_1(b: &mut B) {
        let v = (0..N).collect::<Vec<_>>();
        let s = Stride::new(&*v);
        b.iter(|| {
            test::black_box(&s);
            for e in s.iter() { test::black_box(e); }
        })
    }

    #[bench]
    fn iter_step_13(b: &mut B) {
        let v = (0..13 * N).collect::<Vec<_>>();
        let s = Stride::new(&*v);
        let s = s.substrides(13).next().unwrap();
        b.iter(|| {
            test::black_box(&s);
            for e in s.iter() { test::black_box(e); }
        })
    }
}
