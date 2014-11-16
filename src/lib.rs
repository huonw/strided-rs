#![feature(macro_rules)]

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
