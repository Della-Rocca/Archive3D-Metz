// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod fs_safety;
mod logging;
mod statistics;
mod validation;

use config::{AppConfig, AppConfigState};
use fs_safety::{
    backup_file_with_timestamp, canonicalize_existing_path, canonicalize_existing_roots,
    canonicalize_in_allowed_roots, ensure_and_canonicalize_dir, ensure_dir,
    safe_relative_from_root, safe_segment,
};
use gltf::mesh::Mode;
use logging::{log_audit, AuditAction, AuditEntry};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use tauri::{Manager, State};

/// --------------------
///   STRUCTURES META
/// --------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperationMeta {
    pub code: String,    // ex: "202501"
    pub site: String,    // ex: "Metz-centre"
    pub op_type: String, // ex: "diagnostic" / "fouille"
    pub responsable: String, // ex: "Dupont Jean"
                         // (plus de date ni localisation)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StructureMeta {
    pub id: String,      // ex: "ST443"
    pub st_type: String, // ex: "sepulture", "mur", etc.

    // Description libre (optionnelle). Important : `serde(default)` permet
    // de lire d'anciens metadata.json qui n'ont pas ce champ.
    #[serde(default)]
    pub description: String,

    pub model_author: String,

    #[serde(default)]
    pub depositor: String, // auteur du dépôt

    pub photos_count: String,
    pub faces_count: String, // nombre de polygones (triangles ou équivalent)
    pub software: String,    // ex: "Metashape"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DepositMetadata {
    pub operation: OperationMeta,
    pub structure: StructureMeta,
}

/// Pour afficher la liste des structures
#[derive(Debug, Serialize)]
struct StructureListItem {
    operation_folder: String,
    structure_folder: String,
    path: String,
}

/// Détails d'une structure (pour consultation / validation / archive)
#[derive(Debug, Serialize)]
struct StructureDetails {
    metadata: Option<DepositMetadata>,
    metadata_raw: Option<serde_json::Value>,
    models: Vec<String>,
    orthos: Vec<String>,
    photos: Vec<String>,
    work_files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct StructureFileInfo {
    size_bytes: u64,
}

/// Item d'inventaire pour export CSV
#[derive(Debug, Serialize)]
struct InventoryItem {
    structure_id: String,
    operation_code: String,
    operation_site: String,
    operation_type: String,
    operation_responsable: String,
    structure_type: String,
    photos_count: String,
    photos_total_size_mb: String,
    model_size_mb: String,
    faces_count: String,
    model_author: String,
    depositor: String,
    software: String,
    deposit_date: String,
}

#[derive(Debug, Serialize)]
struct StructureSummaryItem {
    operation_code: String,
    operation_site: String,
    operation_type: String,
    responsable: String,
    structure_id: String,
    structure_type: String,
    model_author: String,
    path: String,
    has_model: bool,
    has_orthos: bool,
    has_production: bool,
    last_modified_unix: Option<u64>,
}

#[derive(Debug, Serialize)]
struct StructureSummaryResponse {
    total: usize,
    page: usize,
    per_page: usize,
    items: Vec<StructureSummaryItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RevisionTag {
    tagged: bool,
    #[serde(default)]
    note: String,
    updated_at: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct StructureSummaryFilters {
    query: Option<String>,
    operation_code: Option<String>,
    operation_site: Option<String>,
    operation_type: Option<String>,
    structure_id: Option<String>,
    structure_type: Option<String>,
    model_author: Option<String>,
    has_model: Option<bool>,
    has_orthos: Option<bool>,
    has_production: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct StructureSummaryPagination {
    page: usize,
    per_page: usize,
}

impl Default for StructureSummaryPagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 100,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct StructureSummarySort {
    field: String,
    direction: String,
}

impl Default for StructureSummarySort {
    fn default() -> Self {
        Self {
            field: "operation_code".to_string(),
            direction: "asc".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
struct RouteStructureResult {
    validation_report: validation::ValidationReport,
    archived: bool,
    validation_path: Option<String>,
    archive_path: Option<String>,
    override_used: bool,
}

/// Fichier de presets/ontologies
#[derive(Debug, Serialize, Deserialize, Clone)]
struct MetadataPresets {
    #[serde(default)]
    operations: Vec<OperationMeta>,
    #[serde(default)]
    structure_types: Vec<String>,
    #[serde(default)]
    operation_types: Vec<String>,
    #[serde(default)]
    software_types: Vec<String>,

    // 🔹 nouvelles ontologies dérivées des métadonnées enregistrées
    #[serde(default)]
    sites: Vec<String>,
    #[serde(default)]
    responsables: Vec<String>,
    #[serde(default)]
    model_authors: Vec<String>,
    #[serde(default)]
    depositors: Vec<String>,
}

/// --------------------
///   HELPERS FS
/// --------------------

fn copy_file_to_dir(src: &Path, dest_dir: &Path) -> Result<(), String> {
    let src_canonical = canonicalize_existing_path(src)?;
    if !src_canonical.is_file() {
        return Err(format!(
            "Le chemin source {:?} n'est pas un fichier",
            src_canonical
        ));
    }

    let file_name = src
        .file_name()
        .ok_or_else(|| "Fichier sans nom".to_string())?;
    let dest_path = dest_dir.join(file_name);

    fs::copy(&src_canonical, &dest_path)
        .map_err(|e| format!("Erreur copie {:?} -> {:?}: {}", src, dest_path, e))?;

    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    ensure_dir(dest)?;
    for entry in
        fs::read_dir(src).map_err(|e| format!("Erreur lecture dossier {:?}: {}", src, e))?
    {
        let entry = entry.map_err(|e| format!("Erreur entrée dossier: {}", e))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else if src_path.is_file() {
            fs::copy(&src_path, &dest_path)
                .map_err(|e| format!("Erreur copie {:?} -> {:?}: {}", src_path, dest_path, e))?;
        }
    }
    Ok(())
}

fn move_dir(src: &Path, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        ensure_dir(parent)?;
    }

    match fs::rename(src, dest) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::CrossesDevices => {
            copy_dir_recursive(src, dest)?;
            fs::remove_dir_all(src)
                .map_err(|rm_err| format!("Erreur nettoyage source {:?}: {}", src, rm_err))
        }
        Err(e) => Err(format!("Erreur déplacement {:?} -> {:?}: {}", src, dest, e)),
    }
}

fn list_files_in_dir(path: &Path) -> Result<Vec<String>, String> {
    if !path.exists() {
        return Ok(vec![]);
    }

    let mut files = Vec::new();
    for entry in
        fs::read_dir(path).map_err(|e| format!("Erreur lecture dossier {:?}: {}", path, e))?
    {
        let entry = entry.map_err(|e| format!("Erreur entrée dir: {}", e))?;
        let p = entry.path();
        if p.is_file() {
            let s = p
                .to_str()
                .ok_or_else(|| "Chemin fichier invalide".to_string())?
                .to_string();
            files.push(s);
        }
    }
    Ok(files)
}

fn has_any_file(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                return true;
            }
        }
    }

    false
}

fn get_modified_unix_seconds(path: &Path) -> Option<u64> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(UNIX_EPOCH).ok()?;
    Some(duration.as_secs())
}

/// Obtient la taille d'un fichier en Mo
fn get_file_size_mb(path: &Path) -> f64 {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.len() as f64 / (1024.0 * 1024.0)
    } else {
        0.0
    }
}

