//! Symbol resolution for plain and receiver completion.

mod types;

use std::collections::HashSet;

use crate::java::ast::{JavaFile, TypeKind};
use crate::java::index::{IndexedFile, SymbolIndex};
use crate::java::keyword::KEYWORDS;

use self::types::{all_types, enclosing_type, resolve_type_named};
use super::model::{type_item, variable_item, TypeInfo};
use super::{CompletionItem, CompletionKind};

pub(super) fn visible_symbols(
    file: &JavaFile,
    caret: usize,
    index: &SymbolIndex,
) -> Vec<CompletionItem> {
    let current = IndexedFile::from_java_file(file);
    let mut items = Vec::new();
    let mut seen = HashSet::new();

    if let Some(ty) = enclosing_type(file, caret) {
        if let Some(member) = ty.enclosing_member(caret) {
            for local in &member.locals {
                if local.visible_from <= caret && caret <= local.scope_end {
                    push(
                        &mut items,
                        &mut seen,
                        variable_item(&local.name, &local.type_name),
                    );
                }
            }
        }
        add_type_members(
            TypeInfo::from(ty),
            file,
            &current,
            index,
            &mut items,
            &mut seen,
            0,
        );
    }

    for ty in all_types(file) {
        push(&mut items, &mut seen, type_item(&ty.name, ty.kind));
    }

    for import in &current.imports {
        if import.is_static {
            add_static_import(import, file, &current, index, &mut items, &mut seen);
        } else if import.is_wildcard {
            for (_, ty) in index.types_in_package(&import.path) {
                push(&mut items, &mut seen, type_item(&ty.name, ty.kind));
            }
        } else {
            let name = import.path.rsplit('.').next().unwrap_or(&import.path);
            let item = index
                .find_type(&import.path)
                .map(|(_, ty)| type_item(&ty.name, ty.kind))
                .unwrap_or_else(|| type_item(name, TypeKind::Class));
            push(&mut items, &mut seen, item);
        }
    }

    for (_, ty) in index.types_in_package(current.package()) {
        push(&mut items, &mut seen, type_item(&ty.name, ty.kind));
    }

    for keyword in KEYWORDS {
        push(
            &mut items,
            &mut seen,
            CompletionItem {
                label: (*keyword).to_owned(),
                detail: String::new(),
                insert_text: (*keyword).to_owned(),
                kind: CompletionKind::Keyword,
            },
        );
    }
    items
}

pub(super) fn receiver_symbols(
    file: &JavaFile,
    caret: usize,
    receiver: &str,
    index: &SymbolIndex,
) -> Vec<CompletionItem> {
    let current = IndexedFile::from_java_file(file);
    let segments = split_dots(receiver);
    let Some(first) = segments.first() else {
        return Vec::new();
    };
    let Some(mut info) = first_segment(first, file, caret, &current, index) else {
        return Vec::new();
    };
    for segment in &segments[1..] {
        let Some(next) = member_type(&info, segment, file, &current, index) else {
            return Vec::new();
        };
        info = next;
    }
    info.items()
}

fn first_segment(
    segment: &str,
    file: &JavaFile,
    caret: usize,
    current: &IndexedFile,
    index: &SymbolIndex,
) -> Option<TypeInfo> {
    let segment = segment.trim();
    if segment == "this" {
        return enclosing_type(file, caret).map(TypeInfo::from);
    }
    if segment == "super" {
        let ty = enclosing_type(file, caret)?;
        let parent = ty.supertypes.first()?;
        return resolve_type_named(parent, file, current, index);
    }
    if let Some(rest) = segment.strip_prefix("new ") {
        return resolve_type_named(strip_call(rest), file, current, index);
    }
    if let Some(member) = find_variable(file, caret, segment) {
        return resolve_type_named(&member, file, current, index);
    }
    if let Some(info) = resolve_type_named(segment, file, current, index) {
        return Some(info);
    }
    let ty = enclosing_type(file, caret)?;
    let info = TypeInfo::from(ty);
    member_type(&info, segment, file, current, index)
}

