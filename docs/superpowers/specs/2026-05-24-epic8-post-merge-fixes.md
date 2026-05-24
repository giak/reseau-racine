# EPIC 8 — Post-merge fixes

## Problem 1: Pipe mort dans `cmd_bench`

`rr bench` redirigeait stdout de `cargo bench` vers `/dev/null` via `Stdio::piped()`, avec `--output-format bencher` inutile car l'output n'était jamais lu.

**Fix:** Supprimer `stdout(Stdio::piped())` et `--output-format bencher`. Utiliser `status()` par défaut (héritage stdout). L'utilisateur voit le live output criterion.

**Fichiers:** `crates/rr-cli/src/main.rs`

## Problem 2: `--quick` en CI fragilise la détection de régression

`cargo bench --bench crypto -- --quick` utilise trop peu d'échantillons pour que criterion détecte fiablement des régressions >5%. La comparaison avec `--quick` est bruyante.

**Fix:** Enlever `--quick` de la commande CI. Criterion utilise son nombre d'échantillons par défaut (statistiquement fiable). Temps CI augmenté (~2-3 min) mais détection de régression correcte.

**Fichiers:** `.github/workflows/ci.yml`
