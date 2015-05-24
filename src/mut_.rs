use std::fmt::{self, Debug};
use std::marker;
use std::mem;
use std::ops::{Index, IndexMut, Deref};
use base;
use base::Stride as Base;

/// A mutable strided slice. This is equivalent to `&mut [T]`, that
/// only refers to every `n`th `T`.
///
/// This can be viewed as an immutable strided slice via the `Deref`
/// implementation, and so many methods are available through that
/// type.
///
/// Many functions in this API take `self` and consume it. The
/// `reborrow` method is a key part of ensuring that ownership doesn't
/// disappear completely: it converts a reference
/// `&'b mut MutStride<'a, T>` into a `MutStride<'b, T>`, that is, gives a
/// by-value slice with a shorter lifetime. This can then be passed
/// directly into the functions that consume `self` without losing
/// control of the original slice.
#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord)] // FIXME: marker types
pub struct Stride<'a,T: 'a> {
    base: Base<'a, T>,
    _marker: marker::PhantomData<&'a mut T>,
}

unsafe impl<'a, T: Sync> Sync for Stride<'a, T> {}
unsafe impl<'a, T: Send> Send for Stride<'a, T> {}

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
            _marker: marker::PhantomData
        }
    }

    /// Creates a new strided slice directly from a conventional
    /// slice. The return value has stride 1.
    #[inline(always)]
    pub fn new(x: &'a mut [T]) -> Stride<'a, T> {
        Stride::new_raw(Base::new(x.as_mut_ptr(), x.len(), 1))
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
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.base.as_mut_ptr()
    }

    /// Creates a temporary copy of this strided slice.
    ///
    /// This is an explicit form of the reborrowing the compiler does
    /// implicitly for conventional `&mut` pointers. This is designed
    /// to allow the by-value `self` methods to be used without losing
    /// access to the slice.
    #[inline(always)]
    pub fn reborrow<'b>(&'b mut self) -> Stride<'b, T> {
        Stride::new_raw(self.base)
    }

    /// Breaks this strided slice into two strided slices pointing to
    /// alternate elements.
    ///
    /// That is, it doubles the stride and (approximately) halves the
    /// length. A slice pointing to values `[1, 2, 3, 4, 5]` becomes
    /// two slices `[1, 3, 5]` and `[2, 4]`. This is guaranteed to
    /// succeed even for mismatched lengths, and even if `self` has
    /// only zero or one elements.
    #[inline]
    pub fn substrides2_mut(self) -> (Stride<'a, T>, Stride<'a, T>) {
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
    /// elements.
    #[inline]
    pub fn substrides_mut(self, n: usize) -> Substrides<'a, T> {
        Substrides {
            base: self.base.substrides(n),
        }
    }
    /// Returns a reference to the `n`th element of `self`, or `None`
    /// if `n` is out-of-bounds.
    #[inline]
    pub fn get_mut<'b>(&'b mut self, n: usize) -> Option<&'b mut T> {
        self.base.get_mut(n).map(|r| &mut *r)
    }

    /// Returns an iterator over references to each successive element
    /// of `self`.
    ///
    /// See also `into_iter` which gives the references the maximum
    /// possible lifetime at the expense of consume the slice.
    #[inline]
    pub fn iter_mut<'b>(&'b mut self) -> ::MutItems<'b, T> {
        self.reborrow().into_iter()
    }

    /// Returns an iterator over reference to each successive element
    /// of `self`, with the maximum possible lifetime.
    ///
    /// See also `iter_mut` which avoids consuming `self` at the
    /// expense of shorter lifetimes.
    #[inline]
    pub fn into_iter(mut self) -> ::MutItems<'a, T> {
        self.base.iter_mut()
    }

    /// Returns a strided slice containing only the elements from
    /// indices `from` (inclusive) to `to` (exclusive).
    ///
    /// # Panic
    ///
    /// Panics if `from > to` or if `to > self.len()`.
    #[inline]
    pub fn slice_mut(self, from: usize, to: usize) -> Stride<'a, T> {
        Stride::new_raw(self.base.slice(from, to))
    }
    /// Returns a strided slice containing only the elements from
    /// index `from` (inclusive).
    ///
    /// # Panic
    ///
    /// Panics if `from > self.len()`.
    #[inline]
    pub fn slice_from_mut(self, from: usize) -> Stride<'a, T> {
        Stride::new_raw(self.base.slice_from(from))
    }
    /// Returns a strided slice containing only the elements to
    /// index `to` (exclusive).
    ///
    /// # Panic
    ///
    /// Panics if `to > self.len()`.
    #[inline]
    pub fn slice_to_mut(self, to: usize) -> Stride<'a, T> {
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
    pub fn split_at_mut(self, idx: usize) -> (Stride<'a, T>, Stride<'a, T>) {
        let (l, r) = self.base.split_at(idx);
        (Stride::new_raw(l), Stride::new_raw(r))
    }
}

impl<'a, T> Index<usize> for Stride<'a, T> {
    type Output = T;
    fn index<'b>(&'b self, n: usize) -> &'b T {
&        (**self)[n]
    }
}
impl<'a, T> IndexMut<usize> for Stride<'a, T> {
    fn index_mut<'b>(&'b mut self, n: usize) -> &'b mut T {
        self.get_mut(n).expect("Stride.index_mut: index out of bounds")
    }
}

impl<'a, T> Deref for Stride<'a, T> {
    type Target = ::imm::Stride<'a, T>;
    fn deref<'b>(&'b self) -> &'b ::imm::Stride<'a, T> {
        unsafe { mem::transmute(self) }
    }
}

/// An iterator over `n` mutable substrides of a given stride, each of
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
mod tests {
    use super::Stride;
    make_tests!(substrides2_mut, substrides_mut,
                slice_mut, slice_to_mut, slice_from_mut, split_at_mut, get_mut, iter_mut, mut);

    #[test]
    fn reborrow() {
        let v = &mut [1u8, 2, 3, 4, 5];
        let mut s = Stride::new(v);
        eq!(s.reborrow(), [1,2,3,4,5]);
        eq!(s.reborrow(), [1,2,3,4,5]);
    }
}
