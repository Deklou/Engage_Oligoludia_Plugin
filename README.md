# Engage Oligoludia Plugin

Plugin Skyline pour **Fire Emblem Engage 2.0.0**. Il ajoute deux exceptions ciblées à Oligoludia, sans modifier les règles globales du jeu.

## Fonctionnalités

- **Break des ennemis immunisés** : Oligoludia peut briser la garde d'une cible normalement protégée, uniquement si elle a l'avantage d'arme et remplit les conditions natives de Break. Une autre arme avec le même avantage conserve l'immunité de la cible.
- **Efficacité malgré la protection du boss** : si le porteur possède déjà la compétence d'efficacité correspondante, Oligoludia ignore l'annulation de cette efficacité :
  - Aberrations (`SID_異形特効`) : multiplicateur de puissance d'arme **×3**.
  - Fell Dragons (`SID_邪竜特効`) : multiplicateur **×2**.
  - Sans compétence d'efficacité ni attribut correspondant, aucun bonus n'est ajouté.

Les deux variantes de l'arme sont prises en charge : `IID_リュール_オリゴルディア` et `IID_リュール_オリゴルディア_通常`.

La modification concerne le **multiplicateur de puissance de l'arme**, pas directement les dégâts finaux. L'effet de Break reste soumis à l'avantage d'arme, même si Oligoludia est modifiée en lance ou en hache par XML.

## Installation / mise à jour (WSL)

Prérequis : environnement Rust avec `cargo skyline`, modloader Skyline/Cobalt/Ozone fonctionnel, Fire Emblem Engage **v2.0.0** et Ryujinx.

Depuis la branche `main` du dépôt :

```bash
git pull --ff-only
```

Fermer complètement le jeu, puis :

```bash
bash scripts/deploy.sh
```

Le script compile le NRO puis l'installe dans `%APPDATA%/Ryujinx/sdcard/engage/mods/Oligoludia_Plugin/liboligoludia_plugin.nro`. Il sauvegarde une éventuelle version précédente en `.nro.bak` lors de la première mise à jour. Il ne modifie pas Terrain Conditions ni le reste d'`engage_modpack`.

## Vérification

Les logs Ryujinx exposent des traces limitées :

- `[oligoludia_plugin] Break check: ... override=true`
- `[oligoludia_plugin] GetBreaked: blocked=0x4000 -> break=0x20`
- `[oligoludia_plugin] Efficacy: ... multiplier=1 -> 3` (ou `-> 2` pour Fell Dragon)

**Testé en jeu le 18 novembre 2026** : Break contre une cible protégée, contrôle avec une autre lance ne provoquant pas le Break, et efficacité **×3** contre une Aberration protégée. Le dernier log Ryujinx montre simultanément les traces du Break et de l'efficacité ×3. L'efficacité ×2 des Fell Dragons est implémentée, mais n'a pas été isolément confirmée dans ce log.

Attention aux anciens contournements XML (`SID_オリゴルディア_ボス耐性無視`, bonus d'attaque fixe, ou `SID_オリゴルディア_ブレイク`) : ils sont indépendants du plugin et peuvent se cumuler s'ils sont encore équipés. Pour un comportement propre, conserver les compétences natives d'efficacité nécessaires, sans les anciens correctifs XML.

## Organisation du code

- `src/break_immunity.rs` : exceptions natives de Break.
- `src/efficacy.rs` : restauration ciblée du multiplicateur d'efficacité.
- `src/lib.rs` : installation des hooks.
- `scripts/deploy.sh` : compilation et déploiement local.

Les offsets de hooks sont spécifiques à **Engage 2.0.0**. Aucune compatibilité avec d'autres versions n'est garantie.
