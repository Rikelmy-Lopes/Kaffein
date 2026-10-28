//! Lightweight Java declarations produced by the source parser.
//!
//! This is not a Java compiler AST. It records exactly what completion needs:
//! packages, imports, type declarations, members, and the visible local
//! variables of each method body.

use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeKind {
    Class,
    Interface,
    Enum,
    Record,
    Annotation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberKind {
    Method,
    Field,
    Constructor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalKind {
    Parameter,
    Variable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub type_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Local {
    pub name: String,
    pub type_name: String,
    pub kind: LocalKind,
    /// Byte offset after which the name can be referenced.
    pub visible_from: usize,
    /// Byte offset at which the enclosing block ends.
    pub scope_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub kind: MemberKind,
    /// Return type for methods, declared type for fields, `None` for
    /// constructors.
    pub type_name: Option<String>,
    pub params: Vec<Param>,
    pub is_static: bool,
    pub is_final: bool,
    pub body: Option<Range<usize>>,
    pub locals: Vec<Local>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDecl {
    pub name: String,
    pub kind: TypeKind,
    pub is_static: bool,
    pub supertypes: Vec<String>,
    pub enum_constants: Vec<String>,
    pub members: Vec<Member>,
    pub nested: Vec<TypeDecl>,
    /// Byte range of the type body, including the braces.
    pub body: Range<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub path: String,
    pub is_static: bool,
    pub is_wildcard: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JavaFile {
    pub package: Option<String>,
    pub imports: Vec<Import>,
    pub types: Vec<TypeDecl>,
}

impl TypeDecl {
    pub fn fields(&self) -> impl Iterator<Item = &Member> {
        self.members
            .iter()
            .filter(|member| member.kind == MemberKind::Field)
    }

    pub fn methods(&self) -> impl Iterator<Item = &Member> {
        self.members
            .iter()
            .filter(|member| matches!(member.kind, MemberKind::Method | MemberKind::Constructor))
    }

    /// Whether `offset` falls inside this type's braces.
    pub fn contains(&self, offset: usize) -> bool {
        self.body.start <= offset && offset <= self.body.end
    }

    /// This declaration and every nested declaration.
    pub fn all_types<'a>(&'a self, out: &mut Vec<&'a TypeDecl>) {
        out.push(self);
        for nested in &self.nested {
            nested.all_types(out);
        }
    }

    /// The member whose body contains `offset`, if any.
    pub fn enclosing_member(&self, offset: usize) -> Option<&Member> {
        self.members.iter().find(|member| {
            member
                .body
                .as_ref()
                .is_some_and(|body| body.start <= offset && offset <= body.end)
        })
    }
}
