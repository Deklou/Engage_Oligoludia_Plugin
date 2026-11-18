# Engage Oligoludia Plugin

Plugin Skyline pour **Fire Emblem Engage 2.0.0** : fonctionnalités propres à Oligoludia.

- `src/break_immunity.rs` : permet aux deux variantes d'Oligoludia de franchir l'immunité au Break, mais seulement avec l'avantage d'arme natif. Les autres armes restent inchangées.
- Efficacité contre les boss : évolution prévue dans une PR future.

## Test WSL (branche feat/break-immunity)

```bash
bash scripts/deploy.sh
```

Le script compile puis installe `liboligoludia_plugin.nro` sous `%APPDATA%/Ryujinx/sdcard/engage/mods/Oligoludia_Plugin/`, sans toucher au plugin Terrain Conditions.

**Statut : code non compilé/non testé en jeu.** Utiliser un boss immunisé et comparer avec une épée normale, puis vérifier Oligoludia sans avantage d'arme. Retirer les anciens skills XML de Break pour isoler le test.
