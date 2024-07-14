#[macro_export]
macro_rules! assert_throws {
    ( $block:block, $message:expr $(,)? ) => {
        let error = std::panic::catch_unwind(|| $block).unwrap_err();
        if let Some(s) = error.downcast_ref::<&'static str>() {
            assert_eq!(*s, $message);
        } else if let Some(s) = error.downcast_ref::<String>() {
            assert_eq!(s, $message);
        } else {
            panic!("unexpected panic payload: {:?}", error);
        }
    };
    ( $statement:expr, $message:expr $(,)? ) => {
        assert_throws!({ $statement }, $message);
    };
}

// #[test]
// fn test_array() {}

// #[test]
// fn test_assign() {}

// #[test]
// fn test_async() {}

#[test]
fn test_await() {
    use std::task::*;
    const DUMMY_FN: fn(*const ()) = |_: *const ()| {};
    static CREATE: fn() -> RawWaker = || RawWaker::new(&() as _, &VTABLE);
    static VTABLE: RawWakerVTable = RawWakerVTable::new(|_| CREATE(), DUMMY_FN, DUMMY_FN, DUMMY_FN);
    let waker = unsafe { Waker::from_raw(CREATE()) };
    let mut cx = Context::from_waker(&waker);

    let true_fut = async { true };
    let false_fut = async { false };
    let expr = std::pin::pin!(async move {
        one_assert::assert!(true_fut.await);
    });
    assert_eq!(std::future::Future::poll(expr, &mut cx), Poll::Ready(()));

    assert_throws!(
        {
            let mut cx = Context::from_waker(&waker);
            let expr = std::pin::pin!(async move {
                one_assert::assert!(false_fut.await);
            });
            let _ = std::future::Future::poll(expr, &mut cx);
        },
        "assertion `false_fut.await` failed"
    );
}

#[test]
fn test_binary() {
    let a = 1;

    one_assert::assert!(a == 1);
    assert_throws!(
        one_assert::assert!(a == 2),
        "assertion `a == 2` failed
     left: 1
    right: 2"
    );

    one_assert::assert!(a != 2);
    assert_throws!(
        one_assert::assert!(a != 1),
        "assertion `a != 1` failed
     left: 1
    right: 1"
    );

    one_assert::assert!(a < 2);
    assert_throws!(
        one_assert::assert!(a < 1),
        "assertion `a < 1` failed
     left: 1
    right: 1"
    );

    one_assert::assert!(a <= 1);
    assert_throws!(
        one_assert::assert!(a <= 0),
        "assertion `a <= 0` failed
     left: 1
    right: 0"
    );

    one_assert::assert!(a > 0);
    assert_throws!(
        one_assert::assert!(a > 1),
        "assertion `a > 1` failed
     left: 1
    right: 1"
    );

    one_assert::assert!(a >= 1);
    assert_throws!(
        one_assert::assert!(a >= 2),
        "assertion `a >= 2` failed
     left: 1
    right: 2"
    );

    let b = true;
    one_assert::assert!(b && true);
    assert_throws!(
        one_assert::assert!(b && false),
        "assertion `b && false` failed
     left: true
    right: false"
    );

    one_assert::assert!(b & true);
    assert_throws!(
        one_assert::assert!(b & false),
        "assertion `b & false` failed
     left: true
    right: false"
    );

    let b = false;
    one_assert::assert!(b || true);
    assert_throws!(
        one_assert::assert!(b || false),
        "assertion `b || false` failed
     left: false
    right: false"
    );

    one_assert::assert!(b | true);
    assert_throws!(
        one_assert::assert!(b | false),
        "assertion `b | false` failed
     left: false
    right: false"
    );

    macro_rules! test_op_to_bool {
        ($op:tt, $op_name:ident, $op_fn_name:ident) => {{
            #[derive(Debug)]
            struct OpToBool(i32);
            impl std::ops::$op_name for OpToBool {
                type Output = bool;
                fn $op_fn_name(self, rhs: Self) -> bool {
                    self.0 == rhs.0
                }
            }

            let a = OpToBool(1);
            one_assert::assert!(a $op OpToBool(1));

            let a = OpToBool(1);
            assert_throws!(
                one_assert::assert!(a $op OpToBool(2)),
                concat!(
                    "assertion `a ", stringify!($op), " OpToBool(2)` failed
     left: OpToBool(1)
    right: OpToBool(2)"
                )
            );
        }};
    }
    test_op_to_bool!(+, Add, add);
    test_op_to_bool!(-, Sub, sub);
    test_op_to_bool!(*, Mul, mul);
    test_op_to_bool!(/, Div, div);
    test_op_to_bool!(%, Rem, rem);
    test_op_to_bool!(&, BitAnd, bitand);
    test_op_to_bool!(|, BitOr, bitor);
    test_op_to_bool!(^, BitXor, bitxor);
    test_op_to_bool!(<<, Shl, shl);
    test_op_to_bool!(>>, Shr, shr);
}