/// Calcule la taille totale d'un dossier de manière récursive (en bytes)
fn get_directory_size_bytes(path: &Path) -> u64 {
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
                total += get_directory_size_bytes(&entry_path);
            }
        }
    }
    total
}

/// Trouve le premier fichier modèle dans le dossier Modeles
fn find_first_model_file(models_dir: &Path) -> Option<PathBuf> {
    if !models_dir.exists() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if ext_lower == "glb" || ext_lower == "gltf" || ext_lower == "obj" {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

fn allowed_roots_from_config(config: &AppConfig) -> Result<Vec<PathBuf>, String> {
    canonicalize_existing_roots(&[
        config.depot_path.clone(),
        config.validation_path.clone(),
        config.archive_path.clone(),
    ])
}

fn canonicalize_structure_in_known_roots(
    config: &AppConfig,
    path: &str,
) -> Result<PathBuf, String> {
    let roots = allowed_roots_from_config(config)?;
    if roots.is_empty() {
        return Err("Aucune racine de stockage valide n'est configurée".to_string());
    }
    canonicalize_in_allowed_roots(Path::new(path), &roots)
}

/// --------------------
///   PRESETS / ONTOLOGIES
///   (fichier local partagé)
/// --------------------

fn metadata_presets_path(config: &AppConfig) -> PathBuf {
    PathBuf::from(&config.presets_path)
}

fn default_software_list() -> Vec<String> {
    vec![
        "Metashape".to_string(),
        "Reality Capture".to_string(),
        "MeshLab".to_string(),
        "CloudCompare".to_string(),
        "Meshroom".to_string(),
    ]
}

fn default_presets() -> MetadataPresets {
    MetadataPresets {
        operations: Vec::new(),
        structure_types: Vec::new(),
        operation_types: Vec::new(),
        software_types: default_software_list(),
        sites: Vec::new(),
        responsables: Vec::new(),
        model_authors: Vec::new(),
        depositors: Vec::new(),
    }
}

fn load_metadata_presets_internal(config: &AppConfig) -> Result<MetadataPresets, String> {
    let path = metadata_presets_path(config);

    if let Some(dir) = path.parent() {
        ensure_dir(dir)?;
    }

    if !path.exists() {
        let defaults = default_presets();
        save_metadata_presets_internal(config, &defaults)?;
        return Ok(defaults);
    }

    let text = fs::read_to_string(&path)
        .map_err(|e| format!("Erreur lecture metadata-presets.json: {}", e))?;

    let mut presets: MetadataPresets = match serde_json::from_str(&text) {
        Ok(parsed) => parsed,
        Err(parse_err) => {
            let backup_path = backup_file_with_timestamp(&path)?;
            eprintln!(
                "Avertissement: presets invalides, backup créé {:?}: {}",
                backup_path, parse_err
            );
            let defaults = default_presets();
            save_metadata_presets_internal(config, &defaults)?;
            defaults
        }
    };

    if presets.software_types.is_empty() {
        presets.software_types = default_software_list();
    }

    Ok(presets)
}

fn save_metadata_presets_internal(
    config: &AppConfig,
    presets: &MetadataPresets,
) -> Result<(), String> {
    let path = metadata_presets_path(config);
    if let Some(dir) = path.parent() {
        ensure_dir(dir)?;
    }

    let json = serde_json::to_string_pretty(presets)
        .map_err(|e| format!("Erreur sérialisation metadata-presets.json: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Erreur écriture metadata-presets.json: {}", e))
}

pub fn update_presets_with_metadata(
    config: &AppConfig,
    meta: &DepositMetadata,
) -> Result<(), String> {
    let mut presets = load_metadata_presets_internal(config)?;

    // 1) opérations (identifiant = code)
    if let Some(existing) = presets
        .operations
        .iter_mut()
        .find(|op| op.code == meta.operation.code)
    {
        *existing = meta.operation.clone();
    } else {
        presets.operations.push(meta.operation.clone());
    }

    // 2) types de structure
    if !meta.structure.st_type.is_empty()
        && !presets.structure_types.contains(&meta.structure.st_type)
    {
        presets.structure_types.push(meta.structure.st_type.clone());
    }

    // 3) types d'opération
    if !meta.operation.op_type.is_empty()
        && !presets.operation_types.contains(&meta.operation.op_type)
    {
        presets.operation_types.push(meta.operation.op_type.clone());
    }

    // 4) types de logiciels
    if !meta.structure.software.is_empty()
        && !presets.software_types.contains(&meta.structure.software)
    {
        presets.software_types.push(meta.structure.software.clone());
    }

    // 5) sites
    if !meta.operation.site.is_empty() && !presets.sites.contains(&meta.operation.site) {
        presets.sites.push(meta.operation.site.clone());
    }

    // 6) responsables
    if !meta.operation.responsable.is_empty()
        && !presets.responsables.contains(&meta.operation.responsable)
    {
        presets
            .responsables
            .push(meta.operation.responsable.clone());
    }

    // 7) auteurs de modèles
    if !meta.structure.model_author.is_empty()
        && !presets.model_authors.contains(&meta.structure.model_author)
    {
        presets
            .model_authors
            .push(meta.structure.model_author.clone());
    }

    // 8) déposants
    if !meta.structure.depositor.is_empty()
        && !presets.depositors.contains(&meta.structure.depositor)
    {
        presets.depositors.push(meta.structure.depositor.clone());
    }

    save_metadata_presets_internal(config, &presets)
}

#[tauri::command]
fn get_metadata_presets(config_state: State<AppConfigState>) -> Result<MetadataPresets, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    load_metadata_presets_internal(&config)
}

#[tauri::command]
fn update_metadata_presets(
    config_state: State<AppConfigState>,
    presets: MetadataPresets,
) -> Result<(), String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    save_metadata_presets_internal(&config, &presets)
}

