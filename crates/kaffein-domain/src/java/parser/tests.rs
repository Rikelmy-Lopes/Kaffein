//! Parser tests: declarations, imports, members, and local scopes.

use super::parse;
use crate::java::ast::{LocalKind, MemberKind, TypeKind};

#[test]
fn parses_package_and_imports() {
    let file = parse(
        "package com.example.app;\n\
         import java.util.List;\n\
         import java.util.*;\n\
         import static java.util.Collections.emptyList;\n\
         import static java.util.Collections.*;\n\
         class App {}\n",
    );

    assert_eq!(file.package.as_deref(), Some("com.example.app"));
    assert_eq!(file.imports.len(), 4);
    assert_eq!(file.imports[0].path, "java.util.List");
    assert!(!file.imports[0].is_wildcard);
    assert_eq!(file.imports[1].path, "java.util");
    assert!(file.imports[1].is_wildcard);
    assert!(file.imports[2].is_static);
    assert_eq!(file.imports[2].path, "java.util.Collections.emptyList");
    assert!(!file.imports[2].is_wildcard);
    assert!(file.imports[3].is_static);
    assert!(file.imports[3].is_wildcard);
}

#[test]
fn parses_a_class_with_fields_and_methods() {
    let file = parse(
        "package demo;\n\
         public class Person {\n\
             private static final int MAX = 3;\n\
             private String name;\n\
             public Person(String name) { this.name = name; }\n\
             public String name() { return name; }\n\
         }\n",
    );

    let ty = &file.types[0];
    assert_eq!(ty.name, "Person");
    assert_eq!(ty.kind, TypeKind::Class);
    assert_eq!(ty.members.len(), 4);
    assert!(ty
        .members
        .iter()
        .any(|member| member.kind == MemberKind::Constructor));
    let max = ty
        .members
        .iter()
        .find(|member| member.name == "MAX")
        .expect("MAX");
    assert_eq!(max.kind, MemberKind::Field);
    assert_eq!(max.type_name.as_deref(), Some("int"));
    assert!(max.is_static);
    assert!(max.is_final);
    let name = ty
        .members
        .iter()
        .find(|member| member.name == "name" && member.kind == MemberKind::Method)
        .expect("name method");
    assert_eq!(name.params.len(), 0);
}

#[test]
fn parses_parameters_with_arrays_and_generics() {
    let file = parse(
        "class Demo {\n\
             void run(Map<String, Integer> values, String[] names, int... rest) {}\n\
         }\n",
    );

    let method = &file.types[0].members[0];
    assert_eq!(method.name, "run");
    assert_eq!(method.type_name.as_deref(), Some("void"));
    assert_eq!(method.params.len(), 3);
    assert_eq!(method.params[0].name, "values");
    assert_eq!(method.params[0].type_name, "Map<String, Integer>");
    assert_eq!(method.params[1].type_name, "String[]");
    assert_eq!(method.params[2].type_name, "int[]");
}

#[test]
fn parses_interfaces_records_and_annotations() {
    let file = parse(
        "interface Shape { double area(); }\n\
         record Point(int x, int y) {}\n\
         @interface Marker {}\n",
    );

    assert_eq!(file.types.len(), 3);
    assert_eq!(file.types[0].kind, TypeKind::Interface);
    assert_eq!(file.types[0].members[0].name, "area");
    assert_eq!(file.types[1].kind, TypeKind::Record);
    assert_eq!(file.types[2].kind, TypeKind::Annotation);
}

#[test]
fn parses_enum_constants_and_members() {
    let file = parse(
        "enum Color {\n\
             RED, GREEN(2), BLUE { };\n\
             private final int code;\n\
             Color() { this(0); }\n\
             Color(int code) { this.code = code; }\n\
         }\n",
    );

    let ty = &file.types[0];
    assert_eq!(ty.kind, TypeKind::Enum);
    assert_eq!(ty.enum_constants, ["RED", "GREEN", "BLUE"]);
    assert!(ty.members.iter().any(|member| member.name == "code"));
    assert_eq!(
        ty.members
            .iter()
            .filter(|member| member.kind == MemberKind::Constructor)
            .count(),
        2
    );
}

#[test]
fn parses_nested_types_and_supertypes() {
    let file = parse(
        "class Outer extends Base implements Runnable, AutoCloseable {\n\
             static class Inner {}\n\
             enum Kind { A }\n\
         }\n",
    );

    let outer = &file.types[0];
    assert_eq!(outer.supertypes, ["Base", "Runnable", "AutoCloseable"]);
    assert_eq!(outer.nested.len(), 2);
    assert_eq!(outer.nested[0].name, "Inner");
    assert!(outer.nested[0].is_static);
    assert_eq!(outer.nested[1].kind, TypeKind::Enum);
}

#[test]
fn discovers_parameters_and_scoped_locals() {
    let file = parse(
        "class Demo {\n\
             int run(String input) {\n\
                 int total = 0;\n\
                 if (input.isEmpty()) {\n\
                     String note = input;\n\
                 }\n\
                 var copy = new StringBuilder();\n\
                 return total;\n\
             }\n\
         }\n",
    );

    let method = &file.types[0].members[0];
    assert_eq!(method.locals.len(), 4);
    let param = &method.locals[0];
    assert_eq!(param.kind, LocalKind::Parameter);
    assert_eq!(param.name, "input");
    assert_eq!(param.type_name, "String");
    let total = method
        .locals
        .iter()
        .find(|local| local.name == "total")
        .expect("total");
    assert_eq!(total.kind, LocalKind::Variable);
    assert_eq!(total.type_name, "int");
    let note = method
        .locals
        .iter()
        .find(|local| local.name == "note")
        .expect("note");
    assert!(note.scope_end < total.scope_end);
    let copy = method
        .locals
        .iter()
        .find(|local| local.name == "copy")
        .expect("copy");
    assert_eq!(copy.type_name, "StringBuilder");
}

#[test]
fn malformed_input_never_panics_and_keeps_known_declarations() {
    let file = parse(
        "class Broken {\n\
             void unfinished( {\n\
             int value = 1;\n\
         ",
    );

    assert_eq!(file.types.len(), 1);
    assert_eq!(file.types[0].name, "Broken");
    let empty = parse("");
    assert!(empty.types.is_empty());
    assert!(empty.package.is_none());
}
