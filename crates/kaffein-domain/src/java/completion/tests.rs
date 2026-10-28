//! Completion tests: locals, members, receivers, imports, and suppression.

use crate::java::completion::{complete, Completion, CompletionKind};
use crate::java::index::SymbolIndex;

const MARKER: &str = "<|>";

fn at(source: &str, index: &SymbolIndex) -> Completion {
    let caret = source.find(MARKER).expect("caret marker");
    let source = source.replace(MARKER, "");
    complete(&source, caret, index).expect("completion")
}

fn labels(completion: &Completion) -> Vec<&str> {
    completion
        .items
        .iter()
        .map(|item| item.label.as_str())
        .collect()
}

fn library() -> SymbolIndex {
    let mut index = SymbolIndex::new();
    index.upsert(
        "C:/lib/Person.java",
        "package demo;\n\
         public class Person {\n\
             public String name;\n\
             public Address address;\n\
             public String greet(String who) { return who; }\n\
             public static Person create() { return new Person(); }\n\
         }\n\
         class Address { public String city; }\n\
         class Util { public static final int MAX = 3; public static void help() {} }",
    );
    index.upsert(
        "C:/lib/Color.java",
        "package demo; public enum Color { RED, GREEN; public int code() { return 0; } }",
    );
    index
}

#[test]
fn completes_visible_locals_and_parameters() {
    let index = SymbolIndex::new();
    let completion = at(
        "class Demo {\n\
             void run(String input) {\n\
                 int total = 0;\n\
                 int other = total;\n\
                 tot<|>\n\
             }\n\
         }",
        &index,
    );

    let labels = labels(&completion);
    assert!(labels.contains(&"total"), "{labels:?}");
    assert!(!labels.contains(&"other"), "{labels:?}");
    assert!(completion.start < completion.end);
    let item = completion
        .items
        .iter()
        .find(|item| item.label == "total")
        .expect("total");
    assert_eq!(item.kind, CompletionKind::Variable);
    assert_eq!(item.insert_text, "total");
}

#[test]
fn completes_fields_and_methods_of_the_enclosing_type() {
    let index = SymbolIndex::new();
    let completion = at(
        "class Counter {\n\
             private int count;\n\
             void increment() {}\n\
             void run() { cou<|> }\n\
         }",
        &index,
    );

    let labels = labels(&completion);
    assert!(labels.contains(&"count"), "{labels:?}");
    let field = completion
        .items
        .iter()
        .find(|item| item.label == "count")
        .expect("count");
    assert_eq!(field.kind, CompletionKind::Field);
    assert_eq!(field.detail, "int count");
}

#[test]
fn completes_methods_of_the_enclosing_type() {
    let index = SymbolIndex::new();
    let completion = at(
        "class Counter {\n\
             private int count;\n\
             void increment() {}\n\
             void run() { inc<|> }\n\
         }",
        &index,
    );

    let labels = labels(&completion);
    assert!(labels.contains(&"increment"), "{labels:?}");
    let method = completion
        .items
        .iter()
        .find(|item| item.label == "increment")
        .expect("increment");
    assert_eq!(method.kind, CompletionKind::Method);
    assert!(method.detail.contains("void increment"));
}

#[test]
fn completes_enum_constants_of_the_enclosing_enum() {
    let index = SymbolIndex::new();
    let completion = at("enum Color { RED, GREEN, BLUE; R<|> }", &index);

    let labels = labels(&completion);
    assert!(labels.contains(&"RED"), "{labels:?}");
    assert!(!labels.contains(&"GREEN"), "{labels:?}");
    let red = completion
        .items
        .iter()
        .find(|item| item.label == "RED")
        .expect("RED");
    assert_eq!(red.kind, CompletionKind::EnumConstant);
}

#[test]
fn a_this_receiver_lists_instance_members() {
    let index = SymbolIndex::new();
    let completion = at(
        "class Counter {\n\
             int count;\n\
             void increment() {}\n\
             void run() { this.<|> }\n\
         }",
        &index,
    );

    let labels = labels(&completion);
    assert!(labels.contains(&"count"), "{labels:?}");
    assert!(labels.contains(&"increment"), "{labels:?}");
}

