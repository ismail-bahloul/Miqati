# À tester sous Windows — lot 1

Synchronisé depuis Linux le **12/09/2026**. Base : commit `4bc71fd`, plus les
modifications du lot 1 (non commitées).

Sur Linux : **46 tests verts** (40 Rust + 6 front), zéro warning. **Rien de ce qui touche Win32 n'a
pu être compilé** — pas de toolchain Windows sur la machine Linux. C'est tout
l'objet de cette session de test.

Ce qui est **déjà couvert par des tests** et n'a donc pas besoin d'être repris à
la main :

- **Scheduler de notifications** (12 tests, `notifier::tests`) — réveil après
  veille, changement de ville, Shuruq, anti-doublon, ré-armement, cycle complet
  Fajr→Isha.
- **Boucle de tick du front** (6 tests, `node --test src/main.test.mjs`) — rendu
  initial, un seul repaint par minute, **reprise après veille**, bascule à zéro,
  vue détaillée non peinte quand repliée.
- **Capability `notification:default`** résolue par l'ACL au build → plugin
  correctement enregistré.
- **Ids `settings.html` ↔ `settings.js`** : concordance vérifiée.
- **L'app a réellement tourné sous Linux** (Plasma/Wayland, WebKitGTK 2.52.6) :
  démarrage propre, widget rendu, fuseau ville correct (machine UTC+1, Paris
  UTC+2 → horaires de Paris affichés), et **un vrai toast capté sur D-Bus**
  (`app_name=miqati`, titre `Isha`, corps `Isha dans 9 minutes`).
- **Les deux types de notification ont tiré en conditions réelles** (config Dakar,
  Isha à 20:21) : le rappel anticipé, puis « C'est l'heure de Isha » sur la
  transition, à l'heure exacte.
- **Glow pré-prière** vérifié par capture d'écran : bordure accent à 00:04 restant.
- **CPU au repos : 0,08 %** (app + 3 process WebKit, build *debug*) — la boucle de
  tick ne coûte plus rien.

> ⚠️ Un bug a été trouvé *grâce* à ce run et corrigé : le rappel annonçait le
> **délai réglé** au lieu du **temps réellement restant**. En régime permanent
> c'était juste à quelques secondes près, mais au lancement de l'app (ou au
> réveil) trois minutes avant une prière, il annonçait « dans 10 minutes ».
> Corrigé + test dédié (`the_reminder_announces_the_time_actually_left`).

Restent à valider à la main : **le z-order Win32** (aucune couverture possible) et
**la plomberie des toasts Windows** (AppUserModelID).

---

## ⚠️ Avant de commencer — ton WIP du 31/08 a été écrasé

La copie qui était ici avait des modifications **non commitées et non poussées**.
Elles sont sauvegardées dans `_wip-backup-2026-08-31/` :

- `wip.patch` — le diff complet (`git apply` pour le rejouer)
- `files/` — les 5 fichiers modifiés, tels quels
- `devlog.txt`, `dev-run.bat`, `enum.ps1`, `rel_check.json`

Ce WIP contenait deux choses distinctes :

1. **`position_above_taskbar`** — ancrage *au-dessus* de la barre (`rcWork.bottom`)
   au lieu de *dessus*. Son `spawn_topmost_keeper` retombait sur le même no-op
   `SetWindowPos(HWND_TOPMOST)` que celui corrigé ici, donc ça ne réglait pas le
   problème de z-order. **Non repris** — la correction du lot 1 s'attaque à la
   cause racine et garde le widget sur la barre.
2. **Widget à 240 px, polices 13 px, padding resserré.** Purement esthétique,
   sans rapport avec le bug. **Non repris non plus** : on est revenu à 300 px /
   17 px. À décider — voir « Décision en attente » plus bas.

`target/` n'a pas été touché : la recompilation sera incrémentale.

---

## Build

```
rustup default stable-x86_64-pc-windows-gnu
$env:PATH = "<WinLibs>\mingw64\bin;" + $env:PATH
cargo tauri dev
```
ou `dev-run.bat` (redirige la sortie vers `devlog.txt`).

Le warning `.rsrc merge failure` reste attendu et bénin.

⚠️ **Nouvelle dépendance** : `tauri-plugin-notification` 2.4.0 (tire
`tauri-winrt-notification`). Premier build plus long.

---

## 1. Le z-order — résolu autant que Windows le permet

**Ce qui n'allait pas :** `SetWindowPos(hwnd, HWND_TOPMOST, …)` est un **no-op**
quand la fenêtre a déjà `WS_EX_TOPMOST`. Windows saute la ré-insertion dans
l'ordre Z. Le keeper tournait 2×/s sans jamais rien remonter.

**Ce qu'on a essayé** (et pourquoi ça a été écarté) :

