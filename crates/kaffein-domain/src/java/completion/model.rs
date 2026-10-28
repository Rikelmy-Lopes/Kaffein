//! Owned type information used while resolving a completion receiver.

use crate::java::ast::{MemberKind, Param, TypeDecl, TypeKind};
use crate::java::index::IndexedType;

use super::{CompletionItem, CompletionKind};

#[derive(Clone, Debug)]
pub(super) struct MemberInfo {
    pub(super) name: String,
    pub(super) kind: MemberKind,
    pub(super) type_name: Option<String>,
    pub(super) params: Vec<Param>,
    pub(super) is_static: bool,
}

#[derive(Clone, Debug)]
pub(super) struct TypeInfo {
    pub(super) name: String,
    pub(super) kind: TypeKind,
    pub(super) supertypes: Vec<String>,
    pub(super) enum_constants: Vec<String>,
    pub(super) members: Vec<MemberInfo>,
    pub(super) nested: Vec<(String, TypeKind)>,
}

impl From<&TypeDecl> for TypeInfo {
    fn from(declaration: &TypeDecl) -> Self {
        Self {
            name: declaration.name.clone(),
            kind: declaration.kind,
            supertypes: declaration.supertypes.clone(),
            enum_constants: declaration.enum_constants.clone(),
            members: declaration
                .members
                .iter()
                .map(|member| MemberInfo {
                    name: member.name.clone(),
                    kind: member.kind,
                    type_name: member.type_name.clone(),
                    params: member.params.clone(),
                    is_static: member.is_static,
                })
                .collect(),
            nested: declaration
                .nested
                .iter()
                .map(|nested| (nested.name.clone(), nested.kind))
                .collect(),
        }
    }
}

impl From<&IndexedType> for TypeInfo {
    fn from(ty: &IndexedType) -> Self {
        Self {
            name: ty.name.clone(),
            kind: ty.kind,
            supertypes: ty.supertypes.clone(),
            enum_constants: ty.enum_constants.clone(),
            members: ty
                .members
                .iter()
                .map(|member| MemberInfo {
                    name: member.name.clone(),
                    kind: member.kind,
                    type_name: member.type_name.clone(),
                    params: member.params.clone(),
                    is_static: member.is_static,
                })
                .collect(),
            nested: ty
                .nested
                .iter()
                .map(|nested| (nested.name.clone(), nested.kind))
                .collect(),
        }
    }
}

impl TypeInfo {
    pub(super) fn find_member(&self, name: &str) -> Option<&MemberInfo> {
        self.members.iter().find(|member| member.name == name)
    }

    pub(super) fn has_nested(&self, name: &str) -> bool {
        self.nested.iter().any(|(nested, _)| nested == name)
    }

    pub(super) fn items(&self) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        for constant in &self.enum_constants {
            items.push(CompletionItem {
                label: constant.clone(),
                detail: self.name.clone(),
                insert_text: constant.clone(),
                kind: CompletionKind::EnumConstant,
            });
        }
        for member in &self.members {
            items.push(member.item());
        }
        for (nested, kind) in &self.nested {
            items.push(type_item(nested, *kind));
        }
        items
    }

    pub(super) fn member_items(&self, static_only: bool) -> Vec<CompletionItem> {
        self.members
            .iter()
            .filter(|member| !static_only || member.is_static)
            .map(MemberInfo::item)
            .collect()
    }
}

impl MemberInfo {
    pub(super) fn item(&self) -> CompletionItem {
        let detail = match self.kind {
            MemberKind::Field => {
                format!("{} {}", self.type_name.as_deref().unwrap_or("?"), self.name)
            }
            MemberKind::Method => format!(
                "{} {}({})",
                self.type_name.as_deref().unwrap_or("void"),
                self.name,
                parameter_list(&self.params)
            ),
            MemberKind::Constructor => {
                format!("{}({})", self.name, parameter_list(&self.params))
            }
        };
        CompletionItem {
            label: self.name.clone(),
            detail,
            insert_text: self.name.clone(),
            kind: if self.kind == MemberKind::Field {
                CompletionKind::Field
            } else {
                CompletionKind::Method
            },
        }
    }
}

pub(super) fn type_item(name: &str, kind: TypeKind) -> CompletionItem {
    CompletionItem {
        label: name.to_owned(),
        detail: format!("{} {name}", type_keyword(kind)),
        insert_text: name.to_owned(),
        kind: CompletionKind::Type,
    }
}

pub(super) fn variable_item(name: &str, type_name: &str) -> CompletionItem {
    CompletionItem {
        label: name.to_owned(),
        detail: if type_name.is_empty() {
            name.to_owned()
        } else {
            format!("{type_name} {name}")
        },
        insert_text: name.to_owned(),
        kind: CompletionKind::Variable,
    }
}

pub(super) fn type_keyword(kind: TypeKind) -> &'static str {
    match kind {
        TypeKind::Class => "class",
        TypeKind::Interface => "interface",
        TypeKind::Enum => "enum",
        TypeKind::Record => "record",
        TypeKind::Annotation => "@interface",
    }
}

fn parameter_list(params: &[Param]) -> String {
    params
        .iter()
        .map(|param| param.type_name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
