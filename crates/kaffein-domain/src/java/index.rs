//! A source index of Java declarations, keyed by file path.
//!
//! The index is built from the lightweight parser, not a compiler. It answers
//! the questions completion asks: which types a package declares, which type a
//! qualified import names, and which members a type exposes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::ast::{Import, JavaFile, Member, MemberKind, Param, TypeDecl, TypeKind};
use super::parser::parse;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedMember {
    pub name: String,
    pub kind: MemberKind,
    pub type_name: Option<String>,
    pub params: Vec<Param>,
    pub is_static: bool,
    pub is_final: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedType {
    pub name: String,
    pub kind: TypeKind,
    pub is_static: bool,
    pub supertypes: Vec<String>,
    pub enum_constants: Vec<String>,
    pub members: Vec<IndexedMember>,
    pub nested: Vec<IndexedType>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IndexedFile {
    pub package: Option<String>,
    pub imports: Vec<Import>,
    pub types: Vec<IndexedType>,
}

#[derive(Clone, Debug, Default)]
pub struct SymbolIndex {
    files: BTreeMap<PathBuf, IndexedFile>,
}

impl IndexedMember {
    fn from_member(member: &Member) -> Self {
        Self {
            name: member.name.clone(),
            kind: member.kind,
            type_name: member.type_name.clone(),
            params: member.params.clone(),
            is_static: member.is_static,
            is_final: member.is_final,
        }
    }
}

impl IndexedType {
    fn from_decl(declaration: &TypeDecl) -> Self {
        Self {
            name: declaration.name.clone(),
            kind: declaration.kind,
            is_static: declaration.is_static,
            supertypes: declaration.supertypes.clone(),
            enum_constants: declaration.enum_constants.clone(),
            members: declaration
                .members
                .iter()
                .map(IndexedMember::from_member)
                .collect(),
            nested: declaration
                .nested
                .iter()
                .map(IndexedType::from_decl)
                .collect(),
        }
    }

    /// This type and every nested type.
    pub fn all_types(&self) -> Vec<&IndexedType> {
        let mut types = Vec::new();
        self.collect(&mut types);
        types
    }

    fn collect<'a>(&'a self, out: &mut Vec<&'a IndexedType>) {
        out.push(self);
        for nested in &self.nested {
            nested.collect(out);
        }
    }

    pub fn is_enum(&self) -> bool {
        self.kind == TypeKind::Enum
    }
}

impl IndexedFile {
    pub fn from_source(source: &str) -> Self {
        Self::from_java_file(&parse(source))
    }

    pub fn from_java_file(file: &JavaFile) -> Self {
        Self {
            package: file.package.clone(),
            imports: file.imports.clone(),
            types: file.types.iter().map(IndexedType::from_decl).collect(),
        }
    }

    /// Every type in the file, including nested declarations.
    pub fn all_types(&self) -> Vec<&IndexedType> {
        let mut types = Vec::new();
        for ty in &self.types {
            ty.collect(&mut types);
        }
        types
    }

    pub fn package(&self) -> &str {
        self.package.as_deref().unwrap_or("")
    }
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses `source` and replaces any previous entry for `path`.
    pub fn upsert(&mut self, path: impl Into<PathBuf>, source: &str) {
        self.files
            .insert(path.into(), IndexedFile::from_source(source));
    }

    pub fn remove(&mut self, path: &Path) -> Option<IndexedFile> {
        self.files.remove(path)
    }

    pub fn clear(&mut self) {
        self.files.clear();
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn file(&self, path: &Path) -> Option<&IndexedFile> {
        self.files.get(path)
    }

    pub fn files(&self) -> impl Iterator<Item = (&Path, &IndexedFile)> {
        self.files.iter().map(|(path, file)| (path.as_path(), file))
    }

    /// Every type declared in any indexed file, flattened.
    pub fn all_types(&self) -> Vec<(&Path, &IndexedType)> {
        self.files
            .iter()
            .flat_map(|(path, file)| {
                file.all_types()
                    .into_iter()
                    .map(move |ty| (path.as_path(), ty))
            })
            .collect()
    }

    /// Types whose package equals `package`.
    pub fn types_in_package(&self, package: &str) -> Vec<(&Path, &IndexedType)> {
        self.files
            .iter()
            .filter(|(_, file)| file.package() == package)
            .flat_map(|(path, file)| {
                file.all_types()
                    .into_iter()
                    .map(move |ty| (path.as_path(), ty))
            })
            .collect()
    }

    /// Resolves a fully qualified type such as `java.util.List`.
    pub fn find_type(&self, qualified: &str) -> Option<(&Path, &IndexedType)> {
        let (package, name) = qualified.rsplit_once('.')?;
        self.files.iter().find_map(|(path, file)| {
            if file.package() != package {
                return None;
            }
            file.all_types()
                .into_iter()
                .find(|ty| ty.name == name)
                .map(|ty| (path.as_path(), ty))
        })
    }

    /// Types with the given simple name, across every indexed package.
    pub fn find_simple_type(&self, name: &str) -> Vec<(&Path, &IndexedType)> {
        self.files
            .iter()
            .flat_map(|(path, file)| {
                file.all_types()
                    .into_iter()
                    .filter(|ty| ty.name == name)
                    .map(move |ty| (path.as_path(), ty))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
