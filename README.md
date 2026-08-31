<p align="center">
  <img src="assets/banner.png" alt="Salaat Widget" width="100%">
</p>

<p align="center">
  <strong>Salaat Widget</strong> &nbsp;·&nbsp; vos horaires de prière, affichés en permanence au-dessus de la barre des tâches Windows
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0-60cdff" alt="version">
  <img src="https://img.shields.io/badge/platform-Windows%2011-1c1c1c" alt="platform">
  <img src="https://img.shields.io/badge/offline-100%25-1c1c1c" alt="offline">
  <img src="https://img.shields.io/badge/lang-fr%20%7C%20en%20%7C%20ar-1c1c1c" alt="langues">
</p>

---

## ✨ Fonctionnalités

- **Widget HUD** compact et discret, toujours au-dessus de vos applications — sans jamais voler le focus.
- **Compte à rebours** vers la prochaine prière, actualisé chaque seconde.
- **Vue détaillée** : les horaires du jour + la **date hégirienne**.
- **100 % hors-ligne** : le calcul des horaires se fait localement, aucune donnée n'est envoyée.
- **Auto-masquage** en plein écran (jeu, vidéo) et réapparition automatique.
- **Localisation automatique** (« Utiliser ma position ») avec le **fuseau horaire de la ville** (gère l'heure d'été).
- **Réglages auto-enregistrés** et interface multilingue (français / English / العربية).
- **Démarrage avec Windows** optionnel.

## 🖼️ Aperçu

<div align="center">
  <img src="assets/screenshot-compact.png" alt="Widget compact" width="280">
  <br><br>
  <img src="assets/screenshot-detail.png" alt="Vue détaillée" width="280">
</div>

## 🚀 Installation

Téléchargez le dernier installeur **`Salaat Widget_x64-setup.exe`** depuis la section [Releases](https://github.com/ismail-bahloul/Miqat/releases) et lancez-le. **Aucun droit administrateur requis** (installation par utilisateur).

> ⚠️ L'app n'est pas encore signée — au premier lancement, Windows peut afficher **« Plus d'infos → Exécuter quand même »**. C'est un certificat de signature qui lèvera cet avertissement à terme.

## 🧭 Utilisation

- Le widget affiche la **prochaine prière** et le **temps restant**.
- **Clic** sur le widget → **vue détaillée** (prières + lever + date hégirienne).
- **Glisser** le widget → le déplacer (position mémorisée ; le menu tray « Docker à la barre » le recolle).
- **Icône tray** : clic gauche affiche/masque, menu pour redocker ou quitter.

## ⚙️ Réglages

- **Ville / coordonnées** : saisissez-les manuellement ou cliquez **« Utiliser ma position »** (géolocalisation IP).
- **Méthode de calcul**, **école (Asr)**, **règle de hautes latitudes**.
- **Langue** (fr / en / ar), **format 12/24 h**.
- **Démarrer avec Windows**, **démarrer masqué**.
- Tous les changements sont **appliqués automatiquement** — pas de bouton « Enregistrer ».

## 🛠️ Compiler depuis les sources

Prérequis : [Rust](https://rustup.rs), une toolchain Windows et le [runtime WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (présent par défaut sur Windows 11).

```bash
cargo build --release            # binaire seul
cargo tauri build                # + installeur NSIS
```

## 📄 Licence

En cours de définition. — © Ismail Bahloul

---

<div align="center">
  <img src="assets/logo.png" alt="Salaat Widget" width="64">
  <br>
  <sub>fait avec ❤️ pour la communauté</sub>
</div>