fn move_structure_between_spaces(
    config: &AppConfig,
    source_path: &str,
    source_root: &str,
    destination_root: &str,
    action: AuditAction,
    audit_metadata: Option<serde_json::Value>,
) -> Result<String, String> {
    let source_root_canonical = canonicalize_existing_path(Path::new(source_root))?;
    let source_canonical = canonicalize_existing_path(Path::new(source_path))?;
    let relative = safe_relative_from_root(&source_canonical, &source_root_canonical)?;

    let destination_root_canonical = ensure_and_canonicalize_dir(Path::new(destination_root))?;
    let destination = destination_root_canonical.join(relative);
    if destination.exists() {
        return Err(format!(
            "Destination déjà existante, déplacement annulé: {}",
            destination.display()
        ));
    }
    move_dir(&source_canonical, &destination)?;

    let result_path = destination
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Impossible de convertir le chemin destination".to_string())?;

    let audit_entry = AuditEntry::new(action, result_path.clone(), audit_metadata);
    if let Err(e) = log_audit(config, &audit_entry) {
        eprintln!("Avertissement: Échec log audit: {}", e);
    }

    Ok(result_path)
}

fn revision_tags_file(config: &AppConfig) -> PathBuf {
    PathBuf::from(&config.logs_path).join("revision-tags.json")
}

fn read_revision_tags(config: &AppConfig) -> Result<std::collections::HashMap<String, RevisionTag>, String> {
    let file = revision_tags_file(config);
    if !file.exists() {
        return Ok(std::collections::HashMap::new());
    }
    let raw = fs::read_to_string(&file)
        .map_err(|e| format!("Erreur lecture revision-tags.json: {}", e))?;
    if raw.trim().is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    serde_json::from_str(&raw)
        .map_err(|e| format!("Erreur parsing revision-tags.json: {}", e))
}