| Approche | Verdict |
|---|---|
| `HWND_TOPMOST` seul | no-op quand déjà topmost |
| `NOTOPMOST` → `TOPMOST` enchaînés | remonte, mais **scintille** (repaint intermédiaire) |
| Vraie AppBar (`ABM_NEW`) | réserve une bande → **rétrécit toutes les autres fenêtres** |
| `WindowFromPoint` pour détecter | **mente** : renvoie notre fenêtre même sous le shell |
| `SetWindowPos(hwnd, shell_hwnd, …)` | met le widget **sous** la barre (`hWndInsertAfter` *précède* : on atterrit derrière) |
| `HWND_TOP` ciblé | retenu — rester dans la bande topmost, donc sans scintillement |

**État final** (`src-tauri/src/win32.rs`) : `spawn_window_watcher` (300 ms) marche
l'ordre Z depuis notre fenêtre (`shell_above`) ; si une surface shell est devant
(`Shell_TrayWnd`, `Shell_SecondaryTrayWnd`, `Windows.UI.Core.CoreWindow`,
`XamlExplorerHostIslandWindow`, `TopLevelWindowForOverflowXamlIsland`), il appelle
`raise_above` (`HWND_TOP`), qui reste **dans** la bande topmost → pas de
scintillement. Le widget est en outre **placé au-dessus** de la barre
(`position_near_taskbar` ancre sur le bord intérieur), pas dessus.

**Limite connue et assumée :** la barre des tâches, le menu Démarrer et les
volets (wifi/batterie, calendrier, centre de notifications) sont composités par
le DWM dans une couche **au-dessus** de toutes les fenêtres utilisateur. Ils
peuvent donc recouvrir le widget — c'est le comportement voulu de Windows et
aucune API publique ne garantit de passer devant. Le widget les recouvre à
nouveau une fois le volet fermé. Pour éviter le problème au lieu de le combattre,
le docking par défaut le place **au-dessus** de la barre.

- [ ] Clic sur la barre des tâches → le widget repasse devant sans scintiller
- [ ] Menu Démarrer : il recouvre le widget (attendu) ; il réapparaît à la fermeture
- [ ] Centre de notifications / paramètres rapides → idem
- [ ] Clic sur l'horloge (volet calendrier) → idem
- [ ] **Pas de scintillement**, y compris pendant un drag de fenêtre d'app
- [ ] Maximiser une app → le widget ne reste pas coincé sous la barre
- [ ] Décocher « Toujours au premier plan » dans les réglages → le widget accepte
      d'être recouvert et le watcher ne le remonte plus
- [ ] Plein écran (jeu/vidéo) → masquage (≤ 0,3 s), puis réapparition en sortie
- [ ] Drag du widget → position conservée, y compris posé sur la barre
      (le watcher ne doit plus le re-docker de force)

## 2. Notifications (nouveau)

`src-tauri/src/notifier.rs`, côté Rust et pas front : le widget peut être masqué
dans le tray ou lancé en `start_hidden`, les rappels doivent partir quand même.

> 🔴 **Piège de test.** Sur Windows, un toast exige un raccourci Menu Démarrer
> (AppUserModelID). Le NSIS le crée, donc une build **installée** notifie ; un
> `cargo tauri dev` ou un exe portable, probablement **pas**. Si rien n'apparaît
> en dev, ce n'est pas forcément un bug — refaire le test avec l'installeur avant
> de conclure.

