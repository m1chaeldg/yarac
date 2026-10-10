use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathResolutionError {
    #[error("Path cannot be empty")]
    EmptyPath,
    #[error("Absolute machine paths are prohibited in .yarac configurations: '{0}'")]
    AbsolutePathProhibited(String),
    #[error("Directory traversal attempted escaping workspace root: '{0}'")]
    DirectoryTraversalProhibited(String),
    #[error("File not found at resolved workspace path: '{0}'")]
    FileNotFound(String),
}

/// Resolves a workspace-relative path safely within the workspace boundary.
/// Rejects absolute paths and paths attempting `../` escape traversal.
pub fn resolve_workspace_path(
    workspace_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, PathResolutionError> {
    let trimmed = relative_path.trim();
    if trimmed.is_empty() {
        return Err(PathResolutionError::EmptyPath);
    }

    let input_path = Path::new(trimmed);

    // Reject absolute paths
    if input_path.is_absolute() {
        return Err(PathResolutionError::AbsolutePathProhibited(trimmed.to_string()));
    }

    // Check components for directory traversal escaping
    let mut normalized_components = Vec::new();
    for component in input_path.components() {
        match component {
            Component::ParentDir => {
                if normalized_components.is_empty() {
                    return Err(PathResolutionError::DirectoryTraversalProhibited(trimmed.to_string()));
                }
                normalized_components.pop();
            }
            Component::Normal(part) => {
                normalized_components.push(part);
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {
                return Err(PathResolutionError::AbsolutePathProhibited(trimmed.to_string()));
            }
        }
    }

    let mut resolved = workspace_root.to_path_buf();
    for comp in normalized_components {
        resolved.push(comp);
    }

    Ok(resolved)
}

/// Helper to convert a path to a portable workspace-relative string format (`./dir/file`)
pub fn to_relative_workspace_string(
    workspace_root: &Path,
    target_path: &Path,
) -> Result<String, PathResolutionError> {
    let rel = match target_path.strip_prefix(workspace_root) {
        Ok(r) => r,
        Err(_) => {
            return Err(PathResolutionError::DirectoryTraversalProhibited(
                target_path.display().to_string(),
            ))
        }
    };

    let mut result = String::from("./");
    for (i, comp) in rel.components().enumerate() {
        if i > 0 {
            result.push('/');
        }
        result.push_str(&comp.as_os_str().to_string_lossy());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_relative_path() {
        let ws = PathBuf::from("/workspace/project");
        let resolved = resolve_workspace_path(&ws, "./fixtures/avatar.png").unwrap();
        assert_eq!(resolved, PathBuf::from("/workspace/project/fixtures/avatar.png"));
    }

    #[test]
    fn test_rejects_absolute_path() {
        let ws = PathBuf::from("/workspace/project");
        let err = resolve_workspace_path(&ws, "/etc/passwd").unwrap_err();
        assert!(matches!(err, PathResolutionError::AbsolutePathProhibited(_)));
    }

    #[test]
    fn test_rejects_traversal_escape() {
        let ws = PathBuf::from("/workspace/project");
        let err = resolve_workspace_path(&ws, "../../secret.txt").unwrap_err();
        assert!(matches!(err, PathResolutionError::DirectoryTraversalProhibited(_)));
    }
}
