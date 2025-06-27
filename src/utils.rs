use std::path::PathBuf;

/// Sanitize the file path to prevent directory traversal attacks
pub fn sanitize_path(path: &str) -> String {
    // Remove any ".." components and ensure we stay within our allowed directory
    let path = path.replace("..", "");

    // Ensure the path is relative and doesn't start with "/"
    let path = path.trim_start_matches('/');

    // Convert to PathBuf for safe path handling
    let path_buf = PathBuf::from(path);

    // Return the sanitized path as a string
    path_buf.to_string_lossy().to_string()
}
