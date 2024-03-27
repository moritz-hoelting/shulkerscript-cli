use std::{io, path::Path};

pub fn to_absolute_path(path: &Path) -> io::Result<String> {
    Ok(std::fs::canonicalize(path)?
        .display()
        .to_string()
        .trim_start_matches(r"\\?\")
        .to_string())
}
