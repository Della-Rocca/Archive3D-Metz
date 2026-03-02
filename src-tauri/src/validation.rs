use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::DepositMetadata;

/// Niveau de sévérité d'un problème de validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,   // Bloque la validation
    Warning, // N'empêche pas la validation mais signale un problème
}

/// Un problème détecté lors de la validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub category: String, // ex: "metadata", "files", "model"
    pub message: String,
}

/// Résultat complet de la validation d'une structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool, // True si aucune erreur critique
    pub issues: Vec<ValidationIssue>,
    pub warnings_count: usize,
    pub errors_count: usize,
}

impl ValidationReport {
    fn new() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
            warnings_count: 0,
            errors_count: 0,
        }
    }

    fn add_error(&mut self, category: &str, message: String) {
        self.valid = false;
        self.errors_count += 1;
        self.issues.push(ValidationIssue {
            severity: IssueSeverity::Error,
            category: category.to_string(),
            message,
        });
    }

    fn add_warning(&mut self, category: &str, message: String) {
        self.warnings_count += 1;
        self.issues.push(ValidationIssue {
            severity: IssueSeverity::Warning,
            category: category.to_string(),
            message,
        });
    }
}

/// Valide une structure complète
pub fn validate_structure(structure_path: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::new();

    if !structure_path.exists() {
        report.add_error(
            "structure",
            "Le dossier de structure n'existe pas".to_string(),
        );
        return Ok(report);
    }

    // 1. Vérifier la présence de metadata.json
    let metadata_path = structure_path.join("metadata.json");
    if !metadata_path.exists() {
        report.add_error("metadata", "Fichier metadata.json manquant".to_string());
        return Ok(report);
    }

    // 2. Lire et valider les métadonnées
    let metadata_result = fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Erreur lecture metadata.json: {}", e))?;

    let metadata: DepositMetadata = match serde_json::from_str(&metadata_result) {
        Ok(m) => m,
        Err(e) => {
            report.add_error("metadata", format!("Fichier metadata.json invalide: {}", e));
            return Ok(report);
        }
    };

    // 3. Valider les champs obligatoires des métadonnées
    validate_metadata_fields(&metadata, &mut report);

    // 4. Vérifier la présence des dossiers
    let models_dir = structure_path.join("Modeles");
    let orthos_dir = structure_path.join("Orthomosaique");
    let prod_dir = structure_path.join("DossierProduction");
    let photos_dir = prod_dir.join("Photos");
    let travail_dir = prod_dir.join("FichiersTravail");

    if !models_dir.exists() {
        report.add_warning("files", "Dossier 'Modeles' manquant".to_string());
    }
    if !orthos_dir.exists() {
        report.add_warning("files", "Dossier 'Orthomosaique' manquant".to_string());
    }
    if !prod_dir.exists() {
        report.add_warning("files", "Dossier 'DossierProduction' manquant".to_string());
    } else {
        if !photos_dir.exists() {
            report.add_warning(
                "files",
                "Sous-dossier 'DossierProduction/Photos' manquant".to_string(),
            );
        }
        if !travail_dir.exists() {
            report.add_warning(
                "files",
                "Sous-dossier 'DossierProduction/FichiersTravail' manquant".to_string(),
            );
        }
    }

    // 5. Vérifier la présence d'au moins un modèle 3D
    if models_dir.exists() {
        match count_files_in_dir(&models_dir) {
            Ok(count) => {
                if count == 0 {
                    report.add_error(
                        "files",
                        "Aucun fichier dans le dossier 'Modeles'".to_string(),
                    );
                }
            }
            Err(e) => {
                report.add_warning(
                    "files",
                    format!("Impossible de lire le dossier Modeles: {}", e),
                );
            }
        }
    }

    // 6. Vérifier la présence de fichiers dans les autres dossiers
    if orthos_dir.exists() {
        if let Ok(count) = count_files_in_dir(&orthos_dir) {
            if count == 0 {
                report.add_warning(
                    "files",
                    "Aucun fichier dans le dossier 'Orthomosaique'".to_string(),
                );
            }
        }
    }

    if photos_dir.exists() {
        if let Ok(count) = count_files_in_dir(&photos_dir) {
            if count == 0 {
                report.add_warning(
                    "files",
                    "Aucun fichier dans 'DossierProduction/Photos'".to_string(),
                );
            }
        }
    }

    if travail_dir.exists() {
        if let Ok(count) = count_files_in_dir(&travail_dir) {
            if count == 0 {
                report.add_warning(
                    "files",
                    "Aucun fichier dans 'DossierProduction/FichiersTravail'".to_string(),
                );
            }
        }
    }

    Ok(report)
}

