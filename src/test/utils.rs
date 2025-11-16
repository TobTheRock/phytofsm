pub fn get_adjacent_file_path(
    current_file_path: &str,
    adjacent_file_name: &str,
) -> std::path::PathBuf {
    let current_path = std::path::Path::new(current_file_path);
    let parent_dir = current_path
        .parent()
        .expect("Failed to get parent directory for adjacent file path");
    parent_dir
        .join(adjacent_file_name)
        .canonicalize()
        .expect("Failed to canonicalize adjacent file path")
}
