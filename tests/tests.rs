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
} // kinda ironic that the crate all about only having one `assert!` macro has a different one here

#[test]
fn test_assert() {
    assert_throws!(
        {
            let x = 1;
            assert!(x == 2);
        },
        "assertion failed: x == 2",
    );
}

#[test]
fn test_assert_eq() {
    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            {
                let x = 1;
                assert_eq!(x, 2);
            },
            "assertion failed: `(left == right)`
  left: `1`,
 right: `2`",
        );
    } else {
        assert_throws!(
            {
                let x = 1;
                assert_eq!(x, 2);
            },
            "assertion `left == right` failed
  left: 1
 right: 2",
        );
    }
}

#[test]
fn test_assert_message() {
    assert_throws!(
        {
            let x = 1;
            assert!(x == 2, "x={}", x);
        },
        "x=1", // really? This doesn't even print the condition?
    );
}

#[test]
fn test_assert_eq_message() {
    if rustc_version::version().unwrap() < rustc_version::Version::new(1, 75, 0) {
        assert_throws!(
            {
                let x = 1;
                assert_eq!(x, 2, "x={}", x);
            },
            "assertion failed: `(left == right)`
  left: `1`,
 right: `2`: x=1",
        );
    } else {
        assert_throws!(
            {
                let x = 1;
                assert_eq!(x, 2, "x={}", x);
            },
            "assertion `left == right` failed: x=1
  left: 1
 right: 2",
        );
    }
}

#[test]
fn test_one_assert() {
    assert_throws!(
        {
            let x = 1;
            one_assert::assert!(x == 2);
        },
        "assertion `x == 2` failed
  left: 1
 right: 2",
    );

    assert_throws!(
        {
            let x = true;
            one_assert::assert!(x && false);
        },
        "assertion `x && false` failed
  left: true
 right: false",
    );
}

#[test]
#[ignore]
fn error_message_tests() {
    let root = std::path::PathBuf::from("tests/fail");
    let mut paths = vec![root.clone(), root.join("expr")];

    // Error Messages are different in nightly => Different .stderr files
    let nightly = rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly;
    let channel = if nightly { "nightly" } else { "stable" };
    paths.push(root.join(channel));

    let t = trybuild::TestCases::new();
    for mut path in paths {
        path.push("*.rs");
        t.compile_fail(path.display().to_string());
    }
}
