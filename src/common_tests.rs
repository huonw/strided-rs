// this is written to be used by both mut_ and imm.

use super::Strided;

#[test]
#[should_fail]
fn no_zero_sized_types() {
    // FIXME: remove this test
    let v = &mut [()];
    Strided::new(v);
}

#[test]
fn stride_len() {
    let v = &mut [1u8, 2, 3, 4, 5];
    let mut _s = Strided::new(v);
    assert_eq!(_s.len(), 5);
    assert_eq!(_s.stride(), 1);

    {
        let (l, r) = _s.reborrow().substrides2();
        assert_eq!(l.len(), 3);
        assert_eq!(r.len(), 2);

        assert_eq!(l.stride(), 2);
        assert_eq!(r.stride(), 2);

        let (ll, lr) = l.substrides2();
        assert_eq!(ll.len(), 2);
        assert_eq!(lr.len(), 1);
        assert_eq!(ll.stride(), 4);
        assert_eq!(lr.stride(), 4);
    }

    let mut it = _s.substrides(3);
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
fn iter() {
    let v = &mut [1u8, 2, 3, 4, 5];
    let s = Strided::new(v);
    eq!(s, [1, 2, 3, 4, 5]);
}

#[test]
fn substrides2() {
    macro_rules! test {
        ($input: expr, $L: expr, $R: expr) => {{
            let v: &mut [u8] = &mut $input;
            let s = Strided::new(v);
            let (l, r) = s.substrides2();
            eq!(l, $L);
            eq!(r, $R);
        }}
    }
    test!([1, 2, 3, 4, 5], [1, 3, 5], [2, 4]);
    test!([1, 2, 3, 4], [1, 3], [2, 4]);
    test!([1, 2, 3], [1, 3], [2]);
    test!([1, 2], [1], [2]);
    test!([1], [1], []);
    test!([], [], []);
}

#[test]
fn substrides() {
    macro_rules! test {
        ($n: expr, $input: expr, [$($expected: expr),*]) => {{
            let v: &mut [u8] = &mut $input;
            let s = Strided::new(v);
            let expected: &[&[_]] = [$({ const X: &'static [u8] = &$expected; X }),*];
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


    test!(3, [1, 2, 3, 4, 5, 6, 7], [[1, 4, 7], [2, 5], [3, 6]]);
    test!(3, [1, 2, 3, 4, 5, 6], [[1, 4], [2, 5], [3, 6]]);
    test!(3, [1, 2, 3, 4, 5], [[1, 4], [2, 5], [3]]);
    test!(3, [1, 2, 3, 4], [[1, 4], [2], [3]]);
    test!(3, [1, 2, 3], [[1], [2], [3]]);
    test!(3, [1, 2], [[1], [2], []]);
    test!(3, [1], [[1], [], []]);
    test!(3, [], [[], [], []]);

    test!(2, [1, 2, 3], [[1, 3], [2]]);
    test!(1, [1, 2, 3], [[1, 2, 3]])
}

#[test]
fn get() {
    macro_rules! test {
        ($input: expr, $expected: expr) => {{
            let e = $expected;
            for i in range(0, e.len() + 10) {
                let expected = e.get(i);
                assert_eq!($input.get(i), expected);
                match expected {
                    Some(x) => assert_eq!(&$input[i], x),
                    None => {}
                }
            }
        }}
    }

    let v: &mut [u8] = [1, 2, 3, 4, 5, 6];
    let base = Strided::new(v);
    test!(base, [1,2,3,4,5,6]);
    let (l, r) = base.substrides2();
    test!(l, [1,3,5]);
    test!(r, [2,4,6])
}
