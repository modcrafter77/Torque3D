use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

const RESOLUTIONS: &[&str] = &["hi", "med", "lo"];
const EXTENSIONS: &[&str] = &["", ".jpg", ".png", ".gif", ".bmp"];

/// Resolve a TGE interior material name (e.g. `ddd/concrete.hi.base`) to an image on disk.
pub fn resolve_material_texture(maps_dir: &Path, material_name: &str) -> Option<PathBuf> {
    for candidate in material_path_candidates(material_name) {
        let base = maps_dir.join(&candidate);
        for ext in EXTENSIONS {
            let path = if ext.is_empty() {
                base.clone()
            } else {
                PathBuf::from(format!("{}{}", base.display(), ext))
            };
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

pub fn resolve_material_textures(
    maps_dir: &Path,
    material_names: &[String],
) -> Vec<Option<PathBuf>> {
    material_names
        .iter()
        .map(|name| resolve_material_texture(maps_dir, name))
        .collect()
}

/// Path for Collada `<init_from>` — relative to the DAE file when possible.
pub fn href_for_collada(dae_path: &Path, texture_path: &Path) -> String {
    let dae_dir = dae_path.parent().unwrap_or(Path::new("."));
    relative_href(dae_dir, texture_path).unwrap_or_else(|| file_uri(texture_path))
}

fn material_path_candidates(material_name: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    let mut push = |value: String| {
        let normalized = value.replace('\\', "/");
        if seen.insert(normalized.clone()) {
            out.push(normalized);
        }
    };

    push(material_name.to_string());

    if let Some(file_name) = material_name.rsplit(['/', '\\']).next() {
        push(file_name.to_string());
    }

    for res in RESOLUTIONS {
        if let Some(swapped) = swap_resolution(material_name, res) {
            push(swapped.clone());
            if let Some(file_name) = swapped.rsplit(['/', '\\']).next() {
                push(file_name.to_string());
            }
        }
    }

    if let Some(stripped) = material_name.strip_suffix(".base") {
        push(stripped.to_string());
        if let Some(file_name) = stripped.rsplit(['/', '\\']).next() {
            push(file_name.to_string());
        }
        for res in RESOLUTIONS {
            if let Some(swapped) = swap_resolution(stripped, res) {
                push(swapped);
            }
        }
    }

    out
}

fn swap_resolution(name: &str, new_res: &str) -> Option<String> {
    let skin_suffix = ".base";
    let (stem, skin) = if let Some(stem) = name.strip_suffix(skin_suffix) {
        (stem, skin_suffix)
    } else {
        (name, "")
    };

    let (prefix, current_res) = stem.rsplit_once('.')?;
    if !RESOLUTIONS.contains(&current_res) {
        return None;
    }
    Some(format!("{prefix}.{new_res}{skin}"))
}

fn relative_href(from_dir: &Path, target: &Path) -> Option<String> {
    let from = from_dir.canonicalize().ok()?;
    let to = target.canonicalize().ok()?;

    let from_parts: Vec<_> = from.components().collect();
    let to_parts: Vec<_> = to.components().collect();

    let mut shared = 0usize;
    while shared < from_parts.len()
        && shared < to_parts.len()
        && from_parts[shared] == to_parts[shared]
    {
        shared += 1;
    }

    let mut href = Vec::new();
    for _ in shared..from_parts.len() {
        href.push("..");
    }
    for part in &to_parts[shared..] {
        if let Component::Normal(name) = part {
            href.push(name.to_str()?);
        }
    }

    if href.is_empty() {
        to.file_name().and_then(|s| s.to_str()).map(str::to_string)
    } else {
        Some(href.join("/"))
    }
}

fn file_uri(path: &Path) -> String {
    let normalized = path.display().to_string().replace('\\', "/");
    if normalized.starts_with("//") {
        format!("file:{normalized}")
    } else {
        format!("file:///{normalized}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_resolution_replaces_hi_with_lo() {
        assert_eq!(
            swap_resolution("ddd/concrete.hi.base", "lo").as_deref(),
            Some("ddd/concrete.lo.base")
        );
    }

    #[test]
    fn candidates_include_flat_filename() {
        let candidates = material_path_candidates("ddd/concrete.hi.base");
        assert!(candidates.contains(&"ddd/concrete.hi.base".to_string()));
        assert!(candidates.contains(&"concrete.hi.base".to_string()));
    }
}
