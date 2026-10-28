//! Tests for the JDK adapter.

use super::{read_limited, StdJavaRunner};
use kaffein_domain::{JavaRunStage, JavaRunner};

#[test]
fn output_is_capped_and_marked_when_truncated() {
    let input = std::io::Cursor::new(b"abcdef".to_vec());
    let (text, truncated) = read_limited(input, 3);

    assert_eq!(text, "abc");
    assert!(truncated);
}

#[test]
fn output_below_the_cap_is_not_marked() {
    let input = std::io::Cursor::new(b"abc".to_vec());
    let (text, truncated) = read_limited(input, 8);

    assert_eq!(text, "abc");
    assert!(!truncated);
}

#[test]
#[ignore = "requires javac and java on PATH"]
fn compiles_and_runs_a_source_string() {
    let source = "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"hello from kaffein\");\n    }\n}\n";
    let output = StdJavaRunner::new()
        .run_source(source)
        .expect("the JDK should run the source");

    assert_eq!(output.stage, JavaRunStage::Run);
    assert_eq!(output.exit_code, Some(0));
    assert!(output.stdout.contains("hello from kaffein"), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
#[ignore = "requires javac and java on PATH"]
fn compiles_and_runs_a_package_declaration() {
    let source = "package com.example.app;\n\npublic class Main {\n    public static void main(String[] args) {\n        System.out.println(\"packaged\");\n    }\n}\n";
    let output = StdJavaRunner::new()
        .run_source(source)
        .expect("the JDK should run the packaged source");

    assert_eq!(output.stage, JavaRunStage::Run);
    assert_eq!(output.exit_code, Some(0));
    assert!(output.stdout.contains("packaged"), "{output:?}");
}
