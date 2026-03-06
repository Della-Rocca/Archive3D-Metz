# Archive Metz

Application desktop de consultation et gestion de l'archive 3D patrimoniale du service archéologique de Metz.

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


- macOS | [Archive-Metz.dmg](https://github.com/Della-Rocca/Archive3D-Metz/releases/download/dowload/Archive.Metz.dmg)
- Windows | [Archive-Metz.exe](https://github.com/Della-Rocca/Archive3D-Metz/releases/download/dowload/Archive.Metz.exe)

L'application n'étant pas signée, un message d'avertissement vous indiquera que le fichier ne contient aucune signature valide et que l'identité de l'éditeur est inconnue. Cela est normal, l'application n'est pas signée, ce qui déclenche un processus de sécurité. Vous pouvez passer outre.

**MDP admin de l'app : 1 (par défaut, modifiable dans les paramètres)**
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

**Produit et développé par François de Senneville**
