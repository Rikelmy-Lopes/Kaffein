//! Infrastructure implementations of the ports declared by `kaffein-domain`.

mod filesystem;
mod project_loader;

#[cfg(test)]
mod test_support;

pub use filesystem::StdFileSystem;
pub use project_loader::FsProjectLoader;
