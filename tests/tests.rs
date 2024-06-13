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
            let a = 1;
            let b = 2;
            assert!(a == b);
        },
        "assertion failed: a == b",
    );
}

#[test]
fn test_assert_eq() {
    assert_throws!(
        {
            let a = 1;
            let b = 2;
            assert_eq!(a, b);
        },
        "assertion `left == right` failed
  left: 1
 right: 2",
    );
}

#[test]
fn test_assert_message() {
    assert_throws!(
        {
            let a = 1;
            let b = 2;
            assert!(a == b, "a={} b={}", a, b);
        },
        "a=1 b=2", // really? This doesn't even print the condition?
    );
}

#[test]
fn test_assert_eq_message() {
    assert_throws!(
        {
            let a = 1;
            let b = 2;
            assert_eq!(a, b, "a={} b={}", a, b);
        },
        "assertion `left == right` failed: a=1 b=2
  left: 1
 right: 2",
    );
}

#[test]
fn test_one_assert() {
    assert_throws!(
        {
            let a = 1;
            let b = 2;
            one_assert::assert!(a == b);
        },
        "assertion `a == b` failed
 a: 1
 b: 2",
    );

    assert_throws!(
        {
            let a = true;
            let b = false;
            one_assert::assert!(a && b);
        },
        "assertion `a && b` failed
 a: true
 b: false",
    );
}

#[test]
#[ignore]
fn error_message_tests() {
    let root = std::path::PathBuf::from("tests/fail");
    let mut paths = vec![root.clone()];

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