/// Valide les champs obligatoires des métadonnées
fn validate_metadata_fields(metadata: &DepositMetadata, report: &mut ValidationReport) {
    // Opération
    if metadata.operation.code.trim().is_empty() {
        report.add_error("metadata", "Code opération manquant".to_string());
    }
    if metadata.operation.site.trim().is_empty() {
        report.add_error("metadata", "Site opération manquant".to_string());
    }
    if metadata.operation.op_type.trim().is_empty() {
        report.add_warning("metadata", "Type d'opération manquant".to_string());
    }
    if metadata.operation.responsable.trim().is_empty() {
        report.add_warning("metadata", "Responsable opération manquant".to_string());
    }

    // Structure
    if metadata.structure.id.trim().is_empty() {
        report.add_error("metadata", "ID structure manquant".to_string());
    }
    if metadata.structure.st_type.trim().is_empty() {
        report.add_error("metadata", "Type de structure manquant".to_string());
    }
    if metadata.structure.model_author.trim().is_empty() {
        report.add_warning("metadata", "Auteur du modèle 3D manquant".to_string());
    }
    if metadata.structure.software.trim().is_empty() {
        report.add_warning("metadata", "Logiciel utilisé manquant".to_string());
    }
}

/// Compte le nombre de fichiers dans un dossier
fn count_files_in_dir(path: &Path) -> Result<usize, String> {
    let mut count = 0;
    for entry in fs::read_dir(path).map_err(|e| format!("Erreur lecture dossier: {}", e))? {
        let entry = entry.map_err(|e| format!("Erreur entrée: {}", e))?;
        if entry.path().is_file() {
            count += 1;
        }
    }
    Ok(count)
}

/// Commande Tauri pour valider une structure
#[tauri::command]
pub fn validate_structure_check(structure_path: String) -> Result<ValidationReport, String> {
    let path = Path::new(&structure_path);
    validate_structure(path)
}

/// Commande Tauri pour mettre à jour les métadonnées d'une structure
#[tauri::command]
pub fn update_structure_metadata(
    config_state: tauri::State<crate::config::AppConfigState>,
    structure_path: String,
    updated_metadata: DepositMetadata,
) -> Result<(), String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    let allowed_roots = crate::fs_safety::canonicalize_existing_roots(&[
        config.depot_path.clone(),
        config.validation_path.clone(),
        config.archive_path.clone(),
    ])?;
    if allowed_roots.is_empty() {
        return Err("Aucune racine de stockage valide n'est configurée".to_string());
    }

    let path = crate::fs_safety::canonicalize_in_allowed_roots(
        Path::new(&structure_path),
        &allowed_roots,
    )?;
    let metadata_path = path.join("metadata.json");

    if !metadata_path.exists() {
        return Err("Fichier metadata.json introuvable".to_string());
    }

    // Sérialiser les nouvelles métadonnées
    let json = serde_json::to_string_pretty(&updated_metadata)
        .map_err(|e| format!("Erreur sérialisation metadata: {}", e))?;

    // Écrire le fichier
    fs::write(&metadata_path, json).map_err(|e| format!("Erreur écriture metadata.json: {}", e))?;

    // Mettre à jour les presets
    crate::update_presets_with_metadata(&config, &updated_metadata)?;

    // Logger l'opération
    let audit_entry = crate::logging::AuditEntry::new(
        crate::logging::AuditAction::Edit,
        structure_path,
        Some(serde_json::to_value(&updated_metadata).unwrap_or(serde_json::Value::Null)),
    );

    if let Err(e) = crate::logging::log_audit(&config, &audit_entry) {
        eprintln!("Avertissement: Échec log audit: {}", e);
    }

    Ok(())
}
