//! Symbol index tests.

use super::{IndexedFile, SymbolIndex};
use crate::java::ast::TypeKind;

#[test]
fn upsert_replaces_a_file_and_remove_deletes_it() {
    let mut index = SymbolIndex::new();
    index.upsert("C:/src/A.java", "package demo; class A {}");
    assert_eq!(index.len(), 1);
    index.upsert("C:/src/A.java", "package demo; class A { int value; }");
    let members = &index.file("C:/src/A.java".as_ref()).expect("file").types[0].members;
    assert_eq!(members.len(), 1);

    assert!(index.remove("C:/src/A.java".as_ref()).is_some());
    assert!(index.is_empty());
}

#[test]
fn files_are_grouped_by_declared_package() {
    let mut index = SymbolIndex::new();
    index.upsert("C:/src/A.java", "package demo.one; class A {}");
    index.upsert("C:/src/B.java", "package demo.two; class B {}");
    index.upsert("C:/src/C.java", "class C {}");

    let one = index.types_in_package("demo.one");
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].1.name, "A");
    assert_eq!(index.types_in_package("demo.two")[0].1.name, "B");
    assert_eq!(index.types_in_package("")[0].1.name, "C");
}

#[test]
fn finds_a_type_by_qualified_name() {
    let mut index = SymbolIndex::new();
    index.upsert(
        "C:/src/List.java",
        "package java.util; public interface List<E> {}",
    );

    let (_, ty) = index.find_type("java.util.List").expect("List");
    assert_eq!(ty.name, "List");
    assert_eq!(ty.kind, TypeKind::Interface);
    assert!(index.find_type("java.util.Map").is_none());
    assert!(index.find_type("List").is_none());
}

#[test]
fn finds_every_type_with_a_simple_name() {
    let mut index = SymbolIndex::new();
    index.upsert("C:/src/A.java", "package one; class Shared {}");
    index.upsert("C:/src/B.java", "package two; class Shared {}");

    assert_eq!(index.find_simple_type("Shared").len(), 2);
    assert_eq!(index.find_simple_type("Missing").len(), 0);
}

#[test]
fn nested_types_are_indexed_and_flattened() {
    let file =
        IndexedFile::from_source("package demo; class Outer { static class Inner { int x; } }");
    let names: Vec<&str> = file.all_types().iter().map(|ty| ty.name.as_str()).collect();
    assert_eq!(names, ["Outer", "Inner"]);
    assert_eq!(file.all_types()[1].members[0].name, "x");
}

#[test]
fn members_keep_static_final_and_parameter_information() {
    let file = IndexedFile::from_source(
        "package demo; class Service {\n\
             static final int LIMIT = 3;\n\
             static String find(int id, String name) { return name; }\n\
         }",
    );
    let ty = &file.types[0];
    let limit = &ty.members[0];
    assert!(limit.is_static);
    assert!(limit.is_final);
    assert_eq!(limit.type_name.as_deref(), Some("int"));
    let find = &ty.members[1];
    assert_eq!(find.params.len(), 2);
    assert_eq!(find.params[1].type_name, "String");
}

#[test]
fn an_unparseable_file_still_produces_an_entry() {
    let mut index = SymbolIndex::new();
    index.upsert("C:/src/Broken.java", "package demo; class Broken {");

    assert_eq!(index.len(), 1);
    assert_eq!(
        index.find_type("demo.Broken").expect("Broken").1.name,
        "Broken"
    );
}
