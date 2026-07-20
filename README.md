# Archive Metz

Application desktop de consultation et gestion de l'archive 3D patrimoniale du service archéologique de Metz.

Permet de parcourir, visualiser et inventorier les structures archéologiques numérisées en 3D, avec consultation des photos, métadonnées et statistiques de l'archive.

---

## Fonctionnalités

- **Consultation** : recherche et navigation dans les structures archivées
- **Visualisation 3D** : chargement et affichage des modèles `.glb` directement dans l'app
- **Inventaire** : tableau complet des structures avec tri, filtrage et export CSV
- **Statistiques** : aperçu global de l'archive (volumes, répartitions par opération, type, logiciel...)

---

## Installation

Télécharge le fichier correspondant à ton système depuis la [dernière release GitHub](../../releases/latest) :

- macOS : fichier `.dmg`
- Windows : fichier `-setup.exe` (installateur NSIS)
- Linux : fichier `.AppImage`

Une fois installée, l'application vérifie automatiquement à chaque démarrage s'il existe une nouvelle version et propose de l'installer en un clic (aucun re-téléchargement manuel nécessaire).

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
npm run tauri dev
```
---

## Stack technique

- **Frontend** — SvelteKit 2 + Svelte 5 + TypeScript
- **Backend** — Tauri 2 (Rust)
- **3D** — Three.js
- **Builds CI** — GitHub Actions (macOS + Windows + Linux)
- **Mises à jour** — tauri-plugin-updater (releases GitHub)

**Produit et développé par François de Senneville**