fn write_revision_tags(
    config: &AppConfig,
    tags: &std::collections::HashMap<String, RevisionTag>,
) -> Result<(), String> {
    let logs_dir = PathBuf::from(&config.logs_path);
    if !logs_dir.exists() {
        fs::create_dir_all(&logs_dir)
            .map_err(|e| format!("Erreur création dossier logs: {}", e))?;
    }
    let file = revision_tags_file(config);
    let json = serde_json::to_string_pretty(tags)
        .map_err(|e| format!("Erreur sérialisation revision tags: {}", e))?;
    fs::write(file, json).map_err(|e| format!("Erreur écriture revision-tags.json: {}", e))
}

/// --------------------
///   COMMANDE DEPOT
/// --------------------
///
/// - crée l'arborescence dans SpacesData/Depot
/// - copie les fichiers dans Modeles / Orthomosaique / DossierProduction/{Photos,FichiersTravail}
/// - écrit un metadata.json pour la structure
/// - met à jour le fichier d'ontologies locales (operations, types, logiciels, sites, responsables, auteurs)
///
#[tauri::command]
fn deposit_structure(
    config_state: State<AppConfigState>,
    metadata: DepositMetadata,
    model_files: Vec<String>,
    ortho_files: Vec<String>,
    photo_files: Vec<String>,
    work_files: Vec<String>,
) -> Result<String, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    // Mise à jour des presets (fichier partagé)
    update_presets_with_metadata(&config, &metadata)?;

    let op_code = safe_segment(&metadata.operation.code, "operation.code")?;
    let op_site = safe_segment(&metadata.operation.site, "operation.site")?;
    let st_id = safe_segment(&metadata.structure.id, "structure.id")?;

    // Nom dossier opération : "202501_Site_test"
    let op_folder_name = format!("{}_{}", op_code, op_site);

    // Nom dossier structure : "ST443"
    let st_folder_name = format!("{}", st_id);

    // Racine : .../Depot/202501_Site_test/ST443_sepulture
    let base_depot = ensure_and_canonicalize_dir(Path::new(&config.depot_path))?;
    let op_dir = base_depot.join(op_folder_name);
    let st_dir = op_dir.join(st_folder_name);

    let models_dir = st_dir.join("Modeles");
    let orthos_dir = st_dir.join("Orthomosaique");
    let prod_dir = st_dir.join("DossierProduction");
    let photos_dir = prod_dir.join("Photos");
    let travail_dir = prod_dir.join("FichiersTravail");

    // Création des dossiers
    ensure_dir(&models_dir)?;
    ensure_dir(&orthos_dir)?;
    ensure_dir(&prod_dir)?;
    ensure_dir(&photos_dir)?;
    ensure_dir(&travail_dir)?;

    // Copie des modèles 3D
    for path in model_files {
        let src = Path::new(&path);
        copy_file_to_dir(src, &models_dir)?;
    }

    // Copie des orthophotos
    for path in ortho_files {
        let src = Path::new(&path);
        copy_file_to_dir(src, &orthos_dir)?;
    }

    // Copie des photos de production
    for path in photo_files {
        let src = Path::new(&path);
        copy_file_to_dir(src, &photos_dir)?;
    }

    // Copie des fichiers de travail
    for path in work_files {
        let src = Path::new(&path);
        copy_file_to_dir(src, &travail_dir)?;
    }

    // Écriture du metadata.json
    let metadata_path = st_dir.join("metadata.json");
    let json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Erreur sérialisation metadata: {}", e))?;

    fs::write(&metadata_path, json).map_err(|e| format!("Erreur écriture metadata.json: {}", e))?;

    // Obtenir le chemin pour le retour et les logs
    let result_path = st_dir
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Impossible de convertir le chemin de structure".to_string())?;

    // Logger l'opération réussie
    let audit_entry = AuditEntry::new(
        AuditAction::Deposit,
        result_path.clone(),
        Some(serde_json::to_value(&metadata).unwrap_or(serde_json::Value::Null)),
    );

    // Ne pas faire échouer l'opération si le log échoue
    if let Err(e) = log_audit(&config, &audit_entry) {
        eprintln!("Avertissement: Échec log audit: {}", e);
    }

    Ok(result_path)
}

/// ------------------------
///   COMMANDE VALIDATION
///   Depot -> Validation
/// ------------------------
#[tauri::command]
fn count_model_polygons(path: String) -> Result<u64, String> {
    let (document, _, _) =
        gltf::import(&path).map_err(|e| format!("Erreur lecture modèle 3D {:?}: {}", path, e))?;

    let mut polygons = 0u64;
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            if let Some(accessor) = primitive.indices() {
                let count = accessor.count() as u64;
                polygons += match primitive.mode() {
                    Mode::Triangles => count / 3,
                    Mode::TriangleStrip | Mode::TriangleFan => count.saturating_sub(2),
                    _ => 0,
                };
            }
        }
    }

    Ok(polygons)
}

