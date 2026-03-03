# Archive Metz

Application desktop de consultation et gestion de l'archive 3D patrimoniale de Metz.

Permet de parcourir, visualiser et inventorier les structures archéologiques numérisées en 3D, avec consultation des photos, métadonnées et statistiques de l'archive.

---

## Fonctionnalités

- **Consultation** — recherche et navigation dans les structures archivées
- **Visualisation 3D** — chargement et affichage des modèles `.glb` directement dans l'app
- **Inventaire** — tableau complet des structures avec tri, filtrage et export CSV
- **Statistiques** — aperçu global de l'archive (volumes, répartitions par opération, type, logiciel...)

---

## Installation

Télécharge le fichier correspondant à ton système depuis les [releases GitHub](../../releases) :

| Système | Fichier |
|---|---|
| macOS | `.dmg` |
| Windows | `.exe` |

Lance l'installateur et suis les instructions.

---

## Développement

### Prérequis

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/)
- [Tauri CLI](https://tauri.app/) v2

### Lancer en mode dev

```bash
npm install
npm run dev
```
---

## Stack technique

- **Frontend** — SvelteKit 2 + Svelte 5 + TypeScript
- **Backend** — Tauri 2 (Rust)
- **3D** — Three.js
- **Builds CI** — GitHub Actions (macOS + Windows)
