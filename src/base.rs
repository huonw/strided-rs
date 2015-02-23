use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::iter::order;
use std::marker;
use std::mem;
use std::num::Int;

#[repr(C)]
#[derive(Clone)]
#[allow(raw_pointer_derive)]
pub struct Stride<'a,T: 'a> {
    data: *const T,
    len: usize,
    stride: usize,

    _marker: marker::PhantomData<&'a T>,
}

impl<'a, T> Copy for Stride<'a, T> {}

impl<'a, T: PartialEq> PartialEq for Stride<'a, T> {
    fn eq(&self, other: &Stride<'a, T>) -> bool {
        self.len() == other.len() &&
            order::eq(self.iter(), other.iter())
    }
}
impl<'a, T: Eq> Eq for Stride<'a, T> {}

impl<'a, T: PartialOrd> PartialOrd for Stride<'a, T> {
    fn partial_cmp(&self, other: &Stride<'a, T>) -> Option<Ordering> {
        order::partial_cmp(self.iter(), other.iter())
    }
}
impl<'a, T: Ord> Ord for Stride<'a, T> {
    fn cmp(&self, other: &Stride<'a, T>) -> Ordering {
        order::cmp(self.iter(), other.iter())
    }
}

impl<'a, T: Debug> Debug for Stride<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "["));
        let mut is_first = true;
        for x in self.iter() {
            if is_first {
                is_first = false;
            } else {
                try!(write!(f, ", "));
            }
            try!(write!(f, "{:?}", *x))
        }
        write!(f, "]")
    }
}


unsafe fn step<T>(ptr: *const T, stride: usize) -> *const T {
    debug_assert!(stride % mem::size_of::<T>() == 0);
    (ptr as *const u8).offset(stride as isize) as *const T
}

impl<'a, T> Stride<'a, T> {
    #[inline(always)]
    pub fn new(data: *mut T, len: usize, elem_stride: usize) -> Stride<'a, T> {
        Stride::new_raw(data, len, elem_stride * mem::size_of::<T>())
    }

    fn new_raw(data: *mut T, len: usize, byte_stride: usize) -> Stride<'a, T> {
        // remove this assertion
        assert!(mem::size_of::<T>() != 0);
        Stride {
            data: data,
            len: len,
            stride: byte_stride,
            _marker: marker::PhantomData,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline(always)]
    pub fn stride(&self) -> usize {
        self.stride
    }
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.data as *mut T
    }


    pub fn substrides2(self) -> (Stride<'a, T>, Stride<'a, T>) {
        let left_len = (self.len() + 1)/2;
        let right_len = self.len() - left_len;
        let stride = self.stride.checked_mul(2).expect("Stride.substrides2: stride too large");

        let left_ptr = self.data;
        let right_ptr = if self.len() == 0 {
            left_ptr
        } else {
            unsafe {step(left_ptr, self.stride)}
        };

        (Stride::new_raw(left_ptr as *mut _, left_len, stride),
         Stride::new_raw(right_ptr as *mut _, right_len, stride))
    }

    #[inline]
    pub fn substrides(self, n: usize) -> Substrides<'a, T> {
        assert!(n != 0);
        let long_len = (self.len() + n - 1) / n;
        let new_stride = n.checked_mul(self.stride).expect("Stride.substrides: stride too large");
        Substrides {
            x: Stride::new_raw(self.data as *mut _, long_len, new_stride),
            base_stride: self.stride,
            nlong: self.len() % n,
            count: n
        }
    }

    pub fn iter(&self) -> Items<'a, T> {
        assert!(self.data as usize + self.len * self.stride >= self.data as usize);
        Items {
            start: self.data as *const _,
            // this points one-stride past the end, and so is
            // possibly undefined behaviour since the underlying array
            // doesn't necessarily extend this far (e.g. a Stride of
            // [1, 2, 3] starting at 2 with stride 2)
            end: unsafe {step(self.data, self.stride * self.len)},
            stride: self.stride,
            _marker: marker::PhantomData,
        }
    }
    pub fn iter_mut(&mut self) -> MutItems<'a, T> {
        assert!(self.data as usize + self.len * self.stride >= self.data as usize);
        MutItems {
            start: self.data as *mut _,
            end: unsafe {step(self.data, self.stride * self.len) as *mut _},
            stride: self.stride,
            _marker: marker::PhantomData,
        }
    }

    #[inline]
    pub fn get(&self, n: usize) -> Option<&'a T> {
        if n < self.len {
            unsafe {Some(&*step(self.data, n * self.stride))}
        } else {
            None
        }
    }
    #[inline]
    pub fn get_mut(&mut self, n: usize) -> Option<&'a mut T> {
        if n < self.len {
            unsafe {Some(&mut *(step(self.data, n * self.stride) as *mut _))}
        } else {
            None
        }
    }


    #[inline]
    pub fn slice(self, from: usize, to: usize) -> Stride<'a, T> {
        assert!(from <= to && to <= self.len());
        unsafe {
            Stride::new_raw(step(self.data, from * self.stride) as *mut _,
                            to - from, self.stride)
        }
    }
    #[inline]
    pub fn slice_from(self, from: usize) -> Stride<'a, T> {
        self.slice(from, self.len())
    }
    #[inline]
    pub fn slice_to(self, to: usize) -> Stride<'a, T> {
        self.slice(0, to)
    }

    pub fn split_at(self, idx: usize) -> (Stride<'a, T>, Stride<'a, T>) {
        assert!(idx <= self.len());
        unsafe {
            (Stride::new_raw(self.data as *mut _, idx, self.stride),
             Stride::new_raw(step(self.data, idx * self.stride) as *mut _,
                             self.len() - idx, self.stride))
        }
    }
}

