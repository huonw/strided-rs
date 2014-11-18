use {Stride, MutStride};

/// Things that can be viewed as a series of equally spaced `T`s in
/// memory.
pub trait Strided<T> for Sized? {
    fn as_stride(&self) -> Stride<T>;

    fn stride(&self) -> uint {
        self.as_stride().stride()
    }
}

/// Things that can be viewed as a series of mutable equally spaced
/// `T`s in memory.
pub trait MutStrided<T> for Sized? : Strided<T> {
    fn as_stride_mut(&mut self) -> MutStride<T>;
}

// this isn't as general as it could be.
impl<T, X: Deref<[T]>> Strided<T> for X {
    fn as_stride(&self) -> Stride<T> {
        Stride::new(&**self)
    }

    #[inline(always)]
    fn stride(&self) -> uint {
        1
    }
}
impl<T, X: DerefMut<[T]>> MutStrided<T> for X {
    fn as_stride_mut(&mut self) -> MutStride<T> {
        MutStride::new(&mut **self)
    }
}

impl<T> Strided<T> for [T] {
    fn as_stride(&self) -> Stride<T> { Stride::new(self) }
    #[inline(always)]
    fn stride(&self) -> uint { 1 }
}
impl<T> MutStrided<T> for [T] {
    fn as_stride_mut(&mut self) -> MutStride<T> { MutStride::new(self) }
}

impl<'a,T> Strided<T> for Stride<'a,T> {
    fn as_stride(&self) -> Stride<T> { *self }
    fn stride(&self) -> uint { Stride::stride(self) }
}
impl<'a,T> Strided<T> for MutStride<'a,T> {
    fn as_stride(&self) -> Stride<T> { **self }
    fn stride(&self) -> uint { MutStride::stride(self) }
}
impl<'a,T> MutStrided<T> for MutStride<'a,T> {
    fn as_stride_mut(&mut self) -> MutStride<T> { self.reborrow() }
}