#[test]
fn a_local_receiver_resolves_its_declared_type() {
    let index = library();
    let completion = at(
        "package app;\n\
         import demo.Person;\n\
         class Main {\n\
             void run(Person p) { p.<|> }\n\
         }",
        &index,
    );

    let labels = labels(&completion);
    assert!(labels.contains(&"name"), "{labels:?}");
    assert!(labels.contains(&"greet"), "{labels:?}");
    assert!(labels.contains(&"address"), "{labels:?}");
    let greet = completion
        .items
        .iter()
        .find(|item| item.label == "greet")
        .expect("greet");
    assert!(greet.detail.contains("String greet"), "{greet:?}");
}

#[test]
fn a_field_receiver_resolves_through_the_field_type() {
    let index = library();
    let completion = at(
        "package app;\n\
         import demo.Person;\n\
         class Main {\n\
             Person p;\n\
             void run() { p.address.<|> }\n\
         }",
        &index,
    );

    let labels = labels(&completion);
    assert!(labels.contains(&"city"), "{labels:?}");
}

#[test]
fn a_type_receiver_lists_static_and_nested_members() {
    let index = library();
    let completion = at(
        "package app;\n\
         import demo.Color;\n\
         class Main {\n\
             void run() { Color.<|> }\n\
         }",
        &index,
    );

    let labels = labels(&completion);
    assert!(labels.contains(&"RED"), "{labels:?}");
    assert!(labels.contains(&"code"), "{labels:?}");
}

#[test]
fn explicit_and_wildcard_imports_expose_project_types() {
    let index = library();
    let explicit = at(
        "package app;\n\
         import demo.Person;\n\
         import demo.*;\n\
         class Main { void run() { Per<|> } }",
        &index,
    );
    assert!(labels(&explicit).contains(&"Person"));

    let wildcard = at(
        "package app;\n\
         import demo.*;\n\
         class Main { void run() { Col<|> } }",
        &index,
    );
    assert!(labels(&wildcard).contains(&"Color"));
}

#[test]
fn static_imports_expose_members_and_constants() {
    let index = library();
    let explicit = at(
        "package app;\n\
         import static demo.Util.MAX;\n\
         class Main { void run() { MA<|> } }",
        &index,
    );
    assert!(labels(&explicit).contains(&"MAX"));

    let wildcard_constant = at(
        "package app;\n\
         import static demo.Util.*;\n\
         class Main { void run() { MA<|> } }",
        &index,
    );
    assert!(labels(&wildcard_constant).contains(&"MAX"));

    let wildcard_method = at(
        "package app;\n\
         import static demo.Util.*;\n\
         class Main { void run() { he<|> } }",
        &index,
    );
    assert!(labels(&wildcard_method).contains(&"help"));

    let explicit_enum = at(
        "package app; import static demo.Color.RED; class Main { void run() { RE<|> } }",
        &index,
    );
    assert!(labels(&explicit_enum).contains(&"RED"));
}

#[test]
fn same_package_project_types_are_visible() {
    let index = library();
    let completion = at(
        "package demo;\n\
         class Main { void run() { Pers<|> } }",
        &index,
    );
    assert!(labels(&completion).contains(&"Person"));
}

#[test]
fn prefix_filtering_is_case_insensitive_and_ignores_other_names() {
    let index = SymbolIndex::new();
    let completion = at("class Demo { void run() { RE<|> } }", &index);
    let labels = labels(&completion);
    assert!(labels.contains(&"return"), "{labels:?}");
    assert!(labels.contains(&"record"), "{labels:?}");
    assert!(!labels.contains(&"while"), "{labels:?}");
}

#[test]
fn comments_and_strings_suppress_completion() {
    let index = SymbolIndex::new();
    let source = "class Demo { void run() { // note\n String s = \"text\n } }";
    let comment_caret = source.find("note").expect("note") + 2;
    assert!(complete(source, comment_caret, &index).is_none());
    let string_caret = source.find("text").expect("text") + 2;
    assert!(complete(source, string_caret, &index).is_none());
}

#[test]
fn an_unknown_prefix_returns_no_completion() {
    let index = SymbolIndex::new();
    let source = "class Demo { void run() { zzz } }";
    let caret = source.find("zzz").expect("prefix") + 3;
    assert!(complete(source, caret, &index).is_none());
}
