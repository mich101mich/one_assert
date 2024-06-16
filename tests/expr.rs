#[macro_export]
macro_rules! assert_throws {
    ( $block:block, $message:literal $(,)? ) => {
        let error = std::panic::catch_unwind(|| $block).unwrap_err();
        if let Some(s) = error.downcast_ref::<&'static str>() {
            assert_eq!(*s, $message);
        } else if let Some(s) = error.downcast_ref::<String>() {
            assert_eq!(s, $message);
        } else {
            panic!("unexpected panic payload: {:?}", error);
        }
    };
    ( $statement:stmt, $message:literal $(,)? ) => {
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
fn test_binary() {} // TODO: Implement

#[test]
fn test_block() {
    one_assert::assert!({
        let a = 1;
        a == 1
    });

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

// #[test]
// fn test_break() {}

#[test]
fn test_call() {} // TODO: Implement

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
        one_assert::assert!(if x == 2 { true } else { y == 3 }),
        "assertion `if x == 2 { true } else { y == 3 }` failed"
    );
}

#[test]
fn test_index() {}

#[test]
fn test_infer() {}

#[test]
fn test_let() {}

#[test]
fn test_lit() {}

#[test]
fn test_loop() {}

#[test]
fn test_macro() {}

#[test]
fn test_match() {}

#[test]
fn test_methodcall() {}

#[test]
fn test_paren() {}

#[test]
fn test_path() {}

#[test]
fn test_range() {}

#[test]
fn test_reference() {}

#[test]
fn test_repeat() {}

#[test]
fn test_return() {}

#[test]
fn test_struct() {}

#[test]
fn test_try() {}

#[test]
fn test_tryblock() {}

#[test]
fn test_tuple() {}

#[test]
fn test_unary() {}

#[test]
fn test_unsafe() {}

#[test]
fn test_verbatim() {}

#[test]
fn test_while() {}

#[test]
fn test_yield() {}