#[tauri::command]
fn validate_structure_from_depot(
    config_state: State<AppConfigState>,
    structure_path_in_depot: String,
) -> Result<String, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    move_structure_between_spaces(
        &config,
        &structure_path_in_depot,
        &config.depot_path,
        &config.validation_path,
        AuditAction::Validate,
        None,
    )
}

#[tauri::command]
fn get_revision_tags(
    config_state: State<AppConfigState>,
) -> Result<std::collections::HashMap<String, RevisionTag>, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    read_revision_tags(&config)
}

#[tauri::command]
fn set_revision_tag(
    config_state: State<AppConfigState>,
    structure_path_in_depot: String,
    tagged: bool,
    note: Option<String>,
) -> Result<(), String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    let depot_root = canonicalize_existing_path(Path::new(&config.depot_path))?;
    let structure_path =
        canonicalize_in_allowed_roots(Path::new(&structure_path_in_depot), &[depot_root])?;
    let key = structure_path
        .to_str()
        .ok_or_else(|| "Chemin structure invalide".to_string())?
        .to_string();

    let mut tags = read_revision_tags(&config)?;
    if tagged {
        tags.insert(
            key,
            RevisionTag {
                tagged: true,
                note: note.unwrap_or_default(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        );
    } else {
        tags.remove(&key);
    }
    write_revision_tags(&config, &tags)
}

/// ------------------------
///   COMMANDE ARCHIVE
///   Validation -> Archive
/// ------------------------
#[tauri::command]
fn archive_structure_from_validation(
    config_state: State<AppConfigState>,
    structure_path_in_validation: String,
) -> Result<String, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    move_structure_between_spaces(
        &config,
        &structure_path_in_validation,
        &config.validation_path,
        &config.archive_path,
        AuditAction::Archive,
        None,
    )
}

/// ------------------------
///   COMMANDE ARCHIVE DIRECTE
///   Depot -> Archive
/// ------------------------
#[tauri::command]
fn archive_structure_from_depot(
    config_state: State<AppConfigState>,
    structure_path_in_depot: String,
) -> Result<String, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    move_structure_between_spaces(
        &config,
        &structure_path_in_depot,
        &config.depot_path,
        &config.archive_path,
        AuditAction::Archive,
        Some(serde_json::json!({ "mode": "direct_from_depot" })),
    )
}

#[tauri::command]
fn validate_then_route_structure(
    config_state: State<AppConfigState>,
    structure_path_in_depot: String,
    allow_override: bool,
    override_reason: Option<String>,
) -> Result<RouteStructureResult, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    let depot_root = canonicalize_existing_path(Path::new(&config.depot_path))?;
    let structure_path =
        canonicalize_in_allowed_roots(Path::new(&structure_path_in_depot), &[depot_root])?;
    let validation_report = validation::validate_structure(&structure_path)?;
    let should_use_override_action = allow_override && !validation_report.valid;

    if !validation_report.valid && !allow_override {
        return Ok(RouteStructureResult {
            validation_report,
            archived: false,
            validation_path: None,
            archive_path: None,
            override_used: false,
        });
    }

    let validation_path = move_structure_between_spaces(
        &config,
        structure_path
            .to_str()
            .ok_or_else(|| "Chemin structure invalide".to_string())?,
        &config.depot_path,
        &config.validation_path,
        AuditAction::Validate,
        None,
    )?;

    let mut archive_metadata = None;
    if allow_override {
        archive_metadata = Some(serde_json::json!({
            "override_reason": override_reason.unwrap_or_else(|| "Non précisé".to_string()),
            "errors_count": validation_report.errors_count,
            "warnings_count": validation_report.warnings_count
        }));
    }

    let action = if should_use_override_action {
        AuditAction::OverrideArchive
    } else {
        AuditAction::Archive
    };

    let archive_path = move_structure_between_spaces(
        &config,
        &validation_path,
        &config.validation_path,
        &config.archive_path,
        action,
        archive_metadata,
    )?;

    Ok(RouteStructureResult {
        validation_report,
        archived: true,
        validation_path: Some(validation_path),
        archive_path: Some(archive_path),
        override_used: should_use_override_action,
    })
}

/// ------------------------
///   LISTE & DÉTAILS
/// ------------------------

