# Salaat Widget — Notes de développement

Widget Tauri 2 (Rust + HTML/JS) pour les horaires de prière, docké au-dessus de la barre des tâches Windows.
Dev principal sur Linux ; le build Windows se fait sur la partition partagée D:.

## État actuel

- Fenêtre compacte 320×60, transparente, sans bordure, toujours au premier plan, `skipTaskbar` : affiche la prochaine prière + compte à rebours (tick 1 s).
- Clic sur le widget → vue détaillée (5 prières + date hégirienne + boutons Réglages/Fermer) ; **drag** du widget → déplacement libre, position mémorisée.
- Tray : icône + tooltip « Asr dans 01:23:45 » (mis à jour chaque seconde), clic gauche affiche/masque, menu contextuel Quitter.
- Calcul 100 % offline dans `crates/salaat-core` (méthodes AlAdhan, hégire).
- Config : `%APPDATA%\SalaatWidget\config.json` (Windows) / `~/.config/SalaatWidget/config.json` (Linux).
- **UI de réglages** (fenêtre dédiée) : ville (lat/lon), méthode, école, règle haute latitude, langue (fr/en/ar), format 12/24 h, démarrage auto Windows, démarrage masqué.
- **Windows uniquement** : fenêtre `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW` (jamais de vol de focus, pas d'Alt-Tab), positionnement réel contre la barre des tâches (`SHAppBarMessage`), auto-masquage quand une app plein écran passe au premier plan.

## Modifs déjà appliquées (à conserver)

- `src-tauri/Cargo.toml` : `crate-type = ["rlib"]`. Le `cdylib` faisait échouer le linker MinGW (`export ordinal too large`), inutile pour un desktop Windows. À rendre conditionnel si on repart sur du mobile.
- `src-tauri/tauri.conf.json` : `"withGlobalTauri": true`. Le front est servi sans bundler : l'import ESM `@tauri-apps/api/core` ne se résolvait pas dans le WebView. `main.js` utilise donc `window.__TAURI__.core.invoke`. Ajout de `"focus": false` sur la fenêtre principale.
- `src-tauri/src/win32.rs` (nouveau, `#![cfg(windows)]`) : `make_no_activate`, `position_near_taskbar` (ABM_GETTASKBARPOS), `spawn_fullscreen_watcher` (GetForegroundWindow + GetMonitorInfoW). HWND récupéré via `raw-window-handle` (évite de dépendre du type du crate `windows`).
- `src-tauri/src/tray.rs` : positionnement « position sauvegardée → barre des tâches (Win) → bas-droite (repli) ». Suppression du `set_focus()`. **Correction DPI** : plus de division par le scale factor — tout est en pixels physiques (`PhysicalPosition`), la position drag est stockée en pixels logiques.
- `src-tauri/src/lib.rs` : plugin `tauri-plugin-autostart` (registre Windows / XDG Linux), sync registre ↔ config au setup et au save, application de `start_hidden`, montage du watcher plein écran.
- `src-tauri/src/commands.rs` : `open_settings` réel, `get_config`, `set_config` (validation + sync autostart + préserve la position drag), `save_window_position`. `StatusPayload` inclut `language` + `hour12`. Date hégirienne localisée (mois fr/en/ar).
- `src/main.js` : i18n fr/en/ar (noms de prières + chaînes UI + tooltip), format 12/24 h depuis la config, drag avec seuil click/drag (pointer capture), garde `typeof import.meta.env !== "undefined"` (crash sans bundler).
- `src/settings.html` + `src/settings.js` + `src/settings.css` : fenêtre de réglages (sans bundler, `window.__TAURI__`).
- `crates/salaat-core/src/hijri.rs` : ajout de `MONTHS_AR` (EN/FR existaient déjà).

## Design — thème Windows 11 (Fluent) sombre

- Palette : surfaces neutres sombres (`#1c1c1c`–`#2e2e2e` translucides + `backdrop-filter: blur(24px)` avec repli solide `#202020`), coins 8 px, texte en niveaux d'opacité (100/66/45 %), accent `#60cdff` (bleu Win11 dark) au lieu de l'or/violet d'origine.
- Police : `Segoe UI Variable Text` → `Segoe UI` → `system-ui` ; chiffres tabulaires ; plus de `text-transform: uppercase`.
- Boutons ghost + focus ring accent dans la fenêtre Réglages (`settings.css`).
- **Piège Linux (WebKitGTK)** : la WebView impose un minimum de 200 px de hauteur → une fenêtre 320×60 devient 320×200 sur Linux (pas sur Windows/WebView2). Fix CSS : la barre compacte est fixée à `height: 60px; align-self: center` → elle reste une vraie barre centrée, le reste est transparent. Rien ne change sur Windows (la fenêtre y fait bien 320×60).

## Build

### Linux
`cargo build` / `cargo run` (racine du workspace).

### Windows (machine actuelle)
Pas de linker MSVC (pas de VS Build Tools) → toolchain GNU + MinGW (WinLibs) :

```
rustup default stable-x86_64-pc-windows-gnu
$env:PATH = "<chemin WinLibs>\mingw64\bin;" + $env:PATH
cargo build
```

Exécutable : `target\debug\salaat-widget.exe`.
DLLs runtime : `libgcc_s_seh-1.dll` et `libwinpthread-1.dll` (dans `<toolchain GNU>\lib\rustlib\x86_64-pc-windows-gnu\bin\`) à mettre dans le PATH ou à côté de l'exe.
Warnings bénins : `.rsrc merge failure` (manifeste MinGW).

## Feuille de route

### Fait ✅
- P0 — jamais voler le focus : `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW` (win32.rs), `"focus": false` (tauri.conf.json), `set_focus()` retiré (tray.rs).
- P0 — démarrage auto : `tauri-plugin-autostart`, toggle dans les réglages.
- P1 — position réelle de la barre : `ABM_GETTASKBARPOS` (4 bords), repli bas-droite ; DPI corrigé (pixels physiques/logiques, plus de division).
- P1 — auto-masquage plein écran : watcher 750 ms, comparaison rect fenêtre = rect moniteur.
- P2 — UI de réglages complète (ville, méthode, école, hautes latitudes, langue, 12/24 h, autostart, démarrage masqué) + `config::save()` utilisé (warning dead_code supprimé).
- P3 — drag + mémorisation de position (pixels logiques dans la config).
- P3 — retour au docking : menu tray « Docker à la barre » (`reset_dock`) → efface la position mémorisée et recolle le widget à la barre.
- P3 — i18n fr/en/ar (prières, chaînes UI, tooltip, mois hégiriens) + garde `import.meta.env`.

### À tester sous Windows (boot)
- Vol de focus : cliquer le widget ne doit pas sortir le clavier de l'app active.
- Positionnement contre la barre (bas/haut/gauche/droite) et multi-écrans.
- Auto-masquage en plein écran (jeu/vidéo) puis réapparition.
- Autostart (registre) + démarrage masqué + drag/mémorisation.
- Fenêtre de réglages : enregistrer → le widget se met à jour (langue, 12/24 h, horaires) sans relancer.

### Reste (idées)
- P3 — compte à rebours dans l'icône du tray : redessiner l'icône avec le temps restant (ex. « 135 ») chaque minute via `tray.set_icon()`. Lisibilité limitée (16×16) → décision : on garde le tray tel quel pour l'instant.
- Nettoyer le warning `.rsrc merge failure` si possible.
- Idéalement : VS Build Tools + toolchain MSVC sur Windows (supprime les soucis MinGW, docs Tauri).

## Décisions UX (assumées)

- **Visible par défaut** : le but est de *voir* le countdown sans rien faire ; démarrer caché dans le tray tuerait l'intérêt. Le toggle « démarrer masqué » (prochain démarrage) est dispo pour ceux qui préfèrent un bureau propre.
- **Docké contre la barre, côté tray** : zone où l'œil va déjà (l'horloge), hors de la zone de travail centrale ; suivi réel de la position de la barre sous Windows.
- **Tray = contrôle secondaire** : clic gauche afficher/masquer, pour récupérer le widget s'il est perdu — pas pour l'utiliser.
- **skipTaskbar + always-on-top** : ce n'est pas une « tâche », c'est un HUD.
- Widget « dans » la barre elle-même : impossible (taskbar bands supprimés sous Windows 11) → fenêtre dockée au-dessus, c'est le bon paradigme.
- Launcher type Flow Launcher : paradigme invoqué, pas « glanceable » → hors sujet pour un compte à rebours permanent (utile seulement pour chercher les horaires à la demande).
- Le vrai trio d'amélioration UX : pas de vol de focus + auto-masquage plein écran + position réelle de la barre — **implémenté**, à valider au prochain boot Windows.

## Validation Windows (30/08/2026) — build OK, app lancée

- Build `cargo +stable-x86_64-pc-windows-gnu build` : OK (seul warning : `.rsrc merge failure`, bénin).
- `win32.rs` n'était jamais compilé sur Linux (cfg windows) → 2 erreurs corrigées :
  - `WS_EX_NOACTIVATE` / `WS_EX_TOOLWINDOW` sont `u32` mais `GetWindowLongW` retourne `i32` → `(style as u32 | …) as i32`.
  - Import `ABE_BOTTOM` inutilisé retiré.
- Capabilities : la fenêtre `settings` n'était pas déclarée → `close()` (bouton Annuler / fermeture auto après enregistrement) était refusé silencieusement. Ajout de `"settings"` à la capability + `core:window:allow-close`.
- Drag DPI : `outerPosition()` renvoie des pixels **physiques**, mais la config stocke du logique → conversion `pos.toLogical(scaleFactor)` avant `save_window_position` (sinon position fausse en écran > 100 %).
- Tests : `salaat-core` (10) OK sous Windows. Les tests de la lib (`salaat_widget_lib`) **crash au chargement** (`0xc0000139` entry point not found) avec la toolchain GNU : les import libs du crate `windows` 0.52 référencent des ordinaux non résolus dans le binaire de test (pas dans l'exe principal). **Workflow : tester sur Linux** (ça passe), compiler/lancer sur Windows.
- App lancée : process stable (~40 Mo), fenêtre « Salaat Widget » responsive.

### À tester à la main (Windows)
- Clic sur le widget : ne doit pas voler le focus clavier de l'app active (WS_EX_NOACTIVATE).
- Widget collé à la barre (côté tray) ; drag pour déplacer → position mémorisée (relance pour vérifier).
- Fenêtre Réglages (bouton dans la vue détaillée) : changer ville/langue/12-24h → Enregistrer → le widget se met à jour sans relancer.
- Plein écran (vidéo/jeu) : le widget se cache, réapparaît à la sortie.
## Session 30/08 soir — fenêtre Réglages + animations (validé à la main via simulation de clics)

- **Fenêtre Réglages** : 4 bugs corrigés de front.
  1. Le `setPointerCapture` du drag (armDrag) volait les clics des boutons de la vue détaillée → « Réglages » ne s'ouvrait jamais. Fix : `if (e.target.closest("button")) return;` dans `pointerdown`.
  2. Création **lazy** depuis une commande → webview WebView2 enfant à 0×0 (fenêtre blanche). Fix : pré-créer la fenêtre `settings` dans `setup()` (visible(false)) ; `open_settings` ne fait que center/show/focus. La pré-création sur le thread principal avec sa taille finale évite la course d'init.
  3. `close()` **détruit** la fenêtre → impossible de la rouvrir ensuite. Fix : `hide()` dans settings.js (Annuler + après Enregistrer) + interception du X de la barre de titre (`on_window_event` → `CloseRequested { api }` → `prevent_close()` + `hide()`).
  4. Fenêtre trop petite (440×640) : les boutons Annuler/Enregistrer étaient hors écran. Fix : 460×740, `min_inner_size` 400×600, `resizable(true)`. + `margin-top: auto` sur `.detail-footer` (le footer flottait au-dessus du bas).
- **`config-changed`** : `set_config` émet un événement → le widget se met à jour immédiatement (langue, 12/24 h, horaires) sans relancer.
- **Animations fade in/out** : le toggle tray émet `animate-out` → le front fond en 180 ms (ease-out) puis invoque `hide_window` ; à l'ouverture, `animate-in` fond en 200 ms. Compteur de génération (`AppState.hide_gen`) pour que le fallback Rust (400 ms, si JS mort) ne masque pas une fenêtre ré-affichée (double-clic tray rapide).
- **Drag sur les deux vues** : `armDrag` sur `#compact` et `#detail` ; clic sur le fond de la vue détaillée = repli (toggleView).
- Validation automatisée (simulation de clics Win32 + PrintWindow) : ouverture, rendu, Annuler→masquer, réouverture, Enregistrer→sauvegarde+masquer — OK.
- `IsWindowVisible` vs `EnumWindows` : une fenêtre masquée reste listée par EnumWindows (piège pour les tests).