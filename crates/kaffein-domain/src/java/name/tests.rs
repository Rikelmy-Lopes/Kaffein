//! Tests for source identity extraction.

use super::{package_name, primary_type_name};

#[test]
fn finds_the_first_type_name() {
    assert_eq!(
        primary_type_name("public class Main { public static void main(String[] a) {} }"),
        Some("Main")
    );
    assert_eq!(primary_type_name("interface Service {}"), Some("Service"));
    assert_eq!(primary_type_name("enum Color { RED }"), Some("Color"));
    assert_eq!(
        primary_type_name("record Point(int x, int y) {}"),
        Some("Point")
    );
}

#[test]
fn prefers_a_public_type_over_an_earlier_package_private_type() {
    let source = "class Helper {}\npublic class Main {}\n";
    assert_eq!(primary_type_name(source), Some("Main"));
}

#[test]
fn ignores_type_names_inside_comments_and_strings() {
    let source = "// class Fake\nString text = \"class Other\";\npublic class Real {}\n";
    assert_eq!(primary_type_name(source), Some("Real"));
}

#[test]
fn handles_annotation_type_declarations() {
    assert_eq!(
        primary_type_name("public @interface Marker {}"),
        Some("Marker")
    );
}

#[test]
fn reads_a_package_declaration() {
    assert_eq!(
        package_name("package com.example.app;\npublic class Main {}\n"),
        Some("com.example.app".to_owned())
    );
    assert_eq!(package_name("public class Main {}\n"), None);
    assert_eq!(
        package_name("package Com.Example.App;\npublic class Main {}\n"),
        Some("Com.Example.App".to_owned())
    );
}

#[test]
fn ignores_package_words_inside_comments() {
    assert_eq!(
        package_name("// package fake.name;\npackage real.name;\nclass Main {}\n"),
        Some("real.name".to_owned())
    );
}
