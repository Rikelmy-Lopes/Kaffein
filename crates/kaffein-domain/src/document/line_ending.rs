//! Line-ending convention detected from a file and restored on save.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineEnding {
    #[default]
    Lf,
    Crlf,
}

impl LineEnding {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lf => "LF",
            Self::Crlf => "CRLF",
        }
    }

    /// Detects the convention from raw file contents. A file containing any
    /// CRLF is treated as CRLF; mixed files are normalised to the dominant
    /// convention and the discrepancy is not surfaced yet.
    pub fn detect(raw: &str) -> Self {
        if raw.contains("\r\n") {
            Self::Crlf
        } else {
            Self::Lf
        }
    }
}