fn list_structures_for_space(
    config: &AppConfig,
    space: &str,
) -> Result<Vec<StructureListItem>, String> {
    let base_raw = match space {
        "Depot" => PathBuf::from(&config.depot_path),
        "Validation" => PathBuf::from(&config.validation_path),
        "Archive" => PathBuf::from(&config.archive_path),
        _ => return Err(format!("Espace inconnu: {}", space)),
    };

    if base_raw.as_os_str().is_empty() || !base_raw.exists() {
        return Ok(vec![]);
    }
    let base = canonicalize_existing_path(&base_raw)?;

    let mut result = Vec::new();

    for op_entry in
        fs::read_dir(&base).map_err(|e| format!("Erreur lecture espace {:?}: {}", base, e))?
    {
        let op_entry = op_entry.map_err(|e| format!("Erreur entrée op: {}", e))?;
        let op_path = op_entry.path();
        if !op_path.is_dir() {
            continue;
        }
        let op_name = op_entry.file_name().to_string_lossy().to_string();

        for st_entry in fs::read_dir(&op_path)
            .map_err(|e| format!("Erreur lecture structures {:?}: {}", op_path, e))?
        {
            let st_entry = st_entry.map_err(|e| format!("Erreur entrée structure: {}", e))?;
            let st_path = st_entry.path();
            if !st_path.is_dir() {
                continue;
            }
            let st_name = st_entry.file_name().to_string_lossy().to_string();

            let path_str = st_path
                .to_str()
                .ok_or_else(|| "Chemin structure invalide".to_string())?
                .to_string();

            result.push(StructureListItem {
                operation_folder: op_name.clone(),
                structure_folder: st_name,
                path: path_str,
            });
        }
    }

    Ok(result)
}

#[tauri::command]
fn list_structures(
    config_state: State<AppConfigState>,
    space: String,
) -> Result<Vec<StructureListItem>, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    list_structures_for_space(&config, &space)
}

#[tauri::command]
fn list_validation_items(
    config_state: State<AppConfigState>,
) -> Result<Vec<StructureListItem>, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    list_structures_for_space(&config, "Depot")
}

fn to_summary_item(
    structure_path: &Path,
    fallback: &StructureListItem,
) -> Result<StructureSummaryItem, String> {
    let metadata_path = structure_path.join("metadata.json");
    let metadata = if metadata_path.exists() {
        let text = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Erreur lecture metadata.json: {}", e))?;
        serde_json::from_str::<DepositMetadata>(&text).ok()
    } else {
        None
    };

    let path = structure_path
        .to_str()
        .ok_or_else(|| "Chemin structure invalide".to_string())?
        .to_string();

    Ok(StructureSummaryItem {
        operation_code: metadata
            .as_ref()
            .map(|m| m.operation.code.clone())
            .unwrap_or_else(|| fallback.operation_folder.clone()),
        operation_site: metadata
            .as_ref()
            .map(|m| m.operation.site.clone())
            .unwrap_or_default(),
        operation_type: metadata
            .as_ref()
            .map(|m| m.operation.op_type.clone())
            .unwrap_or_default(),
        responsable: metadata
            .as_ref()
            .map(|m| m.operation.responsable.clone())
            .unwrap_or_default(),
        structure_id: metadata
            .as_ref()
            .map(|m| m.structure.id.clone())
            .unwrap_or_else(|| fallback.structure_folder.clone()),
        structure_type: metadata
            .as_ref()
            .map(|m| m.structure.st_type.clone())
            .unwrap_or_default(),
        model_author: metadata
            .as_ref()
            .map(|m| m.structure.model_author.clone())
            .unwrap_or_default(),
        path,
        has_model: has_any_file(&structure_path.join("Modeles")),
        has_orthos: has_any_file(&structure_path.join("Orthomosaique")),
        has_production: has_any_file(&structure_path.join("DossierProduction").join("Photos"))
            || has_any_file(
                &structure_path
                    .join("DossierProduction")
                    .join("FichiersTravail"),
            ),
        last_modified_unix: get_modified_unix_seconds(structure_path),
    })
}

fn match_filters(item: &StructureSummaryItem, filters: &StructureSummaryFilters) -> bool {
    let contains_insensitive = |haystack: &str, needle: &str| {
        haystack
            .to_lowercase()
            .contains(&needle.trim().to_lowercase())
    };

    if let Some(has_model) = filters.has_model {
        if item.has_model != has_model {
            return false;
        }
    }

    if let Some(has_orthos) = filters.has_orthos {
        if item.has_orthos != has_orthos {
            return false;
        }
    }

    if let Some(has_production) = filters.has_production {
        if item.has_production != has_production {
            return false;
        }
    }

    if let Some(ref code) = filters.operation_code {
        if !contains_insensitive(&item.operation_code, code) {
            return false;
        }
    }

    if let Some(ref site) = filters.operation_site {
        if !contains_insensitive(&item.operation_site, site) {
            return false;
        }
    }

    if let Some(ref operation_type) = filters.operation_type {
        if !item
            .operation_type
            .eq_ignore_ascii_case(operation_type.trim())
        {
            return false;
        }
    }

    if let Some(ref structure_id) = filters.structure_id {
        if !contains_insensitive(&item.structure_id, structure_id) {
            return false;
        }
    }

    if let Some(ref structure_type) = filters.structure_type {
        if !item
            .structure_type
            .eq_ignore_ascii_case(structure_type.trim())
        {
            return false;
        }
    }

    if let Some(ref author) = filters.model_author {
        if !contains_insensitive(&item.model_author, author) {
            return false;
        }
    }

    if let Some(ref query) = filters.query {
        let q = query.trim().to_lowercase();
        if !q.is_empty() {
            let haystack = format!(
                "{} {} {} {} {} {} {}",
                item.operation_code,
                item.operation_site,
                item.operation_type,
                item.responsable,
                item.structure_id,
                item.structure_type,
                item.model_author
            )
            .to_lowercase();

            if !haystack.contains(&q) {
                return false;
            }
        }
    }

    true
}

