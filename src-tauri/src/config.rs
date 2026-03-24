use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

/// Configuration globale de l'application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub depot_path: String,
    pub validation_path: String,
    pub archive_path: String,
    /// Dossier Paramètres : contient metadata-presets.json, audit.log, revision-tags.json
    pub settings_path: String,
    pub admin_password: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            depot_path: "".to_string(),
            validation_path: "".to_string(),
            archive_path: "".to_string(),
            settings_path: "".to_string(),
            admin_password: "1".to_string(),
        }
    }
}

impl AppConfig {
    /// Chemin vers metadata-presets.json dans le dossier Paramètres
    pub fn presets_file(&self) -> PathBuf {
        PathBuf::from(&self.settings_path).join("metadata-presets.json")
    }

    /// Chemin vers audit.log dans le dossier Paramètres
    pub fn logs_file(&self) -> PathBuf {
        PathBuf::from(&self.settings_path).join("audit.log")
    }

    /// Chemin vers revision-tags.json dans le dossier Paramètres
    pub fn revision_tags_file_path(&self) -> PathBuf {
        PathBuf::from(&self.settings_path).join("revision-tags.json")
    }

    /// Charge la configuration depuis Tauri Store
    pub fn load_from_store(app_handle: &AppHandle) -> Result<Self, String> {
        let defaults = AppConfig::default();
        let store = app_handle
            .store("settings.json")
            .map_err(|e| format!("Erreur chargement store: {}", e))?;

        let depot_path = store
            .get("depot_path")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let validation_path = store
            .get("validation_path")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let archive_path = store
            .get("archive_path")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let settings_path = store
            .get("settings_path")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let admin_password = store
            .get("admin_password")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        Ok(AppConfig {
            depot_path: if depot_path.trim().is_empty() {
                defaults.depot_path
            } else {
                depot_path
            },
            validation_path: if validation_path.trim().is_empty() {
                defaults.validation_path
            } else {
                validation_path
            },
            archive_path: if archive_path.trim().is_empty() {
                defaults.archive_path
            } else {
                archive_path
            },
            settings_path: if settings_path.trim().is_empty() {
                defaults.settings_path
            } else {
                settings_path
            },
            admin_password: if admin_password.trim().is_empty() {
                defaults.admin_password
            } else {
                admin_password
            },
        })
    }

    /// Sauvegarde la configuration dans Tauri Store
    pub fn save_to_store(&self, app_handle: &AppHandle) -> Result<(), String> {
        let store = app_handle
            .store("settings.json")
            .map_err(|e| format!("Erreur chargement store: {}", e))?;

        store.set("depot_path", serde_json::json!(&self.depot_path));
        store.set("validation_path", serde_json::json!(&self.validation_path));
        store.set("archive_path", serde_json::json!(&self.archive_path));
        store.set("settings_path", serde_json::json!(&self.settings_path));
        store.set("admin_password", serde_json::json!(&self.admin_password));

        store
            .save()
            .map_err(|e| format!("Erreur persistance store: {}", e))?;

        Ok(())
    }

    /// Valide que tous les chemins configurés existent et sont accessibles
    pub fn validate_paths(&self) -> PathValidationResult {
        let mut result = PathValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Vérifier depot_path
        if self.depot_path.is_empty() {
            result
                .warnings
                .push("Chemin Dépôt non configuré".to_string());
        } else if !PathBuf::from(&self.depot_path).exists() {
            result
                .errors
                .push(format!("Dépôt introuvable: {}", self.depot_path));
            result.valid = false;
        }

        // Vérifier validation_path
        if self.validation_path.is_empty() {
            result
                .warnings
                .push("Chemin Validation non configuré".to_string());
        } else if !PathBuf::from(&self.validation_path).exists() {
            result
                .errors
                .push(format!("Validation introuvable: {}", self.validation_path));
            result.valid = false;
        }

        // Vérifier archive_path
        if self.archive_path.is_empty() {
            result
                .warnings
                .push("Chemin Archive non configuré".to_string());
        } else if !PathBuf::from(&self.archive_path).exists() {
            result
                .errors
                .push(format!("Archive introuvable: {}", self.archive_path));
            result.valid = false;
        }

        // Vérifier settings_path (dossier Paramètres)
        if self.settings_path.is_empty() {
            result
                .warnings
                .push("Dossier Paramètres non configuré".to_string());
        } else {
            let settings_dir = PathBuf::from(&self.settings_path);
            if !settings_dir.exists() {
                result.warnings.push(format!(
                    "Dossier Paramètres introuvable, sera créé automatiquement: {}",
                    self.settings_path
                ));
            } else if !settings_dir.is_dir() {
                result.errors.push(format!(
                    "Le chemin Paramètres n'est pas un dossier: {}",
                    self.settings_path
                ));
                result.valid = false;
            }
        }

        result
    }
}

