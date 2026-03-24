use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// dunce::canonicalize returns normal paths on Windows (e.g. C:\...) instead of
// the UNC-prefixed form (\\?\C:\...) that std::fs::canonicalize returns.
// This is required so that paths sent to the frontend via Tauri's asset protocol work correctly.
use dunce::canonicalize as fs_canonicalize;

pub fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| format!("Erreur création dossier {:?}: {}", path, e))
}

pub fn ensure_and_canonicalize_dir(path: &Path) -> Result<PathBuf, String> {
    ensure_dir(path)?;
    fs_canonicalize(path).map_err(|e| format!("Erreur canonicalisation dossier {:?}: {}", path, e))
}

pub fn canonicalize_existing_path(path: &Path) -> Result<PathBuf, String> {
    fs_canonicalize(path).map_err(|e| format!("Erreur canonicalisation chemin {:?}: {}", path, e))
}

pub fn safe_segment(input: &str, field_name: &str) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(format!("Le champ '{}' est vide", field_name));
    }

    if trimmed.contains('\0')
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains("..")
        || trimmed.contains(':')
    {
        return Err(format!(
            "Le champ '{}' contient des caractères interdits",
            field_name
        ));
    }

    Ok(trimmed.split_whitespace().collect::<Vec<_>>().join("_"))
}

pub fn safe_relative_from_root(path: &Path, root: &Path) -> Result<PathBuf, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("Le chemin {:?} sort de la racine {:?}", path, root))?;

    for component in relative.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            _ => {
                return Err(format!(
                    "Le chemin relatif {:?} contient des composants invalides",
                    relative
                ))
            }
        }
    }

    Ok(relative.to_path_buf())
}

pub fn canonicalize_existing_roots(roots: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut canonical_roots = Vec::new();
    for root in roots {
        if root.trim().is_empty() {
            continue;
        }
        let path = PathBuf::from(root);
        if !path.exists() {
            continue;
        }
        canonical_roots.push(canonicalize_existing_path(&path)?);
    }
    Ok(canonical_roots)
}

pub fn canonicalize_in_allowed_roots(
    path: &Path,
    allowed_roots: &[PathBuf],
) -> Result<PathBuf, String> {
    let canonical_path = canonicalize_existing_path(path)?;
    if allowed_roots
        .iter()
        .any(|root| canonical_path.starts_with(root))
    {
        return Ok(canonical_path);
    }

    Err(format!(
        "Le chemin {:?} est hors des racines autorisées",
        canonical_path
    ))
}

pub fn backup_file_with_timestamp(path: &Path) -> Result<PathBuf, String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Nom de fichier invalide pour backup".to_string())?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Erreur horodatage backup: {}", e))?
        .as_secs();

    let backup_name = format!("{}.bak.{}", file_name, ts);
    let backup_path = path.with_file_name(backup_name);
    fs::copy(path, &backup_path)
        .map_err(|e| format!("Erreur création backup {:?}: {}", backup_path, e))?;

    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, remove_dir_all, File};

    fn tmp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("archive-metz-test-{}-{}", name, std::process::id()))
    }

    #[test]
    fn safe_segment_rejects_traversal_and_separators() {
        assert!(safe_segment("../hack", "site").is_err());
        assert!(safe_segment("abc/def", "site").is_err());
        assert!(safe_segment("abc\\def", "site").is_err());
        assert!(safe_segment("C:drive", "site").is_err());
    }

    #[test]
    fn safe_segment_normalizes_whitespace() {
        let value = safe_segment("  Site   Test  ", "site").expect("segment should be valid");
        assert_eq!(value, "Site_Test");
    }

    #[test]
    fn canonicalize_in_allowed_roots_blocks_outside_paths() {
        let root = tmp_dir("root");
        let inside = root.join("inside");
        let outside = tmp_dir("outside");
        create_dir_all(&inside).expect("create inside");
        create_dir_all(&outside).expect("create outside");

        let allowed = vec![canonicalize_existing_path(&root).expect("canonical root")];
        assert!(canonicalize_in_allowed_roots(&inside, &allowed).is_ok());
        assert!(canonicalize_in_allowed_roots(&outside, &allowed).is_err());

        remove_dir_all(&root).expect("cleanup root");
        remove_dir_all(&outside).expect("cleanup outside");
    }

    #[test]
    fn safe_relative_from_root_rejects_escape() {
        let root = PathBuf::from("/tmp/a");
        let invalid = PathBuf::from("/tmp/b/file");
        assert!(safe_relative_from_root(&invalid, &root).is_err());
    }

    #[test]
    fn backup_file_creates_timestamped_copy() {
        let root = tmp_dir("backup");
        create_dir_all(&root).expect("create root");
        let file = root.join("metadata-presets.json");
        File::create(&file).expect("create file");
        let backup = backup_file_with_timestamp(&file).expect("backup");
        assert!(backup.exists());
        remove_dir_all(&root).expect("cleanup root");
    }
}
