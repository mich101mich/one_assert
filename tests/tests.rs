macro_rules! assert_throws {
    ( $block:block, $message:literal $(,)? ) => {
        let error = std::panic::catch_unwind(move || $block).unwrap_err();
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
} // kinda ironic that the crate all about only having one `assert!` macro has a different one here

#[test]
fn test_assert() {
    let x = 1;
    assert_throws!(assert!(x == 2), "assertion failed: x == 2",);
}

#[test]
fn test_assert_eq() {
    let x = 1;
    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            assert_eq!(x, 2),
            "assertion failed: `(left == right)`
  left: `1`,
 right: `2`",
        );
    } else {
        assert_throws!(
            assert_eq!(x, 2),
            "assertion `left == right` failed
  left: 1
 right: 2",
        );
    }
}

#[test]
fn test_assert_message() {
    let x = 1;
    assert_throws!(
        assert!(x == 2, "x={}", x),
        "x=1", // really? This doesn't even print the condition?
    );
}

#[test]
fn test_assert_eq_message() {
    let x = 1;
    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            assert_eq!(x, 2, "x={}", x),
            "assertion failed: `(left == right)`
  left: `1`,
 right: `2`: x=1",
        );
    } else {
        assert_throws!(
            assert_eq!(x, 2, "x={}", x),
            "assertion `left == right` failed: x=1
  left: 1
 right: 2",
        );
    }
}

#[test]
fn test_one_assert() {
    let x = 1;
    assert_throws!(
        one_assert::assert!(x == 2),
        "assertion `x == 2` failed
     left: 1
    right: 2",
    );

    let x = true;
    assert_throws!(
        one_assert::assert!(x && false),
        "assertion `x && false` failed
     left: true
    right: false",
    );
}

#[test]
fn test_one_assert_message() {
    let x = 1;
    assert_throws!(
        one_assert::assert!(x == 2, "x={}", x),
        "assertion `x == 2` failed: x=1
     left: 1
    right: 2",
    );

    let x = true;
    assert_throws!(
        one_assert::assert!(x && false, "x={}", x),
        "assertion `x && false` failed: x=true
     left: true
    right: false",
    );
}

#[test]
fn test_misc() {
    one_assert::assert!(!"abc123".replace(|c: char| c.is_alphabetic(), "").is_empty());
}

#[test]
fn test_single_evaluation() {
    fn create_caller() -> impl FnMut() -> bool {
        let mut called = false;
        move || {
            assert!(!called);
            called = true;
            true
        }
    }

    let mut caller = create_caller();
    one_assert::assert!(caller());

    one_assert::assert!(create_caller()());

    let mut caller = create_caller();
    one_assert::assert!(caller() == true);

    let mut caller = create_caller();
    assert_throws!(
        one_assert::assert!(caller() == false),
        "assertion `caller() == false` failed
     left: true
    right: false",
    );
}

#[test]
fn test_crazy_nonsense() {
    #[derive(Debug)]
    struct AddsToBool(i32);
    impl std::ops::Add for AddsToBool {
        type Output = bool;
        fn add(self, rhs: Self) -> bool {
            self.0 == rhs.0
        }
    }
    let x = AddsToBool(1);
    one_assert::assert!(x + AddsToBool(1));

    let x = AddsToBool(1);
    assert_throws!(
        one_assert::assert!(x + AddsToBool(2)),
        "assertion `x + AddsToBool(2)` failed
     left: AddsToBool(1)
    right: AddsToBool(2)",
    );
}

#[test]
#[ignore]
fn error_message_tests() {
    let root = std::path::PathBuf::from("tests/fail");
    let base_paths = vec![root.clone(), root.join("expr")];

    // Error Messages are different in nightly => Different .stderr files
    let nightly = rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly;
    let channel = if nightly { "nightly" } else { "stable" };

    let mut paths = base_paths.clone();
    paths.extend(base_paths.iter().map(|p| p.join(channel)));

    let t = trybuild::TestCases::new();
    for mut path in paths {
        path.push("*.rs");
        t.compile_fail(path.display().to_string());
    }
}
