//! Behaviour tests for semantic identifier classification.

use crate::java::{tokenize_line, LexState, TokenKind};

fn kinds(source: &str) -> Vec<(TokenKind, &str)> {
    tokenize_line(source, LexState::Normal)
        .tokens
        .into_iter()
        .map(|token| (token.kind, &source[token.start..token.end]))
        .collect()
}

#[test]
fn declaration_names_and_type_usages_are_types() {
    let found = kinds("public class Main implements Runnable { List<String> names; }");

    assert!(found.contains(&(TokenKind::Type, "Main")));
    assert!(found.contains(&(TokenKind::Type, "Runnable")));
    assert!(found.contains(&(TokenKind::Type, "List")));
    assert!(found.contains(&(TokenKind::Type, "String")));
    assert!(found.contains(&(TokenKind::Identifier, "names")));
}

#[test]
fn methods_are_distinguished_from_fields_and_locals() {
    let found = kinds("void run() { service.execute(value); }");

    assert!(found.contains(&(TokenKind::Method, "run")));
    assert!(found.contains(&(TokenKind::Method, "execute")));
    assert!(found.contains(&(TokenKind::Identifier, "service")));
    assert!(found.contains(&(TokenKind::Identifier, "value")));
}

#[test]
fn member_access_without_a_call_is_a_field() {
    let found = kinds("return account.balance;");

    assert!(found.contains(&(TokenKind::Field, "balance")));
    assert!(found.contains(&(TokenKind::Identifier, "account")));
}

#[test]
fn constants_and_enum_constants_are_recognised() {
    let found = kinds("static final int MAX_SIZE = 10; Color.RED");

    assert!(found.contains(&(TokenKind::Constant, "MAX_SIZE")));
    assert!(found.contains(&(TokenKind::Constant, "RED")));
}

#[test]
fn constructors_and_acronym_types_stay_types() {
    let found = kinds("URL url = new URL(path);");

    assert!(found.contains(&(TokenKind::Type, "URL")));
    assert!(found.contains(&(TokenKind::Identifier, "url")));
}

#[test]
fn method_references_are_methods() {
    let found = kinds("names.forEach(System.out::println);");

    assert!(found.contains(&(TokenKind::Method, "forEach")));
    assert!(found.contains(&(TokenKind::Type, "System")));
    assert!(found.contains(&(TokenKind::Field, "out")));
    assert!(found.contains(&(TokenKind::Method, "println")));
}

#[test]
fn import_paths_do_not_turn_package_segments_into_fields() {
    let found = kinds("import java.util.List;");

    assert!(found.contains(&(TokenKind::Identifier, "java")));
    assert!(found.contains(&(TokenKind::Identifier, "util")));
    assert!(found.contains(&(TokenKind::Type, "List")));
}

#[test]
fn single_letter_type_parameters_are_not_constants() {
    let found = kinds("class Box<T> { T value; }");

    assert!(found.contains(&(TokenKind::Type, "Box")));
    assert!(found.contains(&(TokenKind::Type, "T")));
    assert!(found.contains(&(TokenKind::Identifier, "value")));
}

#[test]
fn arrays_varargs_and_class_literals_are_types() {
    let found = kinds("URL[] urls = new URL[4]; String... names; String.class");

    assert!(found.contains(&(TokenKind::Type, "URL")));
    assert!(found.contains(&(TokenKind::Type, "String")));
    assert!(found.contains(&(TokenKind::Identifier, "urls")));
    assert!(found.contains(&(TokenKind::Identifier, "names")));
}

#[test]
fn a_lowercase_expression_before_class_stays_an_identifier() {
    let found = kinds("object.class");

    assert!(found.contains(&(TokenKind::Identifier, "object")));
}

#[test]
fn comments_do_not_break_member_context() {
    let found = kinds("object /* note */ .value");

    assert!(found.contains(&(TokenKind::Field, "value")));
}