#[test]
fn test_block() {
    one_assert::assert!({
        let a = 1;
        a == 1
    });

    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            one_assert::assert!({
                let a = 1;
                a == 2
            }),
            "assertion `{ let a = 1 ; a == 2 }` failed
  caused by: block return assertion `a == 2` failed
     left: 1
    right: 2"
        );
    } else {
        assert_throws!(
            one_assert::assert!({
                let a = 1;
                a == 2
            }),
            "assertion `{ let a = 1; a == 2 }` failed
  caused by: block return assertion `a == 2` failed
     left: 1
    right: 2"
        );
    }
}

// #[test]
// fn test_break() {}

#[test]
fn test_call() {
    fn dummy_fn(a0: bool, a1: u8, a2: &str) -> bool {
        a0 && a1 == 1 && a2 == "hello"
    }

    let a = true;
    let b = 1;
    let c = "hello";

    one_assert::assert!(dummy_fn(a, b, c));

    let c = "world";
    assert_throws!(
        one_assert::assert!(dummy_fn(a, b, c)),
        "assertion `dummy_fn(a, b, c)` failed
    arg 0: true
    arg 1: 1
    arg 2: \"world\""
    );

    fn ten_arg_fn(a0: u8, a1: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8) -> bool {
        a0 == a1
    }

    let a = 1;
    let b = 2;
    assert_throws!(
        one_assert::assert!(ten_arg_fn(a, b, 0, 0, 0, 0, 0, 0, 0, 0)),
        "assertion `ten_arg_fn(a, b, 0, 0, 0, 0, 0, 0, 0, 0)` failed
    arg 0: 1
    arg 1: 2
    arg 2: 0
    arg 3: 0
    arg 4: 0
    arg 5: 0
    arg 6: 0
    arg 7: 0
    arg 8: 0
    arg 9: 0"
    );

    #[rustfmt::skip]
    fn eleven_arg_fn(a0: u8, a1: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8) -> bool {
        a0 == a1
    }

    assert_throws!(
        one_assert::assert!(eleven_arg_fn(a, b, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
        "assertion `eleven_arg_fn(a, b, 0, 0, 0, 0, 0, 0, 0, 0, 0)` failed
    arg  0: 1
    arg  1: 2
    arg  2: 0
    arg  3: 0
    arg  4: 0
    arg  5: 0
    arg  6: 0
    arg  7: 0
    arg  8: 0
    arg  9: 0
    arg 10: 0"
    );

    fn simple_true_fn() -> bool {
        true
    }
    fn simple_false_fn() -> bool {
        false
    }
    fn echo_fn(a: bool) -> bool {
        a
    }
    fn curry_true() -> fn() -> bool {
        simple_true_fn
    }
    fn curry_false() -> fn() -> bool {
        simple_false_fn
    }
    fn curry_echo() -> fn(bool) -> bool {
        echo_fn
    }
    fn curry_return(f: fn() -> bool) -> fn() -> bool {
        f
    }

    one_assert::assert!(simple_true_fn());
    one_assert::assert!(curry_true()());
    one_assert::assert!(echo_fn(true));
    one_assert::assert!(curry_echo()(true));
    one_assert::assert!(curry_return(simple_true_fn)());

    assert_throws!(
        one_assert::assert!(simple_false_fn()),
        "assertion `simple_false_fn()` failed"
    );
    assert_throws!(
        one_assert::assert!(curry_false()()),
        "assertion `curry_false() ()` failed"
    );
    assert_throws!(
        one_assert::assert!(echo_fn(false)),
        "assertion `echo_fn(false)` failed
    arg 0: false"
    );
    assert_throws!(
        one_assert::assert!(curry_echo()(false)),
        "assertion `curry_echo() (false)` failed
    arg 0: false"
    );
    assert_throws!(
        one_assert::assert!(curry_return(simple_false_fn)()),
        "assertion `curry_return(simple_false_fn) ()` failed"
    ); // doesn't print args because the actual call is to `simple_false_fn`
}

#[test]
fn test_cast() {
    one_assert::assert!(true as bool);

    assert_throws!(
        one_assert::assert!(false as bool),
        "assertion `false as bool` failed"
    );
}

// #[test]
// fn test_closure() {}

#[test]
fn test_const() {
    one_assert::assert!(
        const {
            let a = 1;
            a == 1
        }
    );

    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            one_assert::assert!(
                const {
                    let a = 1;
                    a == 2
                }
            ),
            "assertion `const { let a = 1 ; a == 2 }` failed
  caused by: block return assertion `a == 2` failed
     left: 1
    right: 2"
        );
    } else {
        assert_throws!(
            one_assert::assert!(
                const {
                    let a = 1;
                    a == 2
                }
            ),
            "assertion `const { let a = 1; a == 2 }` failed
  caused by: block return assertion `a == 2` failed
     left: 1
    right: 2"
        );
    }
}

