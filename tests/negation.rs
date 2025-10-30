#![allow(clippy::nonminimal_bool)]

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
// fn test_negated_array() {}

// #[test]
// fn test_negated_assign() {}

// #[test]
// fn test_negated_async() {}

#[test]
fn test_negated_await() {
    use std::task::*;
    const DUMMY_FN: fn(*const ()) = |_: *const ()| {};
    static CREATE: fn() -> RawWaker = || RawWaker::new(&() as _, &VTABLE);
    static VTABLE: RawWakerVTable = RawWakerVTable::new(|_| CREATE(), DUMMY_FN, DUMMY_FN, DUMMY_FN);
    let waker = unsafe { Waker::from_raw(CREATE()) };
    let mut cx = Context::from_waker(&waker);

    let false_fut = async { false };
    let expr = std::pin::pin!(async move {
        one_assert::assert!(!false_fut.await);
    });
    assert_eq!(std::future::Future::poll(expr, &mut cx), Poll::Ready(()));

    let true_fut = async { true };
    assert_throws!(
        {
            let mut cx = Context::from_waker(&waker);
            let expr = std::pin::pin!(async move {
                one_assert::assert!(!true_fut.await);
            });
            let _ = std::future::Future::poll(expr, &mut cx);
        },
        "assertion `! true_fut.await` failed
  caused by: negated expression `true_fut.await` evaluated to true"
    );
}