macro_rules! iterator {
    ($name: ident -> $elem: ty) => {
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = $elem;
            #[inline]
            fn next(&mut self) -> Option<$elem> {
                if self.start < self.end {
                    unsafe {
                        let ret = Some(mem::transmute::<_, $elem>(self.start));
                        self.start = mem::transmute(step(self.start as *mut T, self.stride));
                        ret
                    }
                } else {
                    None
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let n = (self.end as usize - self.start as usize) / self.stride as usize;
                (n, Some(n))
            }
        }

        impl<'a, T> DoubleEndedIterator for $name<'a, T> {
            #[inline]
            #[allow(unsigned_negation)]
            fn next_back(&mut self) -> Option<$elem> {
                if self.start < self.end {
                    unsafe {
                        self.end = mem::transmute(step(self.end as *mut T, -self.stride));
                        Some(mem::transmute::<_, $elem>(self.end))
                    }
                } else {
                    None
                }
            }
        }
    }
}

/// An iterator over shared references to the elements of a strided
/// slice.
#[allow(raw_pointer_derive)]
#[derive(Copy)]
pub struct Items<'a, T: 'a> {
    start: *const T,
    end: *const T,
    stride: usize,
    _marker: marker::PhantomData<&'a T>,
}
iterator!(Items -> &'a T);

/// An iterator over mutable references to the elements of a strided
/// slice.
pub struct MutItems<'a, T: 'a> {
    start: *mut T,
    end: *mut T,
    stride: usize,
    _marker: marker::PhantomData<&'a mut T>,
}
iterator!(MutItems -> &'a mut T);

pub struct Substrides<'a, T: 'a> {
    x: Stride<'a, T>,
    base_stride: usize,
    nlong: usize,
    count: usize
}

impl<'a, T> Iterator for Substrides<'a, T> {
    type Item = Stride<'a, T>;
    fn next(&mut self) -> Option<Stride<'a, T>> {
        if self.count == 0 { return None }
        self.count -= 1;

        let ret = self.x;

        if self.nlong > 0 {
            self.nlong -= 1;
            if self.nlong == 0 {
                self.x.len -= 1;
            }
        }
        if self.x.len > 0 {
            self.x.data = unsafe {step(self.x.data, self.base_stride)};
        }
        Some(ret)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}