// #[test]
// fn test_continue() {}

#[test]
fn test_field() {
    struct Bob {
        valid: bool,
    }

    let bob = Bob { valid: true };
    one_assert::assert!(bob.valid);

    let unbob = Bob { valid: false };
    assert_throws!(
        one_assert::assert!(unbob.valid),
        "assertion `unbob.valid` failed"
    );
}

// #[test]
// fn test_forloop() {}

// #[test]
// fn test_group() {}

#[test]
fn test_if() {
    let x = 1;
    let y = 2;
    one_assert::assert!(if x == 1 { y == 2 } else { y == 3 });

    assert_throws!(
        one_assert::assert!(if x == 1 { false } else { y == 3 }),
        "assertion `if x == 1 { false } else { y == 3 }` failed
    condition `x == 1`: true
  caused by: block return assertion `false` failed"
    );

    assert_throws!(
        one_assert::assert!(if x == 2 { true } else { y == 3 }),
        "assertion `if x == 2 { true } else { y == 3 }` failed
    condition `x == 2`: false
  caused by: block return assertion `y == 3` failed
     left: 2
    right: 3"
    );

    assert_throws!(
        one_assert::assert!(if x == 0 {
            true
        } else if x == 1 {
            y == x
        } else if x == 2 {
            false
        } else {
            unreachable!()
        }),
        "assertion `if x == 0 { true } else if x == 1 { y == x } else if x == 2 { false } else
{ unreachable! () }` failed
    condition `x == 0`: false
    condition `x == 1`: true
  caused by: block return assertion `y == x` failed
     left: 2
    right: 1"
    );

    assert_throws!(
        one_assert::assert!(if x == 0 {
            true
        } else if x == 5 {
            y == x
        } else if false {
            true
        } else if x == 2 {
            false
        } else {
            if x == 1 {
                y == 3
            } else {
                false
            }
        }),
        "assertion `if x == 0 { true } else if x == 5 { y == x } else if false { true } else if x
== 2 { false } else { if x == 1 { y == 3 } else { false } }` failed
    condition `x == 0`: false
    condition `x == 5`: false
     condition `false`: false
    condition `x == 2`: false
  caused by: block return assertion `if x == 1 { y == 3 } else { false }` failed
    condition `x == 1`: true
  caused by: block return assertion `y == 3` failed
     left: 2
    right: 3"
    );
}

#[test]
fn test_index() {
    let arr = [true, false, false];
    let idx = 0;
    one_assert::assert!(arr[idx]);

    let idx = 1;
    assert_throws!(
        one_assert::assert!(arr[idx]),
        "assertion `arr [idx]` failed
    index: 1"
    );

    assert_throws!(one_assert::assert!(arr[2]), "assertion `arr [2]` failed");

    let map = std::collections::HashMap::<&str, bool>::from_iter([("a", true), ("b", false)]);

    let true_key = "a";
    one_assert::assert!(map[true_key]);

    let false_key = "b";
    assert_throws!(
        one_assert::assert!(map[false_key]),
        r#"assertion `map [false_key]` failed
    index: "b""#
    );
}

// #[test]
// fn test_infer() {}

// #[test]
// fn test_let() {}

#[test]
fn test_lit() {
    assert_throws!(
        one_assert::assert!(false),
        "surprisingly, `false` did not evaluate to true"
    );
}

#[test]
fn test_loop() {
    one_assert::assert!(loop {
        break true;
    });

    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            one_assert::assert!(loop {
                break false;
            }),
            "assertion `loop { break false ; }` failed"
        );
    } else {
        assert_throws!(
            one_assert::assert!(loop {
                break false;
            }),
            "assertion `loop { break false; }` failed"
        );
    }
}

#[test]
fn test_macro() {
    one_assert::assert!(dbg!(true));

    assert_throws!(
        one_assert::assert!(dbg!(false)),
        "assertion `dbg! (false)` failed"
    );
}

#[test]
fn test_match() {
    let x = 1;
    let y = 2;
    let z = 3;
    one_assert::assert!(match (x, y) {
        (2, _) => true,
        (_, 2) => z == 3,
        _ => false,
    });

    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            one_assert::assert!(match (x, y) {
                (2, _) => true,
                (_, 2) => z == 5,
                _ => false,
            }),
            "assertion `match(x, y) { (2, _) => true, (_, 2) => z == 5, _ => false, }` failed
    matched value: (1, 2)
  caused by: match (x, y) entered arm `(_, 2)` where assertion `z == 5` failed
     left: 3
    right: 5"
        );

        assert_throws!(
            one_assert::assert!(match x {
                2 => true,
                _ if y < 5 => {
                    let w = 4;
                    z == w
                }
                _ => false,
            }),
            "assertion `match x { 2 => true, _ if y < 5 => { let w = 4 ; z == w } _ => false, }` failed
    matched value: 1
  caused by: match x entered arm `_ if y < 5` where assertion `{ let w = 4 ; z == w }` failed
  caused by: block return assertion `z == w` failed
     left: 3
    right: 4"
        );
    } else {
        assert_throws!(
            one_assert::assert!(match (x, y) {
                (2, _) => true,
                (_, 2) => z == 5,
                _ => false,
            }),
            "assertion `match (x, y) { (2, _) => true, (_, 2) => z == 5, _ => false, }` failed
    matched value: (1, 2)
  caused by: match (x, y) entered arm `(_, 2)` where assertion `z == 5` failed
     left: 3
    right: 5"
        );

        assert_throws!(
            one_assert::assert!(match x {
                2 => true,
                _ if y < 5 => {
                    let w = 4;
                    z == w
                }
                _ => false,
            }),
            "assertion `match x { 2 => true, _ if y < 5 => { let w = 4; z == w } _ => false, }` failed
    matched value: 1
  caused by: match x entered arm `_ if y < 5` where assertion `{ let w = 4; z == w }` failed
  caused by: block return assertion `z == w` failed
     left: 3
    right: 4"
        );
    }
}

#[test]
fn test_methodcall() {
    let s = String::from("hello");
    one_assert::assert!(s.contains("ell"));

    assert_throws!(
        one_assert::assert!(s.contains("world")),
        r#"assertion `s.contains("world")` failed
    object: "hello"
    method: "contains"
     arg 0: "world""#
    );
}

#[test]
fn test_paren() {
    one_assert::assert!((true));

    assert_throws!(one_assert::assert!((false)), "assertion `(false)` failed");
}

#[test]
fn test_path() {
    let x = true;
    one_assert::assert!(x);

    let x = false;
    assert_throws!(one_assert::assert!(x), "assertion `x` failed");

    mod foo {
        pub mod bar {
            pub const TRUE: bool = true;
            pub const FALSE: bool = false;
        }

        pub struct Generic<const N: isize>;
        impl<const N: isize> Generic<N> {
            pub const IS_POSITIVE: bool = N > 0;
        }
    }

    one_assert::assert!(foo::bar::TRUE);

    assert_throws!(
        one_assert::assert!(foo::bar::FALSE),
        "assertion `foo :: bar :: FALSE` failed"
    );

    one_assert::assert!(foo::Generic::<1>::IS_POSITIVE);

    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            one_assert::assert!(foo::Generic::<-1>::IS_POSITIVE),
            "assertion `foo :: Generic :: < - 1 > :: IS_POSITIVE` failed"
        );
    } else {
        assert_throws!(
            one_assert::assert!(foo::Generic::<-1>::IS_POSITIVE),
            "assertion `foo :: Generic :: < -1 > :: IS_POSITIVE` failed"
        );
    }
}

// #[test]
// fn test_range() {}

// #[test]
// fn test_reference() {}

// #[test]
// fn test_repeat() {}

// #[test]
// fn test_return() {}

// #[test]
// fn test_struct() {}

#[test]
fn test_try() {
    fn fallible_fn() -> Result<(), ()> {
        let x = Ok(true);
        one_assert::assert!(x?);

        Ok(())
    }
    fallible_fn().unwrap();

    assert_throws!(
        (|| -> Result<(), ()> {
            let x = Ok(false);
            one_assert::assert!(x?);
            Ok(())
        })()
        .unwrap(),
        "assertion `x ?` failed"
    );
}

// #[test]
// fn test_tuple() {}

#[test]
fn test_unary() {
    {
        #[derive(Debug)]
        struct OpToBool(bool);
        impl std::ops::Not for OpToBool {
            type Output = bool;
            fn not(self) -> bool {
                self.0
            }
        }

        let a = OpToBool(true);
        one_assert::assert!(!a);

        let b = OpToBool(false);
        assert_throws!(
            one_assert::assert!(!b),
            concat!(
                "assertion `! b` failed
    assertion negated: true"
            )
        );
    }

    {
        #[derive(Debug)]
        struct OpToBool(bool);
        impl std::ops::Neg for OpToBool {
            type Output = bool;
            fn neg(self) -> bool {
                self.0
            }
        }

        let a = OpToBool(true);
        one_assert::assert!(-a);

        let b = OpToBool(false);
        assert_throws!(
            one_assert::assert!(-b),
            concat!(
                "assertion `- b` failed
    original: OpToBool(false)"
            )
        );
    }

    {
        #[derive(Debug)]
        struct OpToBool(bool);
        impl std::ops::Deref for OpToBool {
            type Target = bool;
            fn deref(&self) -> &bool {
                &self.0
            }
        }

        let a = OpToBool(true);
        one_assert::assert!(*a);

        let b = OpToBool(false);
        assert_throws!(
            one_assert::assert!(*b),
            "assertion `* b` failed
    original: OpToBool(false)"
        );
    }
}

#[test]
fn test_unsafe() {
    one_assert::assert!(unsafe { std::mem::transmute(1u8) });

    assert_throws!(
        one_assert::assert!(unsafe { std::mem::transmute(0u8) }),
        "assertion `unsafe { std :: mem :: transmute(0u8) }` failed
  caused by: block return assertion `std :: mem :: transmute(0u8)` failed
    arg 0: 0"
    );
}

// #[test]
// fn test_verbatim() {}

// #[test]
// fn test_while() {}

// Experimental syntax:
// test_tryblock
// test_yield
