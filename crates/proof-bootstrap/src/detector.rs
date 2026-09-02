use std::path::Path;

#[derive(Debug, Clone)]
pub enum Language {
    Python,
    TypeScript,
    JavaScript,
    Java,
    Kotlin,
    Unknown,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Python => write!(f, "python"),
            Language::TypeScript => write!(f, "typescript"),
            Language::JavaScript => write!(f, "javascript"),
            Language::Java => write!(f, "java"),
            Language::Kotlin => write!(f, "kotlin"),
            Language::Unknown => write!(f, "unknown"),
        }
    }
}

pub fn detect_language(path: &Path) -> Language {
    match path.extension().and_then(|e| e.to_str()) {
        Some("py") => Language::Python,
        Some("ts") => Language::TypeScript,
        Some("js") => Language::JavaScript,
        Some("java") => Language::Java,
        Some("kt") => Language::Kotlin,
        _ => Language::Unknown,
    }
}
