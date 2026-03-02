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
    pub admin_password: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        // Valeurs par défaut pour rétrocompatibilité
        let default_base = "/Users/francoisdesenneville/Desktop/App-Archive/SpacesData";
        Self {
            depot_path: format!("{}/Depot", default_base),
            validation_path: format!("{}/Validation", default_base),
            archive_path: format!("{}/Archive", default_base),
            presets_path: format!("{}/Archive/metadata-presets.json", default_base),
            logs_path: format!("{}/logs", default_base),
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

        // Vérifier logs_path
        if self.logs_path.is_empty() {
            result
                .warnings
                .push("Chemin logs non configuré".to_string());
        } else {
            let logs_dir = PathBuf::from(&self.logs_path);
            if !logs_dir.exists() {
                result.warnings.push(format!(
                    "Dossier logs introuvable (sera créé): {}",
                    self.logs_path
                ));
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