/// Résultat de validation des chemins
#[derive(Debug, Serialize, Deserialize)]
pub struct PathValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// État global de configuration partagé entre toutes les commandes
pub type AppConfigState = Mutex<AppConfig>;

/// Commande Tauri pour vérifier le mot de passe admin
#[tauri::command]
pub fn verify_admin_password(
    config_state: tauri::State<AppConfigState>,
    password: String,
) -> Result<bool, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    Ok(config.admin_password == password)
}

/// Commande Tauri pour récupérer la configuration actuelle
#[tauri::command]
pub fn get_app_config(config_state: tauri::State<AppConfigState>) -> Result<AppConfig, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    Ok(config.clone())
}

/// Commande Tauri pour mettre à jour la configuration
#[tauri::command]
pub fn update_app_config(
    app_handle: AppHandle,
    config_state: tauri::State<AppConfigState>,
    new_config: AppConfig,
) -> Result<(), String> {
    new_config.save_to_store(&app_handle)?;

    let mut config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    *config = new_config;

    Ok(())
}

/// Commande Tauri pour valider les chemins configurés
#[tauri::command]
pub fn validate_config_paths(
    config_state: tauri::State<AppConfigState>,
) -> Result<PathValidationResult, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    Ok(config.validate_paths())
}

/// Commande Tauri pour valider une configuration candidate sans la sauvegarder.
#[tauri::command]
pub fn preview_validate_config_paths(new_config: AppConfig) -> Result<PathValidationResult, String> {
    Ok(new_config.validate_paths())
}

/// Crée automatiquement le dossier Paramètres et les 3 fichiers qu'il contient
/// (metadata-presets.json, audit.log, revision-tags.json) s'ils sont absents.
#[tauri::command]
pub fn ensure_default_files(config_state: tauri::State<AppConfigState>) -> Result<(), String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    if config.settings_path.is_empty() {
        return Ok(());
    }

    // Créer le dossier Paramètres s'il n'existe pas
    let settings_dir = PathBuf::from(&config.settings_path);
    if !settings_dir.exists() {
        std::fs::create_dir_all(&settings_dir)
            .map_err(|e| format!("Erreur création dossier Paramètres: {}", e))?;
    }

    // Auto-créer metadata-presets.json si absent
    let presets_file = config.presets_file();
    if !presets_file.exists() {
        let default_presets = serde_json::json!({
            "operations": [],
            "structure_types": [],
            "operation_types": [],
            "software_types": [],
            "sites": [],
            "responsables": [],
            "model_authors": [],
            "depositors": []
        });
        std::fs::write(
            &presets_file,
            serde_json::to_string_pretty(&default_presets)
                .map_err(|e| format!("Erreur sérialisation presets: {}", e))?,
        )
        .map_err(|e| format!("Erreur création metadata-presets.json: {}", e))?;
    }

    // Auto-créer audit.log si absent
    let audit_file = config.logs_file();
    if !audit_file.exists() {
        std::fs::write(&audit_file, "")
            .map_err(|e| format!("Erreur création audit.log: {}", e))?;
    }

    // Auto-créer revision-tags.json si absent
    let tags_file = config.revision_tags_file_path();
    if !tags_file.exists() {
        std::fs::write(&tags_file, "{}")
            .map_err(|e| format!("Erreur création revision-tags.json: {}", e))?;
    }

    Ok(())
}
