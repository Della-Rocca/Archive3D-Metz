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

## Publier une release

Les releases sont construites et publiées automatiquement par GitHub Actions (macOS universel, Windows NSIS, Linux AppImage) avec les artefacts de mise à jour automatique (`latest.json` + signatures).

1. **Bumper la version** dans `src-tauri/tauri.conf.json` **et** `package.json` (ex. `0.2.0`). C'est la version de `tauri.conf.json` que l'updater compare — un tag `v0.2.0` avec une config restée en `0.1.0` ne déclenchera aucune mise à jour chez les utilisateurs.
2. **Commiter, tagger et pousser** :
   ```bash
   git commit -am "release: v0.2.0"
   git tag v0.2.0
   git push origin main --tags
   ```
3. Le workflow crée la release GitHub avec les installateurs, les fichiers `.sig` et le `latest.json`. Les applications installées détectent la nouvelle version au prochain démarrage.

> **Notes de version dans le pop-up** : le texte affiché aux utilisateurs vient du champ `releaseBody` du workflow au moment de la publication. Éditer la release sur GitHub après coup ne met pas à jour `latest.json`.

### Secrets requis (déjà configurés)

- `TAURI_SIGNING_PRIVATE_KEY` — clé privée de signature de l'updater
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — son mot de passe

> ⚠️ Si la clé privée ou son mot de passe sont perdus, les applications déjà installées ne pourront plus vérifier les mises à jour (leur clé publique embarquée ne correspondra plus). Conserver une sauvegarde de `~/.tauri/archive-metz.key`.

---

## Stack technique

- **Frontend** — SvelteKit 2 + Svelte 5 + TypeScript
- **Backend** — Tauri 2 (Rust)
- **3D** — Three.js
- **Builds CI** — GitHub Actions (macOS + Windows + Linux)
- **Mises à jour** — tauri-plugin-updater (releases GitHub)

**Produit et développé par François de Senneville**