#[tauri::command]
fn get_structure_summary(
    config_state: State<AppConfigState>,
    space: String,
    filters: Option<StructureSummaryFilters>,
    pagination: Option<StructureSummaryPagination>,
    sort: Option<StructureSummarySort>,
) -> Result<StructureSummaryResponse, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    let list = list_structures_for_space(&config, &space)?;
    let filters = filters.unwrap_or_default();
    let pagination = pagination.unwrap_or_default();
    let sort = sort.unwrap_or_default();

    let mut items = Vec::new();
    for entry in list {
        let path = PathBuf::from(&entry.path);
        if !path.exists() || !path.is_dir() {
            continue;
        }
        let summary = to_summary_item(&path, &entry)?;
        if match_filters(&summary, &filters) {
            items.push(summary);
        }
    }

    let desc = sort.direction.eq_ignore_ascii_case("desc");
    match sort.field.as_str() {
        "structure_id" => items.sort_by(|a, b| a.structure_id.cmp(&b.structure_id)),
        "modified_at" => items.sort_by(|a, b| a.last_modified_unix.cmp(&b.last_modified_unix)),
        _ => items.sort_by(|a, b| a.operation_code.cmp(&b.operation_code)),
    }
    if desc {
        items.reverse();
    }

    let total = items.len();
    let per_page = pagination.per_page.clamp(1, 500);
    let page = pagination.page.max(1);
    let start = (page - 1).saturating_mul(per_page);
    let paginated_items = items
        .into_iter()
        .skip(start)
        .take(per_page)
        .collect::<Vec<_>>();

    Ok(StructureSummaryResponse {
        total,
        page,
        per_page,
        items: paginated_items,
    })
}

#[tauri::command]
fn get_structure_details(
    config_state: State<AppConfigState>,
    structure_path: String,
) -> Result<StructureDetails, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    let st_dir = canonicalize_structure_in_known_roots(&config, &structure_path)?;

    let metadata_path = st_dir.join("metadata.json");
    let (metadata, metadata_raw) = if metadata_path.exists() {
        let text = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Erreur lecture metadata.json: {}", e))?;
        let raw = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|e| format!("Erreur parsing metadata.json (raw): {}", e))?;
        let typed = serde_json::from_value::<DepositMetadata>(raw.clone()).ok();
        (typed, Some(raw))
    } else {
        (None, None)
    };

    let models = list_files_in_dir(&st_dir.join("Modeles"))?;
    let orthos = list_files_in_dir(&st_dir.join("Orthomosaique"))?;
    let prod_dir = st_dir.join("DossierProduction");
    let photos = list_files_in_dir(&prod_dir.join("Photos"))?;
    let work_files = list_files_in_dir(&prod_dir.join("FichiersTravail"))?;

    Ok(StructureDetails {
        metadata,
        metadata_raw,
        models,
        orthos,
        photos,
        work_files,
    })
}

#[tauri::command]
fn get_structure_file_info(
    config_state: State<AppConfigState>,
    path: String,
) -> Result<StructureFileInfo, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    let safe_path = canonicalize_structure_in_known_roots(&config, &path)?;
    let metadata = fs::metadata(&safe_path)
        .map_err(|e| format!("Erreur lecture métadonnées fichier {:?}: {}", safe_path, e))?;

    if !metadata.is_file() {
        return Err("Le chemin fourni n'est pas un fichier.".to_string());
    }

    Ok(StructureFileInfo {
        size_bytes: metadata.len(),
    })
}

