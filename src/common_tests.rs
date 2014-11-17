// this is written to be used by both mut_ and imm.
#![macro_escape]

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

macro_rules! substrides2 {
    ($substrides2: ident, $input: expr, $L: expr, $R: expr) => {{
        let v: &mut [u16] = &mut $input;
        let s = Stride::new(v);
        let (l, r) = s.$substrides2();
        eq!(l, $L);
        eq!(r, $R);
    }}
}

macro_rules! substrides {
    ($substrides: ident, $n: expr, $input: expr, [$($expected: expr),*]) => {{
        let v: &mut [u16] = &mut $input;
        let s = Stride::new(v);
        let expected: &[&[_]] = [$({ const X: &'static [u16] = &$expected; X }),*];
        let mut n = 0u;
        let mut it = s.substrides($n);
        assert_eq!(it.size_hint(), ($n, Some($n)));
        for (test, real) in it.by_ref().zip(expected.iter()) {
            eq!(test, *real);
            n += 1;
        }
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(n, $n);
    }}
}

macro_rules! get {
    ($get: ident, $input: expr, $expected: expr, $($mut_: tt)*) => {{
        let mut e = $expected;
        for i in range(0, e.len() + 10) {
            let expected = e.$get(i);
            assert_eq!($input.$get(i).map(|x| *x), expected.as_ref().map(|x| **x));

            match expected {
                Some(x) => assert_eq!(*(&$($mut_)* $input[i]), *x),
                None => {}
            }
        }
    }}
}


macro_rules! make_tests {
    ($substrides2: ident, $substrides: ident,
     $slice: ident, $slice_to: ident, $slice_from: ident,
     $split_at: ident, $get: ident, $iter: ident,
     $($mut_:tt)*) => {
        #[test]
        #[should_fail]
        fn no_zero_sized_types() {
            // FIX ME: remove this test
            let v = &mut [()];
            Stride::new(v);
        }

        #[test]
        fn stride_len() {
            let v = &mut [1u16, 2, 3, 4, 5];
            let mut _s = Stride::new(v);
            assert_eq!(_s.len(), 5);
            assert_eq!(_s.stride(), 1);

            {
                let (l, r) = _s.reborrow().$substrides2();
                assert_eq!(l.len(), 3);
                assert_eq!(r.len(), 2);

                assert_eq!(l.stride(), 2);
                assert_eq!(r.stride(), 2);

                let (ll, lr) = l.$substrides2();
                assert_eq!(ll.len(), 2);
                assert_eq!(lr.len(), 1);
                assert_eq!(ll.stride(), 4);
                assert_eq!(lr.stride(), 4);
            }

            let mut it = _s.$substrides(3);
            let a = it.next().unwrap();
            let b = it.next().unwrap();
            let c = it.next().unwrap();
            assert!(it.next().is_none());

            assert_eq!(a.len(), 2);
            assert_eq!(b.len(), 2);
            assert_eq!(c.len(), 1);

            assert_eq!(a.stride(), 3);
            assert_eq!(b.stride(), 3);
            assert_eq!(c.stride(), 3);
        }

        #[test]
        fn show() {
            assert_eq!(format!("{}", Stride::new(&mut [1u16, 2, 3, 4, 5]).$substrides2().0),
                       "[1, 3, 5]".into_string());
            assert_eq!(format!("{}", Stride::new(&mut [1u16, 2, 3]).$substrides2().0),
                       "[1, 3]".into_string());
            assert_eq!(format!("{}", Stride::new(&mut [1u16]).$substrides2().0),
                       "[1]".into_string());
            assert_eq!(format!("{}", Stride::<u16>::new(&mut []).$substrides2().0),
                       "[]".into_string());

            assert_eq!(format!("{:#}", Stride::new(&mut [1u16, 2, 3, 4, 5]).$substrides2().0),
                       "1, 3, 5".into_string());
            assert_eq!(format!("{:#}", Stride::new(&mut [1u16, 2, 3]).$substrides2().0),
                       "1, 3".into_string());
            assert_eq!(format!("{:#}", Stride::new(&mut [1u16]).$substrides2().0),
                       "1".into_string());
            assert_eq!(format!("{:#}", Stride::<u16>::new(&mut []).$substrides2().0),
                       "".into_string())
        }

        #[test]
        fn comparisons() {
            use std::f64;

            let v = &mut [1u16, 2, 3, 4, 5];
            let w = &mut [1, 2, 3, 4, 100];
            let mut s = Stride::new(v);
            let mut t = Stride::new(w);

            assert!(s != t);
            assert!(s == s);
            assert!(t == t);
            assert!(s.reborrow().$slice_to(4) == t.reborrow().$slice_to(4));

            assert_eq!(s.partial_cmp(&t), Some(Less));
            assert_eq!(s.cmp(&t), Less);
            assert_eq!(s.partial_cmp(&s), Some(Equal));
            assert_eq!(s.cmp(&s), Equal);
            assert_eq!(t.partial_cmp(&s), Some(Greater));
            assert_eq!(t.cmp(&s), Greater);
            assert_eq!(t.partial_cmp(&t), Some(Equal));
            assert_eq!(t.cmp(&t), Equal);

            let v = &mut [1.0, f64::NAN];
            let s = Stride::new(v);
            assert_eq!(s.partial_cmp(&s), None);
        }

        #[test]
        fn slice_split() {
            let v = &mut [1u16, 2, 3, 4, 5, 6, 7];
            let s = Stride::new(v);
            let (mut l, mut r) = s.$substrides2();
            eq!(l.reborrow(), [1, 3, 5, 7]);
            eq!(r.reborrow(), [2, 4, 6]);

            eq!(l.reborrow().$slice(1, 3), [3, 5]);
            eq!(l.reborrow().$slice(0, 4), [1, 3, 5, 7]);
            eq!(l.reborrow().$slice_to(3), [1, 3, 5]);
            eq!(l.reborrow().$slice_to(0), []);
            eq!(l.reborrow().$slice_from(2), [5, 7]);
            eq!(l.reborrow().$slice_from(4), []);

            let (ll, lr) = l.$split_at(2);
            eq!(ll, [1, 3]);
            eq!(lr, [5, 7]);
            {
                let (rl, rr) = r.reborrow().$split_at(0);
                eq!(rl, []);
                eq!(rr, [2, 4, 6]);
            }
            {
                let (rl, rr) = r.reborrow().$split_at(3);
                eq!(rl, [2, 4, 6]);
                eq!(rr, []);
            }
        }

        #[test]
        fn iter() {
            let v = &mut [1u16, 2, 3, 4, 5];
            let mut s = Stride::new(v);
            let mut n = 0u;
            for (x, y) in s.$iter().zip([1,2,3,4,5].iter()) {
                assert_eq!(*x, *y);
                n += 1;
            }
            assert_eq!(n, 5)
            let mut n = 0u;
            for (x, y) in (s.$substrides2().0).$iter().zip([1,3,5].iter()) {
                assert_eq!(*x, *y);
                n += 1;
            }
            assert_eq!(n, 3)
        }

        #[test]
        fn substrides2() {
            substrides2!($substrides2, [1, 2, 3, 4, 5], [1, 3, 5], [2, 4]);
            substrides2!($substrides2, [1, 2, 3, 4], [1, 3], [2, 4]);
            substrides2!($substrides2, [1, 2, 3], [1, 3], [2]);
            substrides2!($substrides2, [1, 2], [1], [2]);
            substrides2!($substrides2, [1], [1], []);
            substrides2!($substrides2, [], [], []);
        }

        #[test]
        fn substrides() {
            substrides!($substrides, 3, [1, 2, 3, 4, 5, 6, 7], [[1, 4, 7], [2, 5], [3, 6]]);
            substrides!($substrides, 3, [1, 2, 3, 4, 5, 6], [[1, 4], [2, 5], [3, 6]]);
            substrides!($substrides, 3, [1, 2, 3, 4, 5], [[1, 4], [2, 5], [3]]);
            substrides!($substrides, 3, [1, 2, 3, 4], [[1, 4], [2], [3]]);
            substrides!($substrides, 3, [1, 2, 3], [[1], [2], [3]]);
            substrides!($substrides, 3, [1, 2], [[1], [2], []]);
            substrides!($substrides, 3, [1], [[1], [], []]);
            substrides!($substrides, 3, [], [[], [], []]);

            substrides!($substrides, 2, [1, 2, 3], [[1, 3], [2]]);
            substrides!($substrides, 1, [1, 2, 3], [[1, 2, 3]])
        }

        #[test]
        fn get() {
            let v: &mut [u16] = [1, 2, 3, 4, 5, 6];
            let mut base = Stride::new(v);
            get!($get, base, [1,2,3,4,5,6], $($mut_)*);
            let (mut l, mut r) = base.$substrides2();
            get!($get, l, [1,3,5], $($mut_)*);
            get!($get, r, [2,4,6], $($mut_)*)
        }

    }
}
