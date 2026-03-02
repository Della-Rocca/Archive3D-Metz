# Ops Configuration Guide

## Config Keys
- `depot_path`: incoming structures.
- `validation_path`: temporary validation workspace.
- `archive_path`: final archived structures.
- `presets_path`: `metadata-presets.json` for dynamic ontologies.
- `logs_path`: audit logs folder (`audit.log`).

## Recommended Layout
```text
<base>/
  Depot/
  Validation/
  Archive/
    metadata-presets.json
  logs/
    audit.log
```

## Presets File Behavior
- If `metadata-presets.json` is missing, the app auto-creates a valid default file.
- If it is invalid JSON, the app creates a timestamped backup (`*.bak.<ts>`) and regenerates a default file.
- Presets are automatically enriched on structure deposit and metadata edits.

## Security Rules (Backend)
- Path segments from metadata are sanitized (`..`, separators, drive prefixes blocked).
- Moves between spaces are done from canonicalized roots with strict relative-path checks.
- Structure details/model reads/metadata edits are restricted to configured roots only.

## Common Diagnostics
- `Le chemin ... est hors des racines autorisées`:
  - The requested path is outside Depot/Validation/Archive roots.
- `Aucune racine de stockage valide n'est configurée`:
  - One or more config paths are missing or inaccessible.
- `metadata-presets.json invalide`:
  - Check `.bak` file, then review regenerated defaults.