/// Ouvre l'explorateur de fichiers et sélectionne le chemin spécifié
#[tauri::command]
fn reveal_in_explorer(config_state: State<AppConfigState>, path: String) -> Result<(), String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;
    let safe_path = canonicalize_structure_in_known_roots(&config, &path)?;

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&safe_path)
            .spawn()
            .map_err(|e| format!("Erreur ouverture Finder: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&safe_path)
            .spawn()
            .map_err(|e| format!("Erreur ouverture Explorer: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Sur Linux, on ouvre juste le dossier parent avec xdg-open
        if let Some(parent) = safe_path.parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Erreur ouverture gestionnaire de fichiers: {}", e))?;
        }
    }

    Ok(())
}

#[tauri::command]
fn generate_inventory(config_state: State<AppConfigState>) -> Result<Vec<InventoryItem>, String> {
    let config = config_state
        .lock()
        .map_err(|e| format!("Erreur accès config: {}", e))?;

    let archive_path = PathBuf::from(&config.archive_path);
    if !archive_path.exists() {
        return Ok(vec![]);
    }

    let mut inventory = Vec::new();

    // Parcourir toutes les opérations dans Archive
    for op_entry in
        fs::read_dir(&archive_path).map_err(|e| format!("Erreur lecture archive: {}", e))?
    {
        let op_entry = op_entry.map_err(|e| format!("Erreur entrée op: {}", e))?;
        let op_path = op_entry.path();
        if !op_path.is_dir() {
            continue;
        }

        // Parcourir toutes les structures dans l'opération
        for st_entry in
            fs::read_dir(&op_path).map_err(|e| format!("Erreur lecture structures: {}", e))?
        {
            let st_entry = st_entry.map_err(|e| format!("Erreur entrée structure: {}", e))?;
            let st_path = st_entry.path();
            if !st_path.is_dir() {
                continue;
            }

            // Lire metadata.json
            let metadata_path = st_path.join("metadata.json");
            let metadata = if metadata_path.exists() {
                let text = fs::read_to_string(&metadata_path)
                    .map_err(|e| format!("Erreur lecture metadata.json: {}", e))?;
                serde_json::from_str::<DepositMetadata>(&text).ok()
            } else {
                None
            };

            // Obtenir la taille du modèle 3D
            let models_dir = st_path.join("Modeles");
            let model_size_mb = if let Some(model_file) = find_first_model_file(&models_dir) {
                format!("{:.2}", get_file_size_mb(&model_file))
            } else {
                "0".to_string()
            };
            let photos_total_size_mb = {
                let orthos_dir = st_path.join("Orthomosaique");
                let prod_photos_dir = st_path.join("DossierProduction").join("Photos");
                let total_bytes =
                    get_directory_size_bytes(&orthos_dir) + get_directory_size_bytes(&prod_photos_dir);
                format!("{:.2}", total_bytes as f64 / (1024.0 * 1024.0))
            };

            // Obtenir la date de dépôt (date de modification du dossier)
            let deposit_date = if let Some(timestamp) = get_modified_unix_seconds(&st_path) {
                use std::time::{Duration, SystemTime};
                let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);
                let datetime: chrono::DateTime<chrono::Utc> = system_time.into();
                datetime.format("%Y-%m-%d").to_string()
            } else {
                "".to_string()
            };

            let item = InventoryItem {
                structure_id: metadata
                    .as_ref()
                    .map(|m| m.structure.id.clone())
                    .unwrap_or_default(),
                operation_code: metadata
                    .as_ref()
                    .map(|m| m.operation.code.clone())
                    .unwrap_or_default(),
                operation_site: metadata
                    .as_ref()
                    .map(|m| m.operation.site.clone())
                    .unwrap_or_default(),
                operation_type: metadata
                    .as_ref()
                    .map(|m| m.operation.op_type.clone())
                    .unwrap_or_default(),
                operation_responsable: metadata
                    .as_ref()
                    .map(|m| m.operation.responsable.clone())
                    .unwrap_or_default(),
                structure_type: metadata
                    .as_ref()
                    .map(|m| m.structure.st_type.clone())
                    .unwrap_or_default(),
                photos_count: metadata
                    .as_ref()
                    .map(|m| m.structure.photos_count.clone())
                    .unwrap_or_default(),
                photos_total_size_mb,
                model_size_mb,
                faces_count: metadata
                    .as_ref()
                    .map(|m| m.structure.faces_count.clone())
                    .unwrap_or_default(),
                model_author: metadata
                    .as_ref()
                    .map(|m| m.structure.model_author.clone())
                    .unwrap_or_default(),
                depositor: metadata
                    .as_ref()
                    .map(|m| m.structure.depositor.clone())
                    .unwrap_or_default(),
                software: metadata
                    .as_ref()
                    .map(|m| m.structure.software.clone())
                    .unwrap_or_default(),
                deposit_date,
            };

            inventory.push(item);
        }
    }

    Ok(inventory)
}

/// --------------------
///   MAIN TAURI
/// --------------------
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // Charger la configuration au démarrage
            let config = AppConfig::load_from_store(app.handle()).unwrap_or_default();
            app.manage(std::sync::Mutex::new(config));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            deposit_structure,
            validate_structure_from_depot,
            archive_structure_from_validation,
            archive_structure_from_depot,
            get_revision_tags,
            set_revision_tag,
            validate_then_route_structure,
            list_structures,
            list_validation_items,
            get_structure_summary,
            get_structure_details,
            get_structure_file_info,
            get_metadata_presets,
            update_metadata_presets,
            count_model_polygons,
            reveal_in_explorer,
            generate_inventory,
            config::get_app_config,
            config::update_app_config,
            config::validate_config_paths,
            config::preview_validate_config_paths,
            config::verify_admin_password,
            logging::get_audit_logs,
            logging::get_recent_validations,
            logging::reset_validation_history,
            validation::validate_structure_check,
            validation::update_structure_metadata,
            statistics::get_archive_statistics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
