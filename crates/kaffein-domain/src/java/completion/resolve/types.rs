//! Type lookup across the current buffer and the project index.

use crate::java::ast::{JavaFile, TypeDecl};
use crate::java::index::{IndexedFile, SymbolIndex};

use super::super::model::TypeInfo;

pub(super) fn enclosing_type(file: &JavaFile, caret: usize) -> Option<&TypeDecl> {
    all_types(file)
        .into_iter()
        .filter(|ty| ty.contains(caret))
        .max_by_key(|ty| ty.body.start)
}

pub(super) fn all_types(file: &JavaFile) -> Vec<&TypeDecl> {
    let mut types = Vec::new();
    for ty in &file.types {
        ty.all_types(&mut types);
    }
    types
}

pub(super) fn resolve_type_named(
    raw: &str,
    file: &JavaFile,
    current: &IndexedFile,
    index: &SymbolIndex,
) -> Option<TypeInfo> {
    let name = simple_name(raw);
    if name.is_empty() {
        return None;
    }
    for ty in all_types(file) {
        if ty.name == name {
            return Some(TypeInfo::from(ty));
        }
    }
    for import in &current.imports {
        if import.is_static || import.is_wildcard {
            continue;
        }
        if import.path.rsplit('.').next() == Some(name.as_str()) {
            if let Some((_, ty)) = index.find_type(&import.path) {
                return Some(TypeInfo::from(ty));
            }
        }
    }
    for import in &current.imports {
        if import.is_static || !import.is_wildcard {
            continue;
        }
        if let Some((_, ty)) = index
            .types_in_package(&import.path)
            .into_iter()
            .find(|(_, ty)| ty.name == name)
        {
            return Some(TypeInfo::from(ty));
        }
    }
    if let Some((_, ty)) = index
        .types_in_package(current.package())
        .into_iter()
        .find(|(_, ty)| ty.name == name)
    {
        return Some(TypeInfo::from(ty));
    }
    let matches = index.find_simple_type(&name);
    (matches.len() == 1).then(|| TypeInfo::from(matches[0].1))
}

fn simple_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut name = trimmed.split('<').next().unwrap_or(trimmed).trim();
    while let Some(stripped) = name.strip_suffix("[]") {
        name = stripped.trim();
    }
    name.split_whitespace()
        .next()
        .unwrap_or(name)
        .rsplit('.')
        .next()
        .unwrap_or(name)
        .trim()
        .to_owned()
}
