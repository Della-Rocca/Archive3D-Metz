use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::{config::AppConfig, DepositMetadata};

/// Calcule la taille totale d'un dossier de manière récursive (en bytes)
fn calculate_directory_size_bytes(path: &Path) -> u64 {
    if !path.exists() || !path.is_dir() {
        return 0;
    }

    let mut total: u64 = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Ok(meta) = fs::metadata(&entry_path) {
                    total += meta.len();
                }
            } else if entry_path.is_dir() {
                total += calculate_directory_size_bytes(&entry_path);
            }
        }
    }
    total
}

/// Statistiques globales de l'archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStatistics {
    pub total_structures: usize,
    pub total_polygons: u64,
    pub total_photos: u64,
    #[serde(default)]
    pub total_size_bytes: u64,
    #[serde(default)]
    pub total_models_size_bytes: u64,
    #[serde(default)]
    pub total_photos_size_bytes: u64,
    pub by_structure_type: HashMap<String, usize>,
    pub by_operation: HashMap<String, usize>,
    pub by_author: HashMap<String, usize>,
    #[serde(default)]
    pub by_year: HashMap<String, usize>,
}

impl Default for ArchiveStatistics {
    fn default() -> Self {
        Self {
            total_structures: 0,
            total_polygons: 0,
            total_photos: 0,
            total_size_bytes: 0,
            total_models_size_bytes: 0,
            total_photos_size_bytes: 0,
            by_structure_type: HashMap::new(),
            by_operation: HashMap::new(),
            by_author: HashMap::new(),
            by_year: HashMap::new(),
        }
    }
}

/// Calcule les statistiques pour toutes les structures de l'archive
pub fn calculate_archive_statistics(config: &AppConfig) -> Result<ArchiveStatistics, String> {
    let mut stats = ArchiveStatistics::default();

    let archive_path = Path::new(&config.archive_path);
    if !archive_path.exists() {
        return Ok(stats);
    }

    // Parcourir tous les dossiers d'opération
    for op_entry in
        fs::read_dir(archive_path).map_err(|e| format!("Erreur lecture archive: {}", e))?
    {
        let op_entry = op_entry.map_err(|e| format!("Erreur entrée opération: {}", e))?;
        let op_path = op_entry.path();

        if !op_path.is_dir() {
            continue;
        }

        // Parcourir toutes les structures de cette opération
        for st_entry in
            fs::read_dir(&op_path).map_err(|e| format!("Erreur lecture structures: {}", e))?
        {
            let st_entry = st_entry.map_err(|e| format!("Erreur entrée structure: {}", e))?;
            let st_path = st_entry.path();

            if !st_path.is_dir() {
                continue;
            }

            stats.total_structures += 1;

            // Calculer les tailles de dossiers
            stats.total_size_bytes += calculate_directory_size_bytes(&st_path);
            let models_dir = st_path.join("Modeles");
            stats.total_models_size_bytes += calculate_directory_size_bytes(&models_dir);
            let orthos_dir = st_path.join("Orthomosaique");
            let photos_dir = st_path.join("DossierProduction").join("Photos");
            stats.total_photos_size_bytes +=
                calculate_directory_size_bytes(&orthos_dir) + calculate_directory_size_bytes(&photos_dir);

            // Lire les métadonnées
            let metadata_path = st_path.join("metadata.json");
            if let Ok(content) = fs::read_to_string(&metadata_path) {
                if let Ok(metadata) = serde_json::from_str::<DepositMetadata>(&content) {
                    // Type de structure
                    *stats
                        .by_structure_type
                        .entry(metadata.structure.st_type.clone())
                        .or_insert(0) += 1;

                    // Opération
                    let op_key =
                        format!("{} ({})", metadata.operation.code, metadata.operation.site);
                    *stats.by_operation.entry(op_key).or_insert(0) += 1;

                    // Auteur
                    if !metadata.structure.model_author.is_empty() {
                        *stats
                            .by_author
                            .entry(metadata.structure.model_author.clone())
                            .or_insert(0) += 1;
                    }

                    // Année (extraite du code d'opération, ex: 202206 -> 2022)
                    let year = metadata
                        .operation
                        .code
                        .chars()
                        .take(4)
                        .collect::<String>();
                    let year_key = if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
                        year
                    } else {
                        "Inconnue".to_string()
                    };
                    *stats.by_year.entry(year_key).or_insert(0) += 1;

                    // Polygones
                    if let Ok(faces) = metadata.structure.faces_count.parse::<u64>() {
                        stats.total_polygons += faces;
                    }

                    // Photos
                    if let Ok(photos) = metadata.structure.photos_count.parse::<u64>() {
                        stats.total_photos += photos;
                    }
                }
            }
        }
    }

    Ok(stats)
}

/// Commande Tauri pour obtenir les statistiques
#[tauri::command]
pub fn get_archive_statistics(
    config_state: tauri::State<crate::config::AppConfigState>,
) -> Result<ArchiveStatistics, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    calculate_archive_statistics(&config)
}