fn member_type(
    info: &TypeInfo,
    segment: &str,
    file: &JavaFile,
    current: &IndexedFile,
    index: &SymbolIndex,
) -> Option<TypeInfo> {
    let name = strip_call(segment);
    if let Some(member) = info.find_member(name) {
        return member
            .type_name
            .as_deref()
            .and_then(|name| resolve_type_named(name, file, current, index));
    }
    if info.has_nested(name) {
        return resolve_type_named(name, file, current, index);
    }
    None
}

fn find_variable(file: &JavaFile, caret: usize, name: &str) -> Option<String> {
    let ty = enclosing_type(file, caret)?;
    if let Some(member) = ty.enclosing_member(caret) {
        if let Some(local) = member.locals.iter().find(|local| {
            local.name == name && local.visible_from <= caret && caret <= local.scope_end
        }) {
            return Some(local.type_name.clone());
        }
    }
    ty.fields()
        .find(|field| field.name == name)
        .and_then(|field| field.type_name.clone())
}

fn add_static_import(
    import: &crate::java::ast::Import,
    file: &JavaFile,
    current: &IndexedFile,
    index: &SymbolIndex,
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<(String, CompletionKind)>,
) {
    if import.is_wildcard {
        if let Some(info) = resolve_type_named(&import.path, file, current, index) {
            for constant in &info.enum_constants {
                push(
                    items,
                    seen,
                    CompletionItem {
                        label: constant.clone(),
                        detail: info.name.clone(),
                        insert_text: constant.clone(),
                        kind: CompletionKind::EnumConstant,
                    },
                );
            }
            for item in info.member_items(true) {
                push(items, seen, item);
            }
        }
        return;
    }
    let Some((owner, name)) = import.path.rsplit_once('.') else {
        return;
    };
    let Some(info) = resolve_type_named(owner, file, current, index) else {
        return;
    };
    if let Some(member) = info.find_member(name) {
        push(items, seen, member.item());
    } else if info.has_nested(name) {
        if let Some(nested) = resolve_type_named(name, file, current, index) {
            push(items, seen, type_item(&nested.name, nested.kind));
        }
    } else if info.enum_constants.iter().any(|constant| constant == name) {
        push(
            items,
            seen,
            CompletionItem {
                label: name.to_owned(),
                detail: info.name.clone(),
                insert_text: name.to_owned(),
                kind: CompletionKind::EnumConstant,
            },
        );
    }
}

fn add_type_members(
    info: TypeInfo,
    file: &JavaFile,
    current: &IndexedFile,
    index: &SymbolIndex,
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<(String, CompletionKind)>,
    depth: usize,
) {
    if depth > 8 {
        return;
    }
    for item in info.items() {
        push(items, seen, item);
    }
    for supertype in &info.supertypes {
        if supertype == &info.name {
            continue;
        }
        if let Some(parent) = resolve_type_named(supertype, file, current, index) {
            add_type_members(parent, file, current, index, items, seen, depth + 1);
        }
    }
}

fn push(
    items: &mut Vec<CompletionItem>,
    seen: &mut HashSet<(String, CompletionKind)>,
    item: CompletionItem,
) {
    if seen.insert((item.label.clone(), item.kind)) {
        items.push(item);
    }
}

fn split_dots(expression: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    for (index, ch) in expression.char_indices() {
        match ch {
            '(' | '[' | '<' => depth += 1,
            ')' | ']' | '>' => depth = (depth - 1).max(0),
            '.' if depth == 0 => {
                segments.push(expression[start..index].trim().to_owned());
                start = index + 1;
            }
            _ => {}
        }
    }
    segments.push(expression[start..].trim().to_owned());
    segments.retain(|segment| !segment.is_empty());
    segments
}

fn strip_call(segment: &str) -> &str {
    let segment = segment.trim();
    if let Some(index) = segment.find('(') {
        segment[..index].trim()
    } else {
        segment
    }
}
