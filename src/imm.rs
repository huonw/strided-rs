use std::fmt::{self, Debug};
use std::mem;
use std::ops::Index;
use base;
use base::Stride as Base;

/// A shared strided slice. This is equivalent to a `&[T]` that only
/// refers to every `n`th `T`.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stride<'a,T: 'a> {
    base: Base<'a, T>,
}
impl<'a, T> Copy for Stride<'a, T> {}

impl<'a, T: Debug> Debug for Stride<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.base.fmt(f)
    }
}

impl<'a, T> Stride<'a, T> {
    #[inline(always)]
    fn new_raw(base: Base<'a, T>) -> Stride<'a, T> {
        Stride {
            base: base,
        }
    }

    /// Creates a new strided slice directly from a conventional
    /// slice. The return value has stride 1.
    #[inline(always)]
    pub fn new(x: &'a [T]) -> Stride<'a, T> {
        Stride::new_raw(Base::new(x.as_ptr() as *mut _, x.len(), 1))
    }

    /// Returns the number of elements accessible in `self`.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.base.len()
    }
    /// Returns the offset between successive elements of `self` as a
    /// count of *elements*, not bytes.
    #[inline(always)]
    pub fn stride(&self) -> usize {
        self.base.stride() / mem::size_of::<T>()
    }
    /// Returns a pointer to the first element of this strided slice.
    ///
    /// NB. one must be careful since only every `self.stride()`th
    /// element is guaranteed to have unique access via this object;
    /// the others may be under the control of some other strided
    /// slice.
    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.base.as_mut_ptr() as *const T
    }

    /// Creates a temporary copy of this strided slice.
    ///
    /// This is an explicit form of the reborrowing the compiler does
    /// implicitly for conventional `&mut` pointers. This is designed
    /// to allow the by-value `self` methods to be used without losing
    /// access to the slice. (This exists for consistency with
    /// `MutStride`, since this shared form is `Copy` and so
    /// reborrowing is unnecessary.)
    #[inline(always)]
    #[cfg(test)]
    fn reborrow<'b>(&'b self) -> Stride<'b, T> {
        // only exists to work with tests.
        *self
    }


    /// Breaks this strided slice into two strided slices pointing to
    /// alternate elements.
    ///
    /// That is, it doubles the stride and (approximately) halves the
    /// length. A slice pointing to values `[1, 2, 3, 4, 5]` becomes
    /// two slices `[1, 3, 5]` and `[2, 4]`. This is guaranteed to
    /// succeed even if `self.len()` is odd, and even if `self`
    /// has only zero or one elements.
    #[inline]
    pub fn substrides2(&self) -> (Stride<'a, T>, Stride<'a, T>) {
        let (l, r) = self.base.substrides2();
        (Stride::new_raw(l), Stride::new_raw(r))
    }

    /// Returns an iterator over `n` strided subslices of `self` each
    /// pointing to every `n`th element, starting at successive
    /// offsets.
    ///
    /// Calling `substrides(3)` on a slice pointing to `[1, 2, 3, 4, 5, 6,
    /// 7]` will yield, in turn, `[1, 4, 7]`, `[2, 5]` and finally
    /// `[3, 6]`. Like with `split2` this is guaranteed to succeed
    /// (return `n` strided slices) even if `self` has fewer than `n`
    /// elements and if `self.len()` is not a multiple of `n`.
    #[inline]
    pub fn substrides(&self, n: usize) -> Substrides<'a, T> {
        Substrides {
            base: self.base.substrides(n),
        }
    }
    /// Returns a reference to the `n`th element of `self`, or `None`
    /// if `n` is out-of-bounds.
    #[inline]
    pub fn get(&self, n: usize) -> Option<&'a T> {
        self.base.get(n)
    }

    /// Returns an iterator over references to each successive element
    /// of `self`.
    ///
    /// Unlike `MutStrides`, this can return references with the
    /// maximum lifetime without consuming `self` and so an
    /// `into_iter` equivalent is unnecessary.
    #[inline]
    pub fn iter(&self) -> ::Items<'a, T> {
        self.base.iter()
    }

    /// Returns a strided slice containing only the elements from
    /// indices `from` (inclusive) to `to` (exclusive).
    ///
    /// # Panic
    ///
    /// Panics if `from > to` or if `to > self.len()`.
    #[inline]
    pub fn slice(&self, from: usize, to: usize) -> Stride<'a, T> {
        Stride::new_raw(self.base.slice(from, to))
    }
    /// Returns a strided slice containing only the elements from
    /// index `from` (inclusive).
    ///
    /// # Panic
    ///
    /// Panics if `from > self.len()`.
    #[inline]
    pub fn slice_from(&self, from: usize) -> Stride<'a, T> {
        Stride::new_raw(self.base.slice_from(from))
    }
    /// Returns a strided slice containing only the elements to
    /// index `to` (exclusive).
    ///
    /// # Panic
    ///
    /// Panics if `to > self.len()`.
    #[inline]
    pub fn slice_to(&self, to: usize) -> Stride<'a, T> {
        Stride::new_raw(self.base.slice_to(to))
    }
    /// Returns two strided slices, the first with elements up to
    /// `idx` (exclusive) and the second with elements from `idx`.
    ///
    /// This is semantically equivalent to `(self.slice_to(idx),
    /// self.slice_from(idx))`.
    ///
    /// # Panic
    ///
    /// Panics if `idx > self.len()`.
    #[inline]
    pub fn split_at(&self, idx: usize) -> (Stride<'a, T>, Stride<'a, T>) {
        let (l, r) = self.base.split_at(idx);
        (Stride::new_raw(l), Stride::new_raw(r))
    }
}

impl<'a, T> Index<usize> for Stride<'a, T> {
    type Output = T;
    fn index<'b>(&'b self, n: &usize) -> &'b T {
        self.get(*n).expect("Stride.index: index out of bounds")
    }
}

/// An iterator over `n` shared substrides of a given stride, each of
/// which points to every `n`th element starting at successive
/// offsets.
pub struct Substrides<'a, T: 'a> {
    base: base::Substrides<'a, T>,
}

impl<'a, T> Iterator for Substrides<'a, T> {
    type Item = Stride<'a, T>;
    fn next(&mut self) -> Option<Stride<'a, T>> {
        match self.base.next() {
            Some(s) => Some(Stride::new_raw(s)),
            None => None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
}

#[cfg(test)]
#[allow(unused_mut)]
mod tests {
    use super::Stride;
    make_tests!(substrides2, substrides, slice, slice_to, slice_from, split_at, get, iter, );
}
