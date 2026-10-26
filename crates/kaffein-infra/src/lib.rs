//! Infrastructure implementations of the ports declared by `kaffein-domain`.

mod filesystem;
mod locale_files;
mod project_loader;

#[cfg(test)]
mod test_support;

pub use filesystem::StdFileSystem;
pub use locale_files::FileLocaleSource;
pub use project_loader::FsProjectLoader;
