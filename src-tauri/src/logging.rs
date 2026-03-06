use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use crate::config::AppConfig;

/// Types d'actions auditées
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditAction {
    Deposit,
    Validate,
    Archive,
    OverrideArchive,
    Edit,
    Delete,
}

/// Entrée d'audit complète
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String, // ISO 8601
    pub action: AuditAction,
    pub user: String,
    pub structure_path: String,
    pub metadata: Option<serde_json::Value>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl AuditEntry {
    /// Crée une nouvelle entrée d'audit
    pub fn new(
        action: AuditAction,
        structure_path: String,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            action,
            user: get_system_username(),
            structure_path,
            metadata,
            success: true,
            error: None,
        }
    }

    /// Marque l'entrée comme échouée avec une erreur
    #[allow(dead_code)]
    pub fn with_error(mut self, error: String) -> Self {
        self.success = false;
        self.error = Some(error);
        self
    }
}

/// Récupère le nom d'utilisateur du système
fn get_system_username() -> String {
    whoami::username()
}

/// Écrit une entrée d'audit dans le fichier de logs
pub fn log_audit(config: &AppConfig, entry: &AuditEntry) -> Result<(), String> {
    // logs_path pointe directement vers audit.log
    let audit_file = PathBuf::from(&config.logs_path);

    // Créer le dossier parent s'il n'existe pas
    if let Some(parent) = audit_file.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Erreur création dossier logs: {}", e))?;
        }
    }

    // Ouvrir le fichier en mode append
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&audit_file)
        .map_err(|e| format!("Erreur ouverture audit.log: {}", e))?;

    // Sérialiser l'entrée en JSON (une ligne)
    let json_line =
        serde_json::to_string(entry).map_err(|e| format!("Erreur sérialisation audit: {}", e))?;

    // Écrire la ligne avec un newline
    writeln!(file, "{}", json_line).map_err(|e| format!("Erreur écriture audit.log: {}", e))?;

    Ok(())
}

/// Filtre pour la lecture des logs
#[derive(Debug, Clone, Deserialize)]
pub struct LogFilter {
    #[serde(default)]
    pub action: Option<AuditAction>,
    #[serde(default)]
    pub structure_id: Option<String>,
    #[serde(default)]
    pub success_only: bool,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Lit les entrées d'audit depuis le fichier de logs
pub fn read_audit_logs(config: &AppConfig, filter: LogFilter) -> Result<Vec<AuditEntry>, String> {
    let audit_file = PathBuf::from(&config.logs_path);

    // Si le fichier n'existe pas, retourner une liste vide
    if !audit_file.exists() {
        return Ok(Vec::new());
    }

    // Lire toutes les lignes
    let content = std::fs::read_to_string(&audit_file)
        .map_err(|e| format!("Erreur lecture audit.log: {}", e))?;

    let mut entries: Vec<AuditEntry> = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Parser chaque ligne JSON
        match serde_json::from_str::<AuditEntry>(line) {
            Ok(entry) => {
                // Appliquer les filtres
                if let Some(ref action) = filter.action {
                    if !matches_action(&entry.action, action) {
                        continue;
                    }
                }

                if let Some(ref structure_id) = filter.structure_id {
                    if !entry.structure_path.contains(structure_id) {
                        continue;
                    }
                }

                if filter.success_only && !entry.success {
                    continue;
                }

                entries.push(entry);
            }
            Err(e) => {
                // Log l'erreur mais continue la lecture
                eprintln!("Erreur parsing ligne audit: {}", e);
            }
        }
    }

    // Inverser pour avoir les plus récents en premier
    entries.reverse();

    // Appliquer la limite
    if let Some(limit) = filter.limit {
        entries.truncate(limit);
    }

    Ok(entries)
}

/// Compare deux actions pour le filtrage
fn matches_action(a: &AuditAction, b: &AuditAction) -> bool {
    matches!(
        (a, b),
        (AuditAction::Deposit, AuditAction::Deposit)
            | (AuditAction::Validate, AuditAction::Validate)
            | (AuditAction::Archive, AuditAction::Archive)
            | (AuditAction::OverrideArchive, AuditAction::OverrideArchive)
            | (AuditAction::Edit, AuditAction::Edit)
            | (AuditAction::Delete, AuditAction::Delete)
    )
}

/// Commande Tauri pour lire les logs d'audit
#[tauri::command]
pub fn get_audit_logs(
    config_state: tauri::State<crate::config::AppConfigState>,
    filter: LogFilter,
) -> Result<Vec<AuditEntry>, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    read_audit_logs(&config, filter)
}

/// Commande Tauri pour obtenir les dernières validations
#[tauri::command]
pub fn get_recent_validations(
    config_state: tauri::State<crate::config::AppConfigState>,
    limit: usize,
) -> Result<Vec<AuditEntry>, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    let filter = LogFilter {
        action: None,
        structure_id: None,
        success_only: true,
        limit: None,
    };
    let mut entries = read_audit_logs(&config, filter)?;
    entries.retain(|entry| {
        matches!(
            entry.action,
            AuditAction::Archive | AuditAction::OverrideArchive
        )
    });
    entries.truncate(limit);
    Ok(entries)
}

/// Commande Tauri pour purger l'historique des validations (archive / overridearchive)
#[tauri::command]
pub fn reset_validation_history(
    config_state: tauri::State<crate::config::AppConfigState>,
) -> Result<(), String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    let audit_file = PathBuf::from(&config.logs_path);
    if !audit_file.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&audit_file)
        .map_err(|e| format!("Erreur lecture audit.log: {}", e))?;

    let mut kept_lines: Vec<String> = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<AuditEntry>(line) {
            Ok(entry) => {
                let is_validation_history = matches!(
                    entry.action,
                    AuditAction::Archive | AuditAction::OverrideArchive
                );
                if !is_validation_history {
                    kept_lines.push(line.to_string());
                }
            }
            Err(_) => {
                // Conserver les lignes non parseables pour éviter toute perte inattendue.
                kept_lines.push(line.to_string());
            }
        }
    }

    let output = if kept_lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", kept_lines.join("\n"))
    };
    std::fs::write(&audit_file, output)
        .map_err(|e| format!("Erreur écriture audit.log: {}", e))?;

    Ok(())
}
