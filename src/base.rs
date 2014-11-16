use std::kinds::marker;
use std::mem;
use std::num::Int;

#[repr(C)]
pub struct Strided<'a,T: 'a> {
    data: *mut T,
    len: uint,
    stride: uint,

    _marker: marker::ContravariantLifetime<'a>,
}

unsafe fn step<T>(ptr: *mut T, stride: uint) -> *mut T {
    debug_assert!(stride % mem::size_of::<T>() == 0);
    (ptr as *mut u8).offset(stride as int) as *mut T
}

impl<'a, T> Strided<'a, T> {
    #[inline(always)]
    pub fn new(data: *mut T, len: uint, elem_stride: uint) -> Strided<'a, T> {
        Strided::new_raw(data, len, elem_stride * mem::size_of::<T>())
    }

    fn new_raw(data: *mut T, len: uint, byte_stride: uint) -> Strided<'a, T> {
        // remove this assertion
        assert!(mem::size_of::<T>() != 0);
        Strided {
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


    pub fn substrides2(self) -> (Strided<'a, T>, Strided<'a, T>) {
        let left_len = (self.len() + 1)/2;
        let right_len = self.len() - left_len;
        let stride = self.stride.checked_mul(2).expect("Strided.substrides2: stride too large");

        let right_ptr = if self.len() == 0 {
            self.data
        } else {
            unsafe {step(self.data, self.stride)}
        };

        (Strided::new_raw(self.data, left_len, stride),
         Strided::new_raw(right_ptr, right_len, stride))
    }

    #[inline]
    pub fn substrides(self, n: uint) -> Substrides<'a, T> {
        assert!(n != 0);
        let long_len = (self.len() + n - 1) / n;
        let new_stride = n.checked_mul(self.stride).expect("Strided.substrides: stride too large");
        Substrides {
            x: Strided::new_raw(self.data, long_len, new_stride),
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
            // doesn't necessarily extend this far (e.g. a Strided of
            // [1, 2, 3] starting at 2 with stride 2)
            end: unsafe {step(self.data, self.stride * self.len) as *const _},
            stride: self.stride,
            _marker: marker::ContravariantLifetime,
        }
    }
    pub fn iter_mut(&self) -> MutItems<'a, T> {
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
            unsafe {Some(&*self.data.offset((n * self.stride) as int))}
        } else {
            None
        }
    }
    #[inline]
    pub fn get_mut(&mut self, n: uint) -> Option<&'a mut T> {
        if n < self.len {
            unsafe {Some(&mut *self.data.offset((n * self.stride) as int))}
        } else {
            None
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

pub struct Items<'a, T: 'a> {
    start: *const T,
    end: *const T,
    stride: uint,
    _marker: marker::ContravariantLifetime<'a>,
}
iterator!(Items -> &'a T)

pub struct MutItems<'a, T: 'a> {
    start: *mut T,
    end: *mut T,
    stride: uint,
    _marker: (marker::ContravariantLifetime<'a>, marker::NoCopy),
}
iterator!(MutItems -> &'a mut T)

pub struct Substrides<'a, T: 'a> {
    x: Strided<'a, T>,
    base_stride: uint,
    nlong: uint,
    count: uint
}

impl<'a, T> Iterator<Strided<'a, T>> for Substrides<'a, T> {
    fn next(&mut self) -> Option<Strided<'a, T>> {
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
            self.x.data = unsafe {self.x.data.offset(self.base_stride as int)};
        }
        Some(ret)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        (self.count, Some(self.count))
    }
}