#[test]
fn test_negated_binary() {
    let a = 1;

    one_assert::assert!(!(a == 2));

    assert_throws!(
        one_assert::assert!(!(a == 1)),
        "assertion `! (a == 1)` failed
  caused by: negated expression `a == 1` evaluated to true
     left: 1
    right: 1"
    );

    one_assert::assert!(!(a != 1));
    assert_throws!(
        one_assert::assert!(!(a != 2)),
        "assertion `! (a != 2)` failed
  caused by: negated expression `a != 2` evaluated to true
     left: 1
    right: 2"
    );

    one_assert::assert!(!(a < 1));
    assert_throws!(
        one_assert::assert!(!(a < 2)),
        "assertion `! (a < 2)` failed
  caused by: negated expression `a < 2` evaluated to true
     left: 1
    right: 2"
    );

    one_assert::assert!(!(a <= 0));
    assert_throws!(
        one_assert::assert!(!(a <= 1)),
        "assertion `! (a <= 1)` failed
  caused by: negated expression `a <= 1` evaluated to true
     left: 1
    right: 1"
    );

    one_assert::assert!(!(a > 1));
    assert_throws!(
        one_assert::assert!(!(a > 0)),
        "assertion `! (a > 0)` failed
  caused by: negated expression `a > 0` evaluated to true
     left: 1
    right: 0"
    );

    one_assert::assert!(!(a >= 2));
    assert_throws!(
        one_assert::assert!(!(a >= 1)),
        "assertion `! (a >= 1)` failed
  caused by: negated expression `a >= 1` evaluated to true
     left: 1
    right: 1"
    );

    let b = true;
    one_assert::assert!(!(b && false));
    assert_throws!(
        one_assert::assert!(!(b && true)),
        "assertion `! (b && true)` failed
  caused by: negated expression `b && true` evaluated to true
  caused by: both sides of `&&` evaluated to true"
    );

    one_assert::assert!(!(b & false));
    assert_throws!(
        one_assert::assert!(!(b & true)),
        "assertion `! (b & true)` failed
  caused by: negated expression `b & true` evaluated to true
     left: true
    right: true"
    );

    let b = false;
    one_assert::assert!(!(b || false));
    assert_throws!(
        one_assert::assert!(!(b || true)),
        "assertion `! (b || true)` failed
  caused by: negated expression `b || true` evaluated to true
  caused by: left side of `||` evaluated to false, but right side evaluated to true"
    );

    one_assert::assert!(!(b | false));
    assert_throws!(
        one_assert::assert!(!(b | true)),
        "assertion `! (b | true)` failed
  caused by: negated expression `b | true` evaluated to true
     left: false
    right: true"
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
            one_assert::assert!(!(a $op OpToBool(2)));

            let a = OpToBool(1);
            assert_throws!(
                one_assert::assert!(!(a $op OpToBool(1))),
                concat!(
                    "assertion `! (a ", stringify!($op), " OpToBool(1))` failed
  caused by: negated expression `a ", stringify!($op), " OpToBool(1)` evaluated to true
     left: OpToBool(1)
    right: OpToBool(1)"
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
fn test_negated_block() {
    one_assert::assert!(!{
        let a = 1;
        a == 2
    });

    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            one_assert::assert!(!{
                let a = 1;
                a == 1
            }),
            "assertion `! { let a = 1 ; a == 1 }` failed
  caused by: negated expression `{ let a = 1 ; a == 1 }` evaluated to true"
        );
    } else {
        assert_throws!(
            one_assert::assert!(!{
                let a = 1;
                a == 1
            }),
            "assertion `! { let a = 1; a == 1 }` failed
  caused by: negated expression `{ let a = 1; a == 1 }` evaluated to true"
        );
    }
}

// #[test]
// fn test_negated_break() {}

#[test]
fn test_negated_call() {
    fn dummy_fn(a0: bool, a1: u8, a2: &str) -> bool {
        a0 && a1 == 1 && a2 == "hello"
    }

    let a = true;
    let b = 1;
    let c = "world";

    one_assert::assert!(!dummy_fn(a, b, c));

    let c = "hello";
    assert_throws!(
        one_assert::assert!(!dummy_fn(a, b, c)),
        "assertion `! dummy_fn(a, b, c)` failed
  caused by: negated expression `dummy_fn(a, b, c)` evaluated to true
    arg 0: true
    arg 1: 1
    arg 2: \"hello\""
    );

    #[allow(clippy::too_many_arguments)]
    fn ten_arg_fn(a0: u8, a1: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8) -> bool {
        a0 == a1
    }

    let a = 1;
    let b = 1;
    assert_throws!(
        one_assert::assert!(!ten_arg_fn(a, b, 0, 0, 0, 0, 0, 0, 0, 0)),
        "assertion `! ten_arg_fn(a, b, 0, 0, 0, 0, 0, 0, 0, 0)` failed
  caused by: negated expression `ten_arg_fn(a, b, 0, 0, 0, 0, 0, 0, 0, 0)` evaluated to true
    arg 0: 1
    arg 1: 1
    arg 2: 0
    arg 3: 0
    arg 4: 0
    arg 5: 0
    arg 6: 0
    arg 7: 0
    arg 8: 0
    arg 9: 0"
    );

    #[allow(clippy::too_many_arguments)]
    #[rustfmt::skip]
    fn eleven_arg_fn(a0: u8, a1: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8, _: u8) -> bool {
        a0 == a1
    }

    assert_throws!(
        one_assert::assert!(!eleven_arg_fn(a, b, 0, 0, 0, 0, 0, 0, 0, 0, 0)),
        "assertion `! eleven_arg_fn(a, b, 0, 0, 0, 0, 0, 0, 0, 0, 0)` failed
  caused by: negated expression `eleven_arg_fn(a, b, 0, 0, 0, 0, 0, 0, 0, 0, 0)` evaluated to true
    arg  0: 1
    arg  1: 1
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

    one_assert::assert!(!simple_false_fn());
    one_assert::assert!(!curry_false()());
    one_assert::assert!(!echo_fn(false));
    one_assert::assert!(!curry_echo()(false));
    one_assert::assert!(!curry_return(simple_false_fn)());

    assert_throws!(
        one_assert::assert!(!simple_true_fn()),
        "assertion `! simple_true_fn()` failed
  caused by: negated expression `simple_true_fn()` evaluated to true"
    );
    assert_throws!(
        one_assert::assert!(!curry_true()()),
        "assertion `! curry_true() ()` failed
  caused by: negated expression `curry_true() ()` evaluated to true"
    );
    assert_throws!(
        one_assert::assert!(!echo_fn(true)),
        "assertion `! echo_fn(true)` failed
  caused by: negated expression `echo_fn(true)` evaluated to true
    arg 0: true"
    );
    assert_throws!(
        one_assert::assert!(!curry_echo()(true)),
        "assertion `! curry_echo() (true)` failed
  caused by: negated expression `curry_echo() (true)` evaluated to true
    arg 0: true"
    );
    assert_throws!(
        one_assert::assert!(!curry_return(simple_true_fn)()),
        "assertion `! curry_return(simple_true_fn) ()` failed
  caused by: negated expression `curry_return(simple_true_fn) ()` evaluated to true"
    ); // doesn't print args because the actual call is to `simple_true_fn`
}

#[test]
#[allow(clippy::unnecessary_cast)]
fn test_negated_cast() {
    one_assert::assert!(!(false as bool));

    assert_throws!(
        one_assert::assert!(!(true as bool)),
        "assertion `! (true as bool)` failed
  caused by: negated expression `true as bool` evaluated to true"
    );
}

// #[test]
// fn test_negated_closure() {}

// NOTE: inline consts are only stable since Rust 1.79, which is after the current MSRV of 1.70
// #[test]
// fn test_negated_const() {
//     one_assert::assert!(
//         !const {
//             let a = 1;
//             a == 2
//         }
//     );

//     if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
//         assert_throws!(
//             one_assert::assert!(
//                 !const {
//                     let a = 1;
//                     a == 1
//                 }
//             ),
//             "assertion `! const { let a = 1 ; a == 1 }` failed
//   caused by: negated expression `const { let a = 1 ; a == 1 }` evaluated to true"
//         );
//     } else {
//         assert_throws!(
//             one_assert::assert!(
//                 !const {
//                     let a = 1;
//                     a == 1
//                 }
//             ),
//             "assertion `! const { let a = 1; a == 1 }` failed
//   caused by: negated expression `const { let a = 1; a == 1 }` evaluated to true"
//         );
//     }
// }

// #[test]
// fn test_negated_continue() {}

#[test]
fn test_negated_field() {
    struct Bob {
        valid: bool,
    }

    let bob = Bob { valid: false };
    one_assert::assert!(!bob.valid);

    let unbob = Bob { valid: true };
    assert_throws!(
        one_assert::assert!(!unbob.valid),
        "assertion `! unbob.valid` failed
  caused by: negated expression `unbob.valid` evaluated to true"
    );
}

// #[test]
// fn test_negated_forloop() {}

// #[test]
// fn test_negated_group() {}

#[test]
fn test_negated_if() {
    let x = 1;
    let y = 3;
    one_assert::assert!(!if x == 1 { y == 2 } else { y == 3 });

    assert_throws!(
        one_assert::assert!(!if x == 1 { true } else { y == 3 }),
        "assertion `! if x == 1 { true } else { y == 3 }` failed
  caused by: negated expression `if x == 1 { true } else { y == 3 }` evaluated to true"
    );

    assert_throws!(
        one_assert::assert!(!if x == 2 { true } else { y == 3 }),
        "assertion `! if x == 2 { true } else { y == 3 }` failed
  caused by: negated expression `if x == 2 { true } else { y == 3 }` evaluated to true"
    );

    assert_throws!(
        one_assert::assert!(!if x == 0 {
            true
        } else if x == 1 {
            y == x + 2
        } else if x == 2 {
            false
        } else {
            panic!() // using unreachable!() here causes rust-analyzer to complain, even though cargo doesn't
        }),
        "assertion `! if x == 0 { true } else if x == 1 { y == x + 2 } else if x == 2 { false }
else { panic! () }` failed
  caused by: negated expression `if x == 0 { true } else if x == 1 { y == x + 2 } else if x == 2 { false } else
{ panic! () }` evaluated to true"
    );

    assert_throws!(
        one_assert::assert!(!if x == 0 {
            true
        } else if x == 5 {
            y == x
        } else if false {
            true
        } else if x == 2 {
            false
        } else {
            !if x == 1 { !(y == 3) } else { false }
        }),
        "assertion `! if x == 0 { true } else if x == 5 { y == x } else if false { true } else if
x == 2 { false } else { ! if x == 1 { ! (y == 3) } else { false } }` failed
  caused by: negated expression `if x == 0 { true } else if x == 5 { y == x } else if false { true } else if x
== 2 { false } else { ! if x == 1 { ! (y == 3) } else { false } }` evaluated to true"
    );
}

#[test]
fn test_negated_index() {
    let arr = [true, false, false];
    let idx = 1;
    one_assert::assert!(!arr[idx]);

    let idx = 0;
    assert_throws!(
        one_assert::assert!(!arr[idx]),
        "assertion `! arr [idx]` failed
  caused by: negated expression `arr [idx]` evaluated to true
    index: 0"
    );

    assert_throws!(
        one_assert::assert!(!arr[0]),
        "assertion `! arr [0]` failed
  caused by: negated expression `arr [0]` evaluated to true"
    );

    let map = std::collections::HashMap::<&str, bool>::from_iter([("a", true), ("b", false)]);

    let false_key = "b";
    one_assert::assert!(!map[false_key]);

    let true_key = "a";
    assert_throws!(
        one_assert::assert!(!map[true_key]),
        r#"assertion `! map [true_key]` failed
  caused by: negated expression `map [true_key]` evaluated to true
    index: "a""#
    );
}

// #[test]
// fn test_negated_infer() {}

// #[test]
// fn test_negated_let() {}

#[test]
fn test_negated_lit() {
    one_assert::assert!(!false);

    assert_throws!(
        one_assert::assert!(!true),
        "assertion `! true` failed
  caused by: negated expression `true` evaluated to true"
    );
}

#[test]
#[allow(clippy::never_loop)]
fn test_negated_loop() {
    one_assert::assert!(!loop {
        break false;
    });

    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            one_assert::assert!(!loop {
                break true;
            }),
            "assertion `! loop { break true ; }` failed
  caused by: negated expression `loop { break true ; }` evaluated to true"
        );
    } else {
        assert_throws!(
            one_assert::assert!(!loop {
                break true;
            }),
            "assertion `! loop { break true; }` failed
  caused by: negated expression `loop { break true; }` evaluated to true"
        );
    }
}

#[test]
fn test_negated_macro() {
    one_assert::assert!(!dbg!(false));

    assert_throws!(
        one_assert::assert!(!dbg!(true)),
        "assertion `! dbg! (true)` failed
  caused by: negated expression `dbg! (true)` evaluated to true"
    );
}

#[test]
fn test_negated_match() {
    let x = 1;
    let y = 2;
    let z = 3;
    one_assert::assert!(!match (x, y) {
        (2, _) => true,
        (_, 2) => !(z == 3),
        _ => false,
    });

    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            one_assert::assert!(!match (x, y) {
                (2, _) => true,
                (_, 2) => !(z == 5),
                _ => false,
            }),
            "assertion `! match(x, y) { (2, _) => true, (_, 2) =>! (z == 5), _ => false, }` failed
  caused by: negated expression `match(x, y) { (2, _) => true, (_, 2) =>! (z == 5), _ => false, }` evaluated to true"
        );

        assert_throws!(
            one_assert::assert!(!match x {
                2 => true,
                _ if y < 5 => {
                    let w = 4;
                    z != w
                }
                _ => false,
            }),
            "assertion `! match x { 2 => true, _ if y < 5 => { let w = 4 ; z != w } _ => false, }` failed
  caused by: negated expression `match x { 2 => true, _ if y < 5 => { let w = 4 ; z != w } _ => false, }` evaluated to true"
        );
    } else {
        assert_throws!(
            one_assert::assert!(!match (x, y) {
                (2, _) => true,
                (_, 2) => ! (z == 5),
                _ => false,
            }),
            "assertion `! match (x, y) { (2, _) => true, (_, 2) => ! (z == 5), _ => false, }` failed
  caused by: negated expression `match (x, y) { (2, _) => true, (_, 2) => ! (z == 5), _ => false, }` evaluated to true"
        );

        assert_throws!(
            one_assert::assert!(! match x {
                2 => true,
                _ if y < 5 => {
                    let w = 4;
                    z != w
                }
                _ => false,
            }),
            "assertion `! match x { 2 => true, _ if y < 5 => { let w = 4; z != w } _ => false, }` failed
  caused by: negated expression `match x { 2 => true, _ if y < 5 => { let w = 4; z != w } _ => false, }` evaluated to true"
        );
    }
}

#[test]
fn test_negated_methodcall() {
    let s = String::from("hello");
    one_assert::assert!(!s.contains("world"));

    assert_throws!(
        one_assert::assert!(!s.contains("ell")),
        r#"assertion `! s.contains("ell")` failed
  caused by: negated expression `s.contains("ell")` evaluated to true
     self: "hello"
    arg 0: "ell""#
    );
}

#[test]
fn test_negated_paren() {
    one_assert::assert!(!(!true));

    assert_throws!(
        one_assert::assert!(!(!false)),
        "assertion `! (! false)` failed
  caused by: negated expression `! false` evaluated to true"
    );
}

#[test]
fn test_negated_path() {
    let x = false;
    one_assert::assert!(!x);

    let x = true;
    assert_throws!(
        one_assert::assert!(!x),
        "assertion `! x` failed
  caused by: negated expression `x` evaluated to true"
    );

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

    one_assert::assert!(!foo::bar::FALSE);

    assert_throws!(
        one_assert::assert!(!foo::bar::TRUE),
        "assertion `! foo :: bar :: TRUE` failed
  caused by: negated expression `foo :: bar :: TRUE` evaluated to true"
    );

    one_assert::assert!(!foo::Generic::<-1>::IS_POSITIVE);

    assert_throws!(
        one_assert::assert!(!foo::Generic::<3>::IS_POSITIVE),
        "assertion `! foo :: Generic :: < 3 > :: IS_POSITIVE` failed
  caused by: negated expression `foo :: Generic :: < 3 > :: IS_POSITIVE` evaluated to true"
    );
}

// #[test]
// fn test_negated_range() {}

// #[test]
// fn test_negated_reference() {}

// #[test]
// fn test_negated_repeat() {}

// #[test]
// fn test_negated_return() {}

// #[test]
// fn test_negated_struct() {}

#[test]
fn test_negated_try() {
    fn fallible_fn() -> Result<(), ()> {
        let x = Ok(false);
        one_assert::assert!(!x?);

        Ok(())
    }
    fallible_fn().unwrap();

    assert_throws!(
        (|| -> Result<(), ()> {
            let x = Ok(true);
            one_assert::assert!(!x?);
            Ok(())
        })()
        .unwrap(),
        "assertion `! x ?` failed
  caused by: negated expression `x ?` evaluated to true"
    );
}

// #[test]
// fn test_negated_tuple() {}

#[test]
#[allow(clippy::useless_concat)]
fn test_negated_unary() {
    {
        #[derive(Debug)]
        struct OpToBool(bool);
        impl std::ops::Not for OpToBool {
            type Output = bool;
            fn not(self) -> bool {
                self.0
            }
        }

        let a = OpToBool(false);
        one_assert::assert!(!!a);

        if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
            let b = OpToBool(true);
            assert_throws!(
                one_assert::assert!(!!b),
                concat!(
                    "assertion `!! b` failed
  caused by: negated expression `! b` evaluated to true"
                )
            );
        } else {
            let b = OpToBool(true);
            assert_throws!(
                one_assert::assert!(!!b),
                concat!(
                    "assertion `! ! b` failed
  caused by: negated expression `! b` evaluated to true"
                )
            );
        }
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

        let a = OpToBool(false);
        one_assert::assert!(!-a);

        let b = OpToBool(true);
        assert_throws!(
            one_assert::assert!(!-b),
            concat!(
                "assertion `! - b` failed
  caused by: negated expression `- b` evaluated to true"
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

        let a = OpToBool(false);
        one_assert::assert!(!*a);

        let b = OpToBool(true);
        assert_throws!(
            one_assert::assert!(!*b),
            "assertion `! * b` failed
  caused by: negated expression `* b` evaluated to true"
        );
    }
}

#[test]
#[allow(clippy::transmute_int_to_bool, clippy::missing_transmute_annotations)]
fn test_negated_unsafe() {
    one_assert::assert!(!unsafe { std::mem::transmute(0u8) });

    assert_throws!(
        one_assert::assert!(!unsafe { std::mem::transmute(1u8) }),
        "assertion `! unsafe { std :: mem :: transmute(1u8) }` failed
  caused by: negated expression `unsafe { std :: mem :: transmute(1u8) }` evaluated to true"
    );
}

// #[test]
// fn test_negated_verbatim() {}

// #[test]
// fn test_negated_while() {}

// Experimental syntax:
// test_tryblock
// test_yield
