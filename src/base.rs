use std::fmt::{mod, Show};
use std::iter::order;
use std::kinds::marker;
use std::mem;
use std::num::Int;

#[repr(C)]
#[deriving(Clone)]
#[allow(raw_pointer_deriving)]
pub struct Stride<'a,T: 'a> {
    data: *mut T,
    len: uint,
    stride: uint,

    _marker: marker::ContravariantLifetime<'a>,
}

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

impl<'a, T: Show> Show for Stride<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.flags() & (1 << (fmt::rt::FlagAlternate as uint)) == 0 {
            try!(write!(f, "["));
        }
        let mut is_first = true;
        for x in self.iter() {
            if is_first {
                is_first = false;
            } else {
                try!(write!(f, ", "));
            }
            try!(write!(f, "{}", *x))
        }
        if f.flags() & (1 << (fmt::rt::FlagAlternate as uint)) == 0 {
            try!(write!(f, "]"));
        }
        Ok(())
    }
}


unsafe fn step<T>(ptr: *mut T, stride: uint) -> *mut T {
    debug_assert!(stride % mem::size_of::<T>() == 0);
    (ptr as *mut u8).offset(stride as int) as *mut T
}

impl<'a, T> Stride<'a, T> {
    #[inline(always)]
    pub fn new(data: *mut T, len: uint, elem_stride: uint) -> Stride<'a, T> {
        Stride::new_raw(data, len, elem_stride * mem::size_of::<T>())
    }

    fn new_raw(data: *mut T, len: uint, byte_stride: uint) -> Stride<'a, T> {
        // remove this assertion
        assert!(mem::size_of::<T>() != 0);
        Stride {
            data: data,
            len: len,
            stride: byte_stride,
            _marker: marker::ContravariantLifetime
        }
    }

    #[inline(always)]
    pub fn len(&self) -> uint {
        self.len
    }
    #[inline(always)]
    pub fn stride(&self) -> uint {
        self.stride
    }
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.data
    }


    pub fn substrides2(self) -> (Stride<'a, T>, Stride<'a, T>) {
        let left_len = (self.len() + 1)/2;
        let right_len = self.len() - left_len;
        let stride = self.stride.checked_mul(2).expect("Stride.substrides2: stride too large");

        let right_ptr = if self.len() == 0 {
            self.data
        } else {
            unsafe {step(self.data, self.stride)}
        };

        (Stride::new_raw(self.data, left_len, stride),
         Stride::new_raw(right_ptr, right_len, stride))
    }

    #[inline]
    pub fn substrides(self, n: uint) -> Substrides<'a, T> {
        assert!(n != 0);
        let long_len = (self.len() + n - 1) / n;
        let new_stride = n.checked_mul(self.stride).expect("Stride.substrides: stride too large");
        Substrides {
            x: Stride::new_raw(self.data, long_len, new_stride),
            base_stride: self.stride,
            nlong: self.len() % n,
            count: n
        }
    }

    pub fn iter(&self) -> Items<'a, T> {
        assert!(self.data as uint + self.len * self.stride >= self.data as uint);
        Items {
            start: self.data as *const _,
            // this points one-stride past the end, and so is
            // possibly undefined behaviour since the underlying array
            // doesn't necessarily extend this far (e.g. a Stride of
            // [1, 2, 3] starting at 2 with stride 2)
            end: unsafe {step(self.data, self.stride * self.len) as *const _},
            stride: self.stride,
            _marker: marker::ContravariantLifetime,
        }
    }
    pub fn iter_mut(&mut self) -> MutItems<'a, T> {
        assert!(self.data as uint + self.len * self.stride >= self.data as uint);
        MutItems {
            start: self.data,
            end: unsafe {step(self.data, self.stride * self.len)},
            stride: self.stride,
            _marker: (marker::ContravariantLifetime, marker::NoCopy),
        }
    }

    #[inline]
    pub fn get(&self, n: uint) -> Option<&'a T> {
        if n < self.len {
            unsafe {Some(&*step(self.data, n * self.stride))}
        } else {
            None
        }
    }
    #[inline]
    pub fn get_mut(&mut self, n: uint) -> Option<&'a mut T> {
        if n < self.len {
            unsafe {Some(&mut *step(self.data, n * self.stride))}
        } else {
            None
        }
    }


    #[inline]
    pub fn slice(self, from: uint, to: uint) -> Stride<'a, T> {
        assert!(from <= to && to <= self.len());
        unsafe {
            Stride::new_raw(step(self.data, from * self.stride), to - from, self.stride)
        }
    }
    #[inline]
    pub fn slice_from(self, from: uint) -> Stride<'a, T> {
        self.slice(from, self.len())
    }
    #[inline]
    pub fn slice_to(self, to: uint) -> Stride<'a, T> {
        self.slice(0, to)
    }

    pub fn split_at(self, idx: uint) -> (Stride<'a, T>, Stride<'a, T>) {
        assert!(idx <= self.len());
        unsafe {
            (Stride::new_raw(self.data, idx, self.stride),
             Stride::new_raw(step(self.data, idx * self.stride), self.len() - idx, self.stride))
        }
    }
}

macro_rules! iterator {
    ($name: ident -> $elem: ty) => {
        impl<'a, T> Iterator<$elem> for $name<'a, T> {
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
            fn size_hint(&self) -> (uint, Option<uint>) {
                let n = (self.end as uint - self.start as uint) / self.stride as uint;
                (n, Some(n))
            }
        }

        impl<'a, T> DoubleEndedIterator<$elem> for $name<'a, T> {
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
pub struct Items<'a, T: 'a> {
    start: *const T,
    end: *const T,
    stride: uint,
    _marker: marker::ContravariantLifetime<'a>,
}
iterator!(Items -> &'a T)

/// An iterator over mutable references to the elements of a strided
/// slice.
pub struct MutItems<'a, T: 'a> {
    start: *mut T,
    end: *mut T,
    stride: uint,
    _marker: (marker::ContravariantLifetime<'a>, marker::NoCopy),
}
iterator!(MutItems -> &'a mut T)

pub struct Substrides<'a, T: 'a> {
    x: Stride<'a, T>,
    base_stride: uint,
    nlong: uint,
    count: uint
}

impl<'a, T> Iterator<Stride<'a, T>> for Substrides<'a, T> {
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

    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.count, Some(self.count))
    }
}