- [ ] Réglages : les 3 nouveaux contrôles sont présents et traduits (fr/en/ar)
      (la concordance des ids HTML/JS est vérifiée, donc c'est un contrôle visuel)
- [ ] Décocher « Notifier avant la prière » → le sélecteur de délai se grise
- [ ] Délai à 5 min, attendre une prière → toast « <Prière> dans 5 minutes »
- [ ] À l'heure pile → toast « C'est l'heure de <Prière> »
- [ ] Passer en anglais puis en arabe → les toasts suivants sont traduits
      (le rendu RTL arabe dans un toast Windows vaut le coup d'œil)
- [ ] Les deux cases décochées → plus aucun toast

> Anti-doublon, Shuruq ignoré, changement de ville et réveil après veille sont
> couverts par `notifier::tests` — inutile de les rejouer à la main, sauf si un
> comportement bizarre apparaît.

## 3. Compte à rebours — dérive et veille

`main.js` calcule maintenant le restant depuis une échéance absolue
(`state.nextAt`) au lieu de décrémenter un compteur.

> La reprise après veille et la bascule à zéro sont couvertes par
> `src/main.test.mjs` (horloge figée). Ne restent que les cas qui dépendent
> réellement de Windows :

- [ ] **Vraie** mise en veille Windows 10 min, réveil → compte à rebours juste
      tout de suite (le test simule l'horloge, pas la suspension du WebView2)
- [ ] Après Isha → bascule sur le Fajr du lendemain
- [ ] Le glow des 5 dernières minutes se déclenche toujours (~4:59 restant)

## 4. Vue détaillée — bug corrigé au passage

Le compte à rebours de la ligne « prochaine prière » était figé tant que la vue
restait ouverte (`renderTimes` n'était appelé qu'au refresh horaire).

- [ ] Ouvrir la vue détaillée et la laisser ouverte 3 min → la ligne surlignée
      décrémente bien chaque minute

## 5. Non-régression

- [ ] Le clic sur le widget ne vole pas le focus clavier de l'app active
- [ ] Drag → position mémorisée, conservée après relance
- [ ] Tray : afficher/masquer, « Docker à la barre », Quitter
- [ ] Réglages : ville / langue / 12-24 h → le widget se met à jour sans relancer
- [ ] DPI 125 % et 150 % → pas de chevauchement, largeurs correctes
- [ ] Barre des tâches en haut / à gauche / à droite

---

## Décision en attente

**Largeur du widget.** Ton WIP l'avait réduit à 240 px avec des polices 13 px ;
la sync est repartie de 300 px / 17 px. Les deux se défendent :

| | 300 px / 17 px (actuel) | 240 px / 13 px (ton WIP) |
|---|---|---|
| Lisible de loin | ✅ | ⚠️ |
| Encombrement de la barre | ⚠️ | ✅ |
| Écrans 1366×768 | serré | confortable |

Ça rejoint l'idée de **mode « pastille »** (icône + `1:23`, ~90 px) évoquée comme
alternative à l'auto-hide sur le bord. Si le mode pastille se fait, la largeur du
mode normal compte moins. Dis-moi ce que tu veux et je réapplique — le patch est
dans `_wip-backup-2026-08-31/`.

---

## Suite (lot 2)

- ✅ **Config corrompue = perte silencieuse des réglages — CORRIGÉ.**
  `config::load()` parsait le fichier avec `serde_json::from_str::<PrayerConfig>` :
  un seul champ invalide (ex. `notify_before_minutes: 300`, ou `hour12: "yes"`)
  faisait échouer **tout** le parse et retombait sur `PrayerConfig::default()` —
  ville, coordonnées, fuseau et réglages perdus sans un mot.

  Correctif (`src-tauri/src/config.rs`) :
  - `from_value_lenient()` lit le JSON en `serde_json::Value` et extrait chaque
    champ indépendamment ; un champ manquant ou invalide retombe sur son défaut
    (les nombres hors bornes `u8` sont **clampés**, pas jetés).
  - `read_config()` tente le fichier principal puis `config.bak.json`.
  - `save()` copie le précédent `config.json` en `config.bak.json` avant de
    l'écraser.

  **Validé en conditions réelles** : `config.json` volontairement corrompu
  (`notify_before_minutes: 300`) + `window_position` distinctive → le widget a
  été placé exactement à la position mémorisée (`bottomRight=(700,500)`),
  preuve que les champs valides sont bien récupérés au lieu d'un reset global.

- Offsets manuels ±minutes par prière (la demande n° 1 de cette niche)
- Ajustement hijri ±2 jours (`gregorian_to_hijri(date, 1)` est en dur)
- ✅ **i18n du menu tray — FAIT.** Le menu natif (côté Rust) était en français
  en dur, alors que le reste de l'app est traduit fr/en/ar. Il est maintenant
  construit par `tray::apply_tray_menu(app, lang)` : au démarrage avec la langue
  configurée, puis **reconstruit à chaud** depuis `set_config` quand la langue
  change. Le titre de la fenêtre Réglages suit aussi (`settings_window_title`) :
  « Réglages — Miqati » / « Settings — Miqati » / « الإعدادات — Miqati ».
  Restent en français : les messages d'erreur de `validate_config` (affichés
  seulement si le frontend envoie une config invalide, donc rare).
- ✅ **Offsets manuels ±minutes par prière — FAIT.**
  `PrayerOffsets` (6 × `i16`, `serde(default)`) dans la config, appliqué dans
  `compute_status_payload` **juste après** `builder.build()` et avant la
  sélection de la prochaine prière — donc l'affichage, le compte à rebours et
  les notifications restent cohérents (`notifier` passe par la même fonction).
  Le Fajr du lendemain porte aussi l'offset, sinon le compte à rebours sautait
  à la bascule de journée.
  Bornées à ±60 min par `validate_config` ; lues de façon tolérante côté config
  (un offset absent ou mal formé retombe sur 0 sans perdre le reste).
  Côté UI : la fonctionnalité a d'abord été exposée comme une grille de 6 champs
  numériques, puis **retirée de l'interface** (décision de l'utilisateur) : le
  réglage des horaires doit rester simple (loupe → ville → horaires justes), et
  6 champs invitent à l'erreur (décalage oublié = horaires faux sans explication).
  L'ajustement reste disponible dans la config et le backend.
  Tests ajoutés (compile sur Windows, s'exécute sur Linux) :
  `offsets_shift_the_reported_times`, `offsets_feed_the_countdown`,
  `offsets_default_to_zero_and_survive_a_roundtrip`.
  **Validé sur Windows** : démarrage sans erreur avec des offsets non nuls en
  config.
- ✅ **Ajustement hijri ±2 jours — FAIT.** `gregorian_to_hijri(date, 1)` était en
  dur. Le nouveau réglage `hijri_adjust: i32` (défaut **1**, donc aucune
  régression : c'est la valeur qui était appliquée avant) est passé depuis
  `compute_status_payload`. Borné à −2…+2 par `validate_config`, lu de façon
  tolérante (hors bornes → clampé, pas de perte de config).
  Côté UI : sélecteur `−2 … +2` dans les Réglages (fr/en/ar), auto-sauvegardé.
  Test ajouté : `hijri_adjust_defaults_to_one_and_is_clamped`.
  **Validé sur Windows** : démarrage sans erreur avec `hijri_adjust: -1`.
- ✅ **Géoloc / « Fully offline » — FAIT (approche retenue avec l'utilisateur).**
  Le README affirmait « Fully offline », ce qui était faux : la détection de
  ville au premier lancement part sur `http://ip-api.com`. Plutôt qu'un
  consentement séparé, on a ajouté un réglage **`offline_mode`** (défaut
  **désactivé**), qui promet qu'aucune requête ne part **sans action explicite** :
  - `commands::detect_location` refuse la requête en mode hors ligne (l'état est
    lu via `AppState`) ;
  - `main.js` n'appelle plus `autoConfigure()` (détection automatique) en mode
    hors ligne, et relit le flag à chaque `config-changed` ;
  - les horaires restent **toujours** calculés localement — le widget est
    pleinement fonctionnel hors ligne, seul le confort de la détection auto est
    perdu ;
  - côté Réglages, la case « Mode hors ligne » grise le bouton de géoloc avec un
    `title` explicatif (« Fait une requête réseau »), i18n fr/en/ar.
  README corrigé en conséquence (« les horaires sont calculés localement ; le
  seul usage réseau est la détection de ville optionnelle »).
  Test ajouté : `offline_mode_defaults_to_off_and_survives_a_roundtrip`.
  **Validé sur Windows** : démarrage sans erreur avec `offline_mode: true`
  (config remise au défaut ensuite).
  ⚠️ Reste HTTP, pas HTTPS : l'offre gratuite d'ip-api.com est HTTP uniquement.
  Un passage en HTTPS impliquerait un fournisseur payant (ipinfo.io,
  ipgeolocation.io…) — à décider séparément.

---

## ⚠️ `cargo test` ne démarre plus sous Windows (toolchain GNU)

Depuis l'ajout de `tauri-plugin-notification`, le binaire de test importe les
API sets WinRT et échoue **au chargement** :

```
error while loading shared libraries:
api-ms-win-core-winrt-error-l1-1-0.dll: cannot open shared object file
STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)
```

Ces `api-ms-win-core-winrt-*` ne sont pas des DLL physiques (elles sont
résolues par le loader via des API set contracts) ; le linker MinGW les réclame
en import direct, ce que le loader ne peut pas satisfaire. **L'application, elle,
démarre normalement** — c'est uniquement le harnais de test qui casse.

Conséquence : la validation par `cargo test` doit se faire **sur Linux** (où les
46 tests passent). Sous Windows, valider à la main. Une vraie solution serait de
passer la toolchain Windows en **MSVC** (`stable-x86_64-pc-windows-msvc`), ou de
rendre la notification conditionnelle pour les tests.

---

## 6. Bug : cliquer sur le bureau masquait le widget — CORRIGÉ

`foreground_is_fullscreen()` ne comparait que la **géométrie** (rect de la fenêtre
au premier plan == rect du moniteur). Le bureau (`Progman` / `WorkerW`, plus
`SysListView32` pour les icônes) couvre tout l'écran : cliquer sur le bureau était
donc interprété comme un passage en plein écran et **cachait le widget**.

Corrigé par `is_shell_window()`, qui exclut de la détection les surfaces du shell
(bureau, icônes, barre des tâches, Start, volets). Bénéfice de bord : cliquer sur
la barre ou le menu Start ne masque plus le widget non plus.

- [ ] Clic sur le bureau → le widget reste affiché
- [ ] Clic sur la barre des tâches → idem
- [ ] Menu Démarrer ouvert/fermé → idem
- [ ] Vraie vidéo/jeu en plein écran → le widget se masque toujours bien
