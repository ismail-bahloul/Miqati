# Miqati — Notes de développement

Widget Tauri 2 (Rust + HTML/JS) pour les horaires de prière, docké au-dessus de la barre des tâches Windows.
Dev principal sur Linux ; le build Windows se fait sur la partition partagée D:.

## État actuel

- Fenêtre compacte **240×48** (largeur unique compact/détail, chiffres 13 px), transparente, sans bordure, toujours au premier plan, `skipTaskbar` : affiche la prochaine prière + compte à rebours (tick 1 s).
- Clic sur le widget → vue détaillée (5 prières + date hégirienne + boutons Réglages/Fermer) ; **drag** du widget → déplacement libre, position mémorisée.
- Tray : icône + tooltip « Asr dans 01:23:45 » (mis à jour quand la minute change), clic gauche affiche/masque, menu contextuel localisé (fr/en/ar) : Afficher/Masquer, Docker à la barre, Quitter.
- **Notifications système** (plugin Tauri) : rappel **avant** la prière (délai réglable 5–30 min) et **à l'heure** de la prière ; scheduler Rust (`notifier.rs`) qui fonctionne même widget masqué, gère le réveil après veille, le changement de ville et l'anti-doublon (Shuruq exclu).
- Calcul 100 % local dans `crates/salaat-core` (méthodes AlAdhan, hégire).
- **Ajustement de la date hégirienne** : présent dans la config/le backend, **retiré de l'interface** (trop opaque pour l'utilisateur).
- **Offsets ±minutes par prière** : présents dans la config et le backend (`PrayerOffsets`), **volontairement absents de l'interface** — le réglage des horaires doit rester simple (loupe → ville → horaires justes).
- **Mode hors ligne** (défaut : désactivé) : aucune requête réseau automatique ; les horaires restent calculés localement. La détection de ville par IP (`ip-api.com`, HTTP) est le seul usage réseau.
- Config : `%APPDATA%\Miqati\config.json` (Windows) / `~/.config/Miqati/config.json` (Linux) ; **lecture tolérante** (un champ invalide ne fait plus perdre toute la config) + copie `config.bak.json`.
- **UI de réglages** (fenêtre dédiée, volontairement minimale) : en clair — Ville + loupe de détection, Langue, Format de l'heure, **Rappels** (un seul sélecteur), et 4 cases (démarrage auto, démarrage masqué, toujours au premier plan, mode hors ligne). Sous **« Réglages avancés »** (replié) : latitude/longitude, méthode de calcul, école (Asr), règle hautes latitudes, fuseau horaire.
- **Windows uniquement** : fenêtre `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW` (jamais de vol de focus, pas d'Alt-Tab), positionnement réel contre la barre des tâches (`SHAppBarMessage`), placement **au-dessus** de la barre (jamais à cheval), auto-masquage quand une app plein écran passe au premier plan (~300 ms).
  - **Limite connue** : la barre des tâches, le menu Démarrer et les volets (wifi/batterie, calendrier) sont composités par le DWM au-dessus de toutes les fenêtres utilisateur — ils peuvent recouvrir le widget. Comportement de Windows, assumé ; le widget revient devant à la fermeture du volet.

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

## Workflow git (source de vérité unique)

- Remote privé : `git@github.com:ismail-bahloul/Miqati.git` (branch `main`).
- Dev principal sur Linux ; Windows = simple clone/pull/push, **plus de rsync**.
- Linux : `git pull` / `git push` (SSH déjà configuré).
- Windows (dans `D:\Miqati`, ex-copie SHARED) :
  ```
  git remote add origin git@github.com:ismail-bahloul/Miqati.git
  git fetch origin && git reset --hard origin/main
  ```
  (la copie locale est identique à origin ; `reset --hard` aligne l'historique.
  Si pas de clé SSH sous Windows : utiliser l'URL HTTPS
  `https://github.com/ismail-bahloul/Miqati.git` + Git Credential Manager.)
- Ne plus éditer les deux copies en parallèle — toujours pull avant de travailler,
  push après.

## Calcul des horaires — aligné sur AlAdhan (vérifié)

- **Moteur** : port fidèle de PrayTimes, cross-validé contre l'API AlAdhan (calibrations Paris/Sydney/Maroc/Singapour/Russie dans les tests, tolérance ≤ 3 min). Le moteur est correct.
- **Table des méthodes réalignée sur AlAdhan** (référence utilisée par la quasi-totalité des apps) :
  - Corrigé : Qatar (Isha 90 min après Maghrib), Golfe (Isha 90 min), Singapour (Fajr 20°), Russie (16/15).
  - Ajouté : **Maroc (19°/17° + Dhuhr+5/Maghrib+5)**, Tunisie (18/18), Algérie (18/17), Dubaï (18.2/18.2), JAKIM (20/18), KEMENAG (20/18), Portugal (Isha 77 min, Maghrib +3 min), Jordanie (Maghrib +5 min).
  - Support ajouté : Isha en **minutes** (Oumm Al-Qura/Qatar/Golfe/Portugal), offset **Maghrib** (angle : Téhéran 4.5°, Jafari 4° ; minutes : Portugal 3, Jordanie 5, Maroc 5), offset **Dhuhr** (Maroc +5).
- **Le repli hors-ligne du widget KDE est incorrect** (`OfflinePrayerCalc.js` :
  Karachi/ISNA inversés, Shia/Téhéran/UOIF/Russie faux, pas de méthode Maroc).
  **Nuance importante** : ce widget n'est pas hors-ligne en temps normal — il
  interroge l'API AlAdhan (`api.aladhan.com`), et son `methodMap` traduit
  correctement son index 14 vers AlAdhan 21 (Maroc). En ligne il est donc juste ;
  c'est uniquement son **repli sans réseau** qui se dégrade (Maroc → MWL, ~5 min
  d'écart sur Fajr/Dhuhr/Maghrib). Miqati reproduit le calcul d'AlAdhan
  **sans réseau** : identique en ligne, meilleur hors-ligne.
- **Piège Maroc/DST** : le Maroc est UTC+1 toute l'année sauf pendant le Ramadan
  (UTC+0). Le calcul utilise le fuseau **de la ville** (`chrono-tz`, `resolve_offset`
  + `display_clock`), donc les heures restent locales-correctes même si le fuseau
  machine diffère ; à reconfirmer un jour de Ramadan. Vérifié le 19/09/2026 :
  AlAdhan renvoie lui aussi des heures UTC+1 pour Casablanca (test
  `calibration_casablanca_against_aladhan`) — l'ancienne note « base UTC+0 hors
  Ramadan » ne se vérifie plus.
- **Base de fuseaux figée — correctif 23/09/2026** : `chrono-tz` embarque un
  snapshot tzdata **gelé** et ne le rafraîchit qu'à ses propres releases. Le
  Maroc est repassé **UTC+0 le 20/09/2026** (tzdata 2026d), invisible pour
  chrono-tz 0.10.4 (qui répond encore UTC+1) → horaires décalés d'une heure.
  Comme l'OS tient sa propre base à jour, `commands::effective_timezone` renvoie
  `None` (→ horloge machine, `chrono::Local`) quand le fuseau configuré **est
  celui de la machine** ; un fuseau de ville volontairement différent continue
  d'utiliser chrono-tz. Limite assumée : machine dans un autre fuseau que la
  ville choisie ⇒ on reste dépendant du snapshot chrono-tz (aucune release plus
  récente n'existe).
- Le `format_clock` ne normalise pas au-delà de 24 h (gère le rollover après minuit) ; les heures sont en minutes depuis minuit local.
- **Géoloc auto + méthode** : le bouton « loupe » de la fenêtre Réglages détecte la position (ip-api.com) et **auto-sélectionne la méthode du pays** (table `COUNTRY_METHOD` dans `settings.js` ; repli MWL). Ex. utilisateur au Maroc → méthode Maroc automatiquement.
- **Icône loupe** (SVG vectoriel, `currentColor`, pas d'emoji) remplace le texte « Utiliser ma position ».
- `appearance: none` sur les `<select>` (WebKitGTK rendait le natif clair) + flèche chevron SVG personnalisée → sombre partout.

## Session Windows 31/08 — fuseaux ville, icône, premier lancement, README

- **`chrono-tz`** : nouveau champ `timezone` dans la config (IANA, ex. `Africa/Casablanca`). `commands::resolve_offset` calcule le décalage UTC réel de la **ville** (avec DST) via `chrono_tz`, repli sur le fuseau machine. Règle proprement le piège Maroc/DST. La géoloc récupère le fuseau (ip-api `timezone`).
- **Icône refaite** (`scripts/gen_icon.py`) : surface sombre Fluent + croissant/étoile bleu accent `#60cdff` ; toutes les icônes régénérées. `assets/` (banner, logo, screenshots) + `README.md` complet ajoutés.
- **Premier lancement** : sans position configurée, le widget affiche « Configurer la position » ; un clic ouvre les réglages (`state.hasLocation`).
- **Bundle NSIS** : `targets: ["nsis"]`, `installMode: "currentUser"` → installation par utilisateur, sans admin.
- **Bouton « Réduire »** : dans la vue détaillée, remplace « Fermer » → cache le widget dans le tray (`hide_window`). Quitter reste dispo via le menu tray.
- **Auto-détection au premier lancement** : `commands::detect_location` (Rust, `ureq` sans TLS → pas de souci « mixed content » WebView2) interroge ip-api.com et applique automatiquement ville/lat/lon/**fuseau**/**méthode du pays** quand aucune position n'est configurée. Le bouton loupe des réglages utilise la même commande (plus de duplication du `COUNTRY_METHOD` → déplacé dans `commands::country_method`).
- **Raccourcis install** : Tauri NSIS crée **déjà par défaut** le raccourci du **menu Démarrer** ET celui du **bureau** (`installer.nsi`) — rien à configurer.
- **Fix DPI (petits écrans / 125-150 %)** : `toggleView` passait la largeur en pixels **physiques** → à haute échelle le widget devenait 160/120 px logiques et tout se mélangeait. Passé en **taille logique** (`LogicalSize`) ; vue détaillée élargie à **300 px logiques** ; ancrage **coin bas-droite** (grandit vers le haut/gauche depuis la barre). Position drag sauvegardée = coin bas-droite.
- **Fix fuseau horaire** : quand le fuseau configuré ≠ fuseau machine, le compte à rebours/la date étaient décalés (heures en fuseau ville mais `now` en horloge machine). Tout utilise désormais l'horloge du fuseau configuré (`display_clock` : date, minutes courantes, rollover, hégire). Ajout d'un **sélecteur de fuseau** dans les réglages (~45 fuseaux) pour choisir volontairement un autre fuseau.

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

Exécutable : `target\debug\miqati.exe`.
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

### Lot 2 — fait ✅ (session 12/09/2026, Windows)
- **Notifications système** (`notifier.rs` + `tauri-plugin-notification`) : rappel avant + à l'heure, scheduler Rust testé (12 tests `notifier::tests`).
- **Compte à rebours fiabilisé** : calculé depuis une échéance absolue (`nextAt`), plus de dérive ni de gel après veille (tests `src/main.test.mjs`).
- **Compteur de la vue détaillée** : la ligne « prochaine prière » décrémente aussi (elle était figée tant que la vue restait ouverte).
- **Config corrompue** : `config::load()` ne perd plus les réglages (lecture champ par champ tolérante + `config.bak.json`).
- **i18n du menu tray** + titre de la fenêtre Réglages (reconstruits à chaud au changement de langue).
- **Offsets ±minutes par prière** : implémentés et testés côté backend, puis **retirés de l'interface** (le réglage des horaires doit rester simple).
- **Ajustement hijri −2…+2 jours** : le `+1` figé est devenu un réglage ; lui aussi **retiré de l'interface** ensuite, seul le backend le porte.
- **Mode hors ligne** (défaut désactivé) + README corrigé (il promettait « Fully offline », c'était faux).

### À tester sous Windows (boot)
- Vol de focus : cliquer le widget ne doit pas sortir le clavier de l'app active.
- Positionnement contre la barre (bas/haut/gauche/droite) et multi-écrans.
- Auto-masquage en plein écran (jeu/vidéo) puis réapparition.
- Autostart (registre) + démarrage masqué + drag/mémorisation.
- Fenêtre de réglages : enregistrer → le widget se met à jour (langue, 12/24 h, horaires) sans relancer.
- Notifications : **nécessitent une build installée** (un toast Windows exige un raccourci Menu Démarrer/AppUserModelID que le NSIS crée ; `cargo tauri dev` ne suffit pas).

### Reste (idées)
- P3 — compte à rebours dans l'icône du tray : redessiner l'icône avec le temps restant (ex. « 135 ») chaque minute via `tray.set_icon()`. Lisibilité limitée (16×16) → décision : on garde le tray tel quel pour l'instant.
- **Mode « pastille »** (~90 px : icône + `1:23`) pour les petits écrans, évoqué comme alternative au widget large.
- **Géoloc en HTTPS** : l'offre gratuite d'ip-api.com est HTTP uniquement ; il faudrait un fournisseur payant pour du TLS.
- Nettoyer le warning `.rsrc merge failure` si possible.
- **VS Build Tools + toolchain MSVC** : la toolchain `stable-x86_64-pc-windows-msvc` est installée mais **pas** les Build Tools (`cl.exe`/`link.exe` absents) → `cargo test` ne démarre pas sous Windows (`api-ms-win-core-winrt-error-l1-1-0.dll` introuvable, `STATUS_ENTRYPOINT_NOT_FOUND`), à cause du plugin notification. Les tests s'exécutent donc sur **Linux** (46 verts).

## Session 13/09/2026 — Linux : CI, test cassé, régression watcher

Relecture de la passe Windows depuis Linux, là où la suite peut réellement tourner.

- **`offsets_feed_the_countdown` échouait.** Assertion inversée : décaler Dhuhr de
  +30 min **allonge** le temps restant (`base=1676`, `shifted=3476`, delta exactement
  1800 s). Le code des offsets était juste. Le `if base.next_name == "Dhuhr" && …`
  rendait en plus le test silencieusement vide si la prochaine prière changeait →
  remplacé par deux `assert_eq!` sur la précondition.
  **Invisible sous Windows** : sans Build Tools MSVC, `cargo test` n'y démarre pas.
- **Régression dans `spawn_window_watcher`.** `hidden_by_fullscreen = true` était
  posé *hors* du garde `if window.is_visible()`. Conséquence : widget masqué via le
  tray → une app passe en plein écran → en sortant, `show()` le **ressuscitait**
  contre la volonté de l'utilisateur. Le garde étant réévalué tous les 300 ms, un
  widget affiché *pendant* le plein écran est masqué au tick suivant de toute façon :
  le flag n'avait aucune raison d'être inconditionnel. Remis à l'intérieur du garde.
- **`&lang=fr` codé en dur** dans l'URL ip-api → un anglophone au Caire voyait
  « Le Caire ». `ip_api_lang()` suit désormais la langue configurée (ip-api ne parle
  pas arabe → repli sur l'anglais).
- **Base qualité assainie** : `cargo fmt` appliqué, 6 warnings clippy corrigés
  (`z -= x`, `#[derive(Default)]` + `#[default]` sur `CalculationMethod` et
  `PrayerOffsets`, `assert!(!x)`). 0 warning, `fmt` conforme.
- **`A_TESTER_WINDOWS.md` retiré du dépôt** : checklist de session, dont le savoir
  durable est déjà dans ce fichier.

### CI (`.github/workflows/ci.yml`)

Deux jobs, motivés par ce qui vient de se produire :

- **Tests (Linux)** — `fmt --check`, `clippy`, `cargo test --workspace`,
  `node --test src/main.test.mjs`. C'est le seul endroit où la suite tourne.
- **Build (Windows, MSVC)** — `cargo clippy --all-targets` compile `win32.rs`, que
  le job Linux ne parse même pas (`#![cfg(target_os = "windows")]`), plus les tests
  de `salaat-core`. Comme les Build Tools manquent en local, c'est **le seul endroit
  où le code Win32 est type-check**.

`RUSTFLAGS: -D warnings` : la base étant propre, autant la garder ainsi.

### Build de l'installeur en CI

Le job Windows produit aussi le bundle NSIS (`tauri-action` sans `tagName` =
build seul, pas de publication), uploadé en artefact 14 jours.

Trois raisons, dans cet ordre d'importance :

1. **La CI build mieux que la machine de dev.** Pas de Build Tools MSVC en local
   → repli GNU → `.rsrc merge failure` et binaires de test inchargeables. GitHub
   fournit MSVC d'office.
2. **Ça débloque le test des notifications.** Un toast Windows exige un raccourci
   Menu Démarrer (AppUserModelID) créé par le NSIS : il faut donc une build
   *installée*, exactement ce qui est pénible à produire localement.
3. **Substitut à la signature de code.** Pas de certificat (mauvais rapport
   qualité/prix, et SmartScreen ne se tait qu'après réputation). `release.yml`
   attache une **attestation de provenance** (`gh attestation verify`) + des
   checksums SHA256 : vérifiable, et c'est ce que le public FOSS regarde.

Effet de bord utile : on ne reboote plus sous Windows **pour builder**, seulement
**pour tester**, avec un installeur déjà prêt.

`release.yml` se déclenche sur un tag `v*` et crée une release **draft** — rien
n'atteint les utilisateurs sans validation manuelle.

**Piège évité** : c'est un workspace Cargo, donc le bundle est dans
`target/release/bundle/nsis/`, **pas** `src-tauri/target/…`.

### À vérifier au prochain boot Windows

- Le correctif du watcher : masquer le widget via le tray, lancer une vidéo plein
  écran, en sortir → **le widget doit rester masqué**.
- Non-régression z-order (le fichier a été touché).
- Premier run de CI : que le job Windows compile `win32.rs` et sorte l'installeur.

## Code signing (Windows)

Statut : **plomberie en place, inactive tant qu'aucun compte n'est configuré.**
Rien n'est signé aujourd'hui ; voici pourquoi, et comment on l'active.

### Décision : pourquoi SignPath, et pas un certificat de CA

- **Auto-signer le `.exe` ne sert à rien** contre SmartScreen : c'est bon pour des
  tests internes, mais un certificat inconnu du grand public ne fait pas disparaître
  l'avertissement chez l'utilisateur.
- **EV n'apporte plus rien.** Depuis 2024 Microsoft a retiré le traitement de faveur
  des certificats EV : OV comme EV construisent la réputation SmartScreen de la même
  façon, et une release fraîchement signée peut encore avertir (cf. doc Tauri
  « Windows Code Signing »).
- Les certificats OV/EV délivrés **après le 1er juin 2023** exigent un **HSM/token
  matériel** → difficiles à utiliser en CI.
- Les services cloud payants (Azure Artifact Signing, ~10 $/mois) règlent le problème,
  mais **miqati ne peut pas payer** → **SignPath Foundation**, qui signe
  **gratuitement les projets open source** (Miqati est public GPL-3.0). C'est le
  chemin retenu.

### Ce qui est branché (`.github/workflows/release.yml`)

- Après le build `tauri-action`, l'installeur est **ré-uploadé comme artefact de
  workflow** (SignPath signe des artefacts GitHub, pas des fichiers locaux), puis
  une **demande de signature** est soumise ; l'installeur signé remplace l'asset
  non signé du draft (`gh release upload --clobber`).
- L'attestation de provenance et les checksums portent ensuite sur **le fichier
  réellement publié** (signé si la signature a eu lieu, sinon celui du build).
- Les étapes de signature sont sautées si `SIGNPATH_API_TOKEN` est absent → **les
  releases continuent de fonctionner, non signées, sans rien casser**
  (idem job Windows de `ci.yml`, qui ne change pas).

### À faire une fois (compte SignPath, manuel)

1. Rejoindre la **SignPath Foundation** (gratuit OSS) et créer un projet pour
   `ismail-bahloul/Miqati`, avec le *Trusted Build System* **GitHub.com** lié et
   l'app GitHub SignPath installée.
2. Créer l'**Artifact Configuration** en uploadant un **échantillon de l'installeur**
   (SignPath le génère ; l'upload-artifact zippe par défaut → la racine sera
   `<zip-file>`). Si son slug n'est pas celui par défaut, ajouter
   `artifact-configuration-slug: <slug>` aux `with:` de l'étape « Sign the installer ».
3. Poser les secrets : `SIGNPATH_API_TOKEN`, `SIGNPATH_ORGANIZATION_ID`. Vérifier
   que `project-slug` / `signing-policy-slug` dans le workflow correspondent au projet.

### À vérifier (Windows, non testable depuis Linux)

- Contraintes SignPath OSS : **tous les jobs du workflow doivent tourner sur des
  runners GitHub-hosted** (c'est le cas : `windows-latest`).
- Après activation : l'installeur doit porter une signature (clic droit → Propriétés
  → onglet *Signatures numériques*) et l'onglet « Éditeur » ne doit plus afficher
  « Éditeur inconnu ».
- Même avec signature, SmartScreen peut encore avertir tant que la réputation du
  certificat SignPath ne s'est pas construite.

## Session 12/09/2026 — Windows : z-order, largeur, lot 2

### Z-order : ce qui a été essayé et écarté

Le widget devait rester devant la barre des tâches. Table des tentatives :

| Approche | Verdict |
|---|---|
| `SetWindowPos(HWND_TOPMOST)` seul | **no-op** quand la fenêtre a déjà `WS_EX_TOPMOST` — c'était le bug de départ |
| `NOTOPMOST` → `TOPMOST` enchaînés | remonte, mais **scintille** (repaint intermédiaire) |
| Vraie AppBar (`ABM_NEW`) | réserve une bande → **rétrécit toutes les autres fenêtres** (Zed compris) |
| Détection par `WindowFromPoint` | **mente** : renvoie notre fenêtre même quand le shell est visiblement dessus |
| `SetWindowPos(hwnd, shell_hwnd, …)` | met le widget **dessous** (`hWndInsertAfter` *précède*) |
| **`HWND_TOP` ciblé** (retenu) | reste dans la bande topmost → pas de scintillement |

**Retenu** : `shell_above()` marche l'ordre Z depuis notre fenêtre pour savoir si une
surface shell est devant ; si oui, `raise_above()` (`HWND_TOP`). Plus le placement
**au-dessus** de la barre (`position_near_taskbar` ancre sur le bord intérieur) :
si le widget ne chevauche pas la barre, le problème n'existe plus.

**Assumé** : le shell Windows (barre, Start, volets) reste devant. Toutes les
apis Win32 publiques échouent à garantir l'inverse ; TrafficMonitor utilise
`SetParent` dans `Shell_TrayWnd`, fragile aux redémarrages d'Explorer → écarté.

### Autres correctifs de la session

- **Clic sur le bureau masquait le widget** : `foreground_is_fullscreen()` ne
  regardait que la géométrie (rect de la fenêtre au premier plan == rect du
  moniteur). Or `Progman`/`WorkerW` (le bureau) couvrent tout l'écran → cliquer
  sur le bureau était pris pour un passage en plein écran et **cachait le
  widget**. Corrigé par `is_shell_window()`, qui exclut les surfaces du shell
  (bureau, icônes, barre des tâches, Start, volets) de la détection.
- **Offsets ±minutes par prière : retirés de l'interface.** Le principe retenu
  est que le réglage des horaires doit rester **simple** (loupe → ville →
  horaires justes). Exposer 6 champs numériques invitait à l'erreur (un décalage
  oublié = horaires faux, incompréhensibles pour l'utilisateur). La
  fonctionnalité reste dans la config et le backend (`PrayerOffsets`,
  `validate_config`), simplement plus éditable depuis l'UI. L'**ajustement
  hijri** (−2…+2) est conservé : un seul sélecteur discret, et la variation des
  mois lunaires selon les pays est un vrai besoin.
- **Réglages simplifiés** (même session) : le formulaire exposait 12 contrôles à
  plat, dont 4 techniques (lat/lon, méthode, école, hautes latitudes, fuseau).
  Restructuré :
  - **Vue principale** : Ville + loupe, Langue + Format de l'heure, **Rappels**
    (un seul sélecteur), puis les 4 cases d'options.
  - **« Réglages avancés »** : `<details>` replié par défaut → lat/lon, méthode,
    école, hautes latitudes, fuseau, ajustement hégirien. La loupe règle tout ça
    pour l'utilisateur normal.
  - **Rappels = un seul `<select>`** (« Désactivés / À l'heure / Avant / Les
    deux ») au lieu de 2 cases + 1 sélecteur. Le mapping vers les deux booléens
    du backend se fait dans `notifyModeFrom()` / `notifyFlags()` → **aucune
    migration de config**.
- **Ajustement hégirien retiré de l'interface** (même session) : trop opaque pour
  l'utilisateur (il ne peut pas savoir s'il doit choisir −1 ou +1). La valeur
  reste dans la config/le backend, réglable à la main dans `config.json`.
  Principe appliqué : **l'interface ne montre que ce qui est actionnable et
  compréhensible** — Ville + loupe, Langue, Format, Rappels, 4 cases ; le reste
  (lat/lon, méthode, école, hautes latitudes, fuseau) sous « Avancé ».
- **Régression évitée** : le formulaire poste une config complète, donc les
  champs qu'il ne rend pas (`window_position`, `offsets`, `hijri_adjust`)
  étaient **écrasés par leurs défauts à chaque sauvegarde**. Centralisé dans
  `preserve_hidden_fields()` (côté Rust) + test
  `form_saves_keep_the_fields_hidden_from_the_ui`.
- **Bouts blancs au scroll des Réglages** : seul le `body` portait le dégradé,
  la WebView montrait son canevas blanc en butée/rebond. Corrigé en peignant
  aussi `html`, avec `background-attachment: fixed` et `overscroll-behavior: none`.
- **Drag** : une logique « anti-chevauchement de la barre » ajoutée puis retirée —
elle **téléportait** le widget dès qu'on le posait sur la barre.
- **Plein écran** : masquage revérifié à chaque tick (le premier plein écran
  n'était pas masqué) et tolérance ±2 px sur la comparaison des rects.
- **Largeur** : retour à une largeur **unique** compact/détail (240 px), chiffres
  13 px. Un essai à 200 px a été jugé trop étroit.
- Le démarrage du dev server se fait via **`Start-Process` détaché** (voir
  `A_TESTER_WINDOWS.md`) : les tâches planifiées tuent leur arbre de processus.

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
- Tests : `salaat-core` (10) OK sous Windows. Les tests de la lib (`miqati_lib`) **crash au chargement** (`0xc0000139` entry point not found) avec la toolchain GNU : les import libs du crate `windows` 0.52 référencent des ordinaux non résolus dans le binaire de test (pas dans l'exe principal). **Workflow : tester sur Linux** (ça passe), compiler/lancer sur Windows.
- App lancée : process stable (~40 Mo), fenêtre « Miqati » responsive.

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

## Sélecteur de ville + feedback de position (23/09/2026)

- **Problème** : le champ « Ville » des Réglages n'était qu'un **libellé**. Taper un
  nom ne faisait rien — et rien ne le disait : `autoSave` sortait en silence si
  lat/lon étaient vides, et `validate_config` exige des coordonnées.
- **Liste de villes embarquée** : `scripts/gen_cities.py` télécharge GeoNames
  (`cities5000` + `admin1CodesASCII`, CC BY 4.0 — attribution dans le README) et
  écrit `src/cities.json` : **12 614 villes dont toutes les villes marocaines**
  (règle : pop ≥ 50 000 dans le monde, ou pays = `MA` quelle que soit la taille),
  triées par population. Format compact indexé (table de fuseaux partagée, région
  ajoutée seulement en cas d'homonymie dans le pays), orthographes françaises pour
  les grandes villes marocaines (GeoNames donne parfois « Fes »/« Marrakesh »).
- **Recherche** : `src/cities.js`, module ES **pur** testé par `node --test` —
  insensible aux accents et à la casse (« tetouan » → « Tétouan »), préfixe classé
  avant sous-chaîne, plafonnée à 12 résultats.
- **UI** : combobox (suggestions sous le champ, clavier ↑/↓/Entrée/Échap) ;
  sélectionner une ville remplit ville + lat/lon + **fuseau** + **méthode du pays**
  (`commands::method_for_country`, même règle que la géoloc), puis auto-save.
  `settings.js` est passé en `type="module"` pour importer le module de recherche.
- **Ligne de statut** sous le champ : « ✓ <lat>, <lon> · <fuseau> » quand une
  position est définie, sinon « Aucune position définie… ». Elle décrit ce que
  l'app utilisera *réellement*, donc un libellé tapé mais non résolu ne peut plus
  passer pour enregistré.
- **Bug latent corrigé** : la liste de fuseaux du markup (~45 zones) ne couvre pas
  les 284 fuseaux des villes ; une ville hors liste faisait retomber le `<select>`
  sur « Local » à chaque enregistrement (horaires décalés). `setTimezoneValue`
  ajoute l'option à la demande.
- **Fichiers** : `src/cities.json`, `src/cities.js`, `src/cities.test.mjs`,
  `scripts/gen_cities.py` + `settings.{html,js,css}`, `commands.rs`,
  `src-tauri/src/lib.rs`, `.github/workflows/ci.yml` (2ᵉ fichier de test).

## Notification de mise à jour (23/09/2026)

- **Contexte** : aucun mécanisme de mise à jour, donc un user installé reste sur
  une version fausse (règles de fuseau, etc.) sans le savoir. La signature de
  code (SignPath) étant repoussée, l'updater silencieux n'est pas envisageable
  (un install silencieux non signé peut déclencher SmartScreen — pire que rien).
- **Choix** : un simple **avis de mise à jour**. Vérifie la dernière release
  publiée sur `api.github.com` **une fois au lancement** (+ toutes les 24 h, le
  widget pouvant tourner des semaines), puis affiche une ligne discrète :
  bouton accent dans la vue détaillée du widget, et `Miqati x.y.z · Version z
  disponible` dans les Réglages. Rien n'est téléchargé ni installé : le clic
  ouvre la page des releases dans le navigateur.
- **Pas de TLS côté Rust** : la requête part du **webview** (`fetch` HTTPS, il a
  déjà une pile TLS et l'API GitHub est en CORS `*`), donc **pas de feature TLS
  ajoutée à `ureq`** — ce qui aurait cassé le build GNU local (voir la note sur
  le toolchain). Le parsing/comparaison est pur et testé (`src/update.test.mjs`).
- **Offline mode respecté** : aucune requête d'update en mode hors ligne, dans
  les deux fenêtres. `PRIVACY.md` + `README.md` mis à jour : la promesse n'est
  plus « une seule requête » mais « deux, toutes deux coupées en mode hors ligne ».
- **`tauri-plugin-opener` réintroduit** (il avait été retiré comme inutilisé) :
  nécessaire pour ouvrir la page des releases. Appelé **côté Rust**
  (`open_update_page`, URL constante) → pas de permission capability ajoutée et
  le front ne peut pas faire ouvrir n'importe quelle URL.
- **Commandes ajoutées** : `app_version` (version depuis `package_info`) et
  `open_update_page`.
- **Note** : la vérification ne fait rien tant qu'aucune release n'est *publiée*
  (`releases/latest` renvoie 404 sur un dépôt sans release publiée). Normal.
- **Fichiers** : `src/update.js`, `src/update.test.mjs`, `src-tauri/src/commands.rs`,
  `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `index.html`, `styles.css`,
  `main.js`, `main.test.mjs`, `settings.{html,js,css}`, `PRIVACY.md`, `README.md`,
  `.github/workflows/ci.yml`.