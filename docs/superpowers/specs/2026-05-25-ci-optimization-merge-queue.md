# CI Optimization: Path Filtering + Cancel-in-Progress

**Date:** 2026-05-25
**Author:** giak (after design discussion)
**Status:** Implemented — PR #22

## Problem

Notre CI actuelle lance 8 jobs sur **chaque** PR, incluant check-cross (macOS, Windows), audit, bench, udeps. Pour les PR docs-only (~50% de nos PRs), ces 20+ minutes sont du gaspillage total — aucun code Rust n'a changé, aucun bug ne peut être détecté.

**Métriques avant optimisation :**
- PR code : ~20-30 min wall time
- PR docs-only : ~20-30 min wall time (gaspillage total)
- Runs redondants sur force-push (pas de cancel-in-progress)

## Solution: Workflow unique avec path filtering

Pas de merge queue (non disponible sur repos personnels GitHub). Workflow unique `ci.yml` avec :
1. **Path filtering** via `dorny/paths-filter` : détection des changements Rust
2. **Cancel-in-progress** : force-push annule le run précédent
3. **Jobs conditionnels** : tous les jobs sauf `changes` sont gated sur `needs.changes.outputs.rust == 'true' || github.event_name == 'push'`
4. **`sbom`** dépend de `build-cli` (même workflow, `needs` fonctionne)

### Architecture

```
PR docs-only :
  └─ changes job (30s, détecte rust=false)
  └─ lint/test/audit/check-cross/... → skipped
  └─ 8 required checks → skipped ✅
  └─ Total: ~30s

PR code :
  └─ changes job (30s, détecte rust=true)  
  └─ lint/test/audit/check-cross/... → run
  └─ 8 required checks → run ✅  
  └─ Total: ~5-15 min (inchangé)

Force-push :
  └─ cancel-in-progress annule le run précédent
  └─ Nouveau run commence proprement
```

### Workflow

`ci.yml` unique avec tous les jobs. Seul le job `changes` tourne toujours. Les autres jobs sont conditionnés :

```yaml
changes:
  name: Detect Changes
  runs-on: ubuntu-latest
  outputs:
    rust: ${{ steps.filter.outputs.rust }}
  steps:
    - uses: dorny/paths-filter@v3
      id: filter
      with:
        filters: |
          rust:
            - 'crates/**/*.rs'
            - 'crates/**/Cargo.toml'
            - 'Cargo.toml'
            - 'Cargo.lock'
            - 'rust-toolchain.toml'
            - '.github/workflows/ci.yml'

lint:
  if: needs.changes.outputs.rust == 'true' || github.event_name == 'push'
  ...
```

### Bénéfices

| Métrique | Avant | Après | Gain |
|----------|-------|-------|------|
| PR docs-only | 20-30 min | ~30s | 40-60x |
| PR code | 20-30 min | ~5-15 min | inchangé |
| Force-push waste | runs redondants | cancel-in-progress | éliminé |
| Complexité | 1 workflow | 1 workflow | inchangé |

### Risques et mitigations

| Risque | Mitigation |
|--------|------------|
| Path filter manque un fichier Rust → jobs skipped | Le filter inclut `.github/workflows/ci.yml` — une modif du workflow déclenche tout |
| `sbom` dépend de `build-cli` → besoin du artifact | Même workflow, `needs: [build-cli]` fonctionne (pas besoin de duplication) |
| Cancel-in-progress annule un run important | Seulement sur PR, pas sur push to main |

### Critères de succès

- [x] PR docs-only : <1 min, 8 checks skipped
- [x] PR code : tous les jobs runnent
- [x] Cancel-in-progress : force-push annule le run précédent
- [x] Ruleset inchangé (8 checks requis, bypass admin)
