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
    pub presets_path: String,
    pub logs_path: String,
    pub revision_tags_path: String,
    pub admin_password: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            depot_path: "".to_string(),
            validation_path: "".to_string(),
            archive_path: "".to_string(),
            presets_path: "".to_string(),
            logs_path: "".to_string(),
            revision_tags_path: "".to_string(),
            admin_password: "1".to_string(), // Mot de passe par défaut
        }
    }
}

impl AppConfig {
    /// Charge la configuration depuis Tauri Store
    pub fn load_from_store(app_handle: &AppHandle) -> Result<Self, String> {
        let defaults = AppConfig::default();
        let store = app_handle
            .store("settings.json")
            .map_err(|e| format!("Erreur chargement store: {}", e))?;

        // Charger les chemins depuis le store
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

        let presets_path = store
            .get("presets_path")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let logs_path = store
            .get("logs_path")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let revision_tags_path = store
            .get("revision_tags_path")
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
            presets_path: if presets_path.trim().is_empty() {
                defaults.presets_path
            } else {
                presets_path
            },
            logs_path: if logs_path.trim().is_empty() {
                defaults.logs_path
            } else {
                logs_path
            },
            revision_tags_path: if revision_tags_path.trim().is_empty() {
                defaults.revision_tags_path
            } else {
                revision_tags_path
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

        // L'API store.set() ne retourne pas de Result, donc on ne peut pas utiliser map_err
        store.set("depot_path", serde_json::json!(&self.depot_path));
        store.set("validation_path", serde_json::json!(&self.validation_path));
        store.set("archive_path", serde_json::json!(&self.archive_path));
        store.set("presets_path", serde_json::json!(&self.presets_path));
        store.set("logs_path", serde_json::json!(&self.logs_path));
        store.set("revision_tags_path", serde_json::json!(&self.revision_tags_path));
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

        // Vérifier presets_path (fichier, pas dossier)
        if self.presets_path.is_empty() {
            result
                .warnings
                .push("Chemin metadata-presets.json non configuré".to_string());
        } else {
            let presets_file = PathBuf::from(&self.presets_path);
            if !presets_file.exists() {
                result.warnings.push(format!(
                    "Fichier metadata-presets.json introuvable (sera créé): {}",
                    self.presets_path
                ));
            } else if !presets_file.is_file() {
                result.errors.push(format!(
                    "Le chemin presets n'est pas un fichier: {}",
                    self.presets_path
                ));
                result.valid = false;
            } else if let Ok(content) = std::fs::read_to_string(&presets_file) {
                if serde_json::from_str::<serde_json::Value>(&content).is_err() {
                    result.warnings.push(format!(
                        "Le fichier metadata-presets.json est invalide et sera régénéré avec backup: {}",
                        self.presets_path
                    ));
                }
            }
        }

        // Vérifier logs_path (chemin direct vers le fichier audit.log)
        if self.logs_path.is_empty() {
            result
                .warnings
                .push("Fichier audit.log non configuré".to_string());
        } else {
            let audit_file = PathBuf::from(&self.logs_path);
            if !audit_file.exists() {
                result.warnings.push(format!(
                    "Fichier audit.log introuvable, sera créé automatiquement: {}",
                    self.logs_path
                ));
            } else if !audit_file.is_file() {
                result.errors.push(format!(
                    "Le chemin logs n'est pas un fichier: {}",
                    self.logs_path
                ));
                result.valid = false;
            }
        }

        // Vérifier revision_tags_path (chemin direct vers le fichier revision-tags.json)
        if self.revision_tags_path.is_empty() {
            result
                .warnings
                .push("Fichier revision-tags.json non configuré".to_string());
        } else {
            let tags_file = PathBuf::from(&self.revision_tags_path);
            if !tags_file.exists() {
                result.warnings.push(format!(
                    "Fichier revision-tags.json introuvable (sera créé): {}",
                    self.revision_tags_path
                ));
            } else if !tags_file.is_file() {
                result.errors.push(format!(
                    "Le chemin revision_tags n'est pas un fichier: {}",
                    self.revision_tags_path
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
    // Sauvegarder dans le store
    new_config.save_to_store(&app_handle)?;

    // Mettre à jour l'état en mémoire
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

/// Crée automatiquement les fichiers par défaut (metadata-presets.json et audit.log)
/// si les chemins sont configurés mais les fichiers absents.
#[tauri::command]
pub fn ensure_default_files(config_state: tauri::State<AppConfigState>) -> Result<(), String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    // Auto-créer metadata-presets.json si configuré et absent
    if !config.presets_path.is_empty() {
        let presets_file = PathBuf::from(&config.presets_path);
        if !presets_file.exists() {
            if let Some(parent) = presets_file.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Erreur création dossier presets: {}", e))?;
                }
            }
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
    }

    // Auto-créer audit.log si configuré et absent
    if !config.logs_path.is_empty() {
        let audit_file = PathBuf::from(&config.logs_path);
        if !audit_file.exists() {
            if let Some(parent) = audit_file.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Erreur création dossier logs: {}", e))?;
                }
            }
            std::fs::write(&audit_file, "")
                .map_err(|e| format!("Erreur création audit.log: {}", e))?;
        }
    }

    // Auto-créer revision-tags.json si configuré et absent
    if !config.revision_tags_path.is_empty() {
        let tags_file = PathBuf::from(&config.revision_tags_path);
        if !tags_file.exists() {
            if let Some(parent) = tags_file.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Erreur création dossier revision-tags: {}", e))?;
                }
            }
            std::fs::write(&tags_file, "{}")
                .map_err(|e| format!("Erreur création revision-tags.json: {}", e))?;
        }
    }

    Ok(())
}
