use std::fs;
use std::path::Path;

fn main() {
    tauri_build::build();
    normalize_generated_schemas();
}

/// Reformat generated JSON schema files with pretty printing and a trailing newline.
///
/// Tauri's build generates JSON files without trailing newlines and some in minified format.
/// This causes repeated diffs with formatters (basefmt, prettier) that enforce trailing newlines
/// and consistent formatting.
fn normalize_generated_schemas() {
    let schema_dir = Path::new("gen/schemas");
    let Ok(entries) = fs::read_dir(schema_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
                continue;
            };
            let Ok(mut pretty) = serde_json::to_string_pretty(&value) else {
                continue;
            };
            pretty.push('\n');
            // Only write if content changed to avoid unnecessary rebuilds.
            if pretty != content {
                let _ = fs::write(&path, pretty);
            }
        }
    }
}
