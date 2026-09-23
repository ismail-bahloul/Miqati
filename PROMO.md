# Promo — plan et assets

Compagnon de `NOTES.md`, mais côté diffusion. À lire depuis la machine Windows
au moment de tourner les assets.

## Le pitch (une phrase)

Un widget qui garde la prochaine prière et un compte à rebours **en permanence**
sur la barre des tâches Windows — gratuit, open source, hors-ligne, sans pub ni
tracking, en FR/EN/AR.

Face à Athan (Azan) (freemium, propriétaire, il faut l'ouvrir), l'angle est :
**toujours visible, jamais intrusif, zéro réseau**.

## Le GIF (à faire AVANT de poster)

C'est le livrable qui manque : un widget de barre des tâches se vend en
mouvement, pas en capture statique.

### Scène A — héros, ~10 s (indispensable)

| Temps | Action |
|---|---|
| 0–10 s | Taper en continu dans un éditeur / le Bloc-notes |
| ~3 s | Cliquer une fois sur le widget, sans arrêter de taper |
| suite | Le texte continue de s'écrire sans interruption |

Montre **le compte à rebours qui décrémente** *et* **le focus qui ne bouge pas**.

### Scène B — interaction, ~6 s (recommandé)

Repos compact → clic → vue détaillée (5 prières + date hégirienne) → attendre →
clic sur le fond → retour compact.

### Scène C — plein écran, ~6 s (optionnel)

Lancer une vidéo plein écran → le widget se cache → sortir → il revient.

### Réglages de tournage

- **Outil** : [ScreenToGif](https://www.screentogif.com/) (gratuit, MIT) ;
  alternative ShareX.
- **Cadre serré** : une bande autour du widget + la barre des tâches
  (~500×160 px logiques), **pas** tout l'écran.
- **Avant de tourner** : ville configurée (vrais horaires, pas « Configurer la
  position »), bureau propre, aucune notification, **langue anglaise**.
- **Format** : 12 fps, ≤ 10 s, boucle, **≤ 3 Mo** par GIF.
- **Fichiers** : `assets/demo-compact.gif` (A), `assets/demo-detail.gif` (B).
- **Après dépôt** : câbler la section *Preview* du README sur les GIF.

## Checklist avant de poster

- [ ] GIF A déposé dans `assets/`
- [ ] Premier lancement à froid revérifié (désinstaller → réinstaller)
- [ ] `PRIVACY.md` en ligne (fait)
- [ ] Mention SignPath au README (fait)
- [ ] Release `v0.2.0` publiée (fait)

## Canaux, par ordre de rendement

- **A. Communautés musulmanes** (les vrais utilisateurs) : Reddit `r/islam`,
  `r/Muslim`, et surtout les subs **pays** (`r/Morocco`, `r/algeria`,
  `r/Tunisia`, `r/Egypt`, `r/pakistan`, `r/indonesia`, `r/malaysia`, `r/Turkey`)
  — c'est là que le FR/AR + les méthodes locales font la différence ; groupes
  Facebook d'associations/mosquées ; Discord/X.
  Vérifier les règles de self-promo de chaque sub.
- **B. Découverte « outil Windows »** : AlternativeTo (catégorie existante,
  concurrent Athan listé), annuaires freeware (Softpedia, MajorGeeks, SnapFiles,
  LO4D), Reddit `r/software`, `r/Windows11`, `r/freeware`, Product Hunt.
- **C. Amplification dev/OSS** : Show HN, `r/rust`, `r/tauri`. Peu
  d'utilisateurs, mais **des étoiles et de la crédibilité** — ce qui alimente
  aussi la case *Reputation* de la candidature SignPath.

## Mesure

Téléchargements par release, étoiles, issues. C'est le signal « marché » — et ce
qui dira s'il vaut le coup de relancer SignPath.
