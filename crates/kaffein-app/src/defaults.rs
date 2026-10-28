//! Content used when the workspace has no saved document yet.

/// The Java source shown in a new scratch document.
///
/// It is intentionally a complete, runnable class so the Run command has
/// something to execute before a file or project is opened.
pub const DEFAULT_JAVA_SOURCE: &str = "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, Kaffein!\");\n    }\n}\n";
