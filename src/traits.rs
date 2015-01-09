use {Stride, MutStride};
use std::ops::{Deref, DerefMut};

/// Things that can be viewed as a series of equally spaced `T`s in
/// memory.
pub trait Strided {
    type Elem;
    fn as_stride(&self) -> Stride<Self::Elem>;

    fn stride(&self) -> usize {
        self.as_stride().stride()
    }
}

/// Things that can be viewed as a series of mutable equally spaced
/// `T`s in memory.
pub trait MutStrided : Strided {
    fn as_stride_mut(&mut self) -> MutStride<<Self as Strided>::Elem>;
}

// this isn't as general as it could be.
impl<T, X: Deref<Target=[T]>> Strided for X {
    type Elem = T;
    fn as_stride(&self) -> Stride<T> {
        Stride::new(&**self)
    }

    #[inline(always)]
    fn stride(&self) -> usize {
        1
    }
}
impl<T, X: DerefMut + Deref<Target=[T]>> MutStrided for X {
    fn as_stride_mut(&mut self) -> MutStride<T> {
        MutStride::new(&mut **self)
    }
}

impl<T> Strided for [T] {
    type Elem = T;
    fn as_stride(&self) -> Stride<T> { Stride::new(self) }
    #[inline(always)]
    fn stride(&self) -> usize { 1 }
}
impl<T> MutStrided for [T] {
    fn as_stride_mut(&mut self) -> MutStride<T> { MutStride::new(self) }
}

impl<'a,T> Strided for Stride<'a,T> {
    type Elem = T;
    fn as_stride(&self) -> Stride<T> { *self }
    fn stride(&self) -> usize { Stride::stride(self) }
}
impl<'a,T> Strided for MutStride<'a,T> {
    type Elem = T;
    fn as_stride(&self) -> Stride<T> { **self }
    fn stride(&self) -> usize { MutStride::stride(self) }
}
impl<'a,T> MutStrided for MutStride<'a,T> {
    fn as_stride_mut(&mut self) -> MutStride<T> { self.reborrow() }
}
