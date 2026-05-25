# CI Optimization: Path Filtering + Cancel-in-Progress

> **Status:** Implemented — PR #22
> **Merge queue abandonné** (non disponible sur repos personnel GitHub)
> **Solution retenue :** workflow unique `ci.yml` avec path filtering

## Résumé

**Problème :** PR docs-only prenaient 20-30 min (tous les jobs Rust).
**Solution :** job `changes` avec `dorny/paths-filter` détecte les changements Rust.
**Résultat :** PR docs-only → ~30s. Force-push → run précédent annulé.

## Changements

- `.github/workflows/ci.yml` : ajout `changes` job + `concurrency` + `if` gates sur tous les jobs
- `.github/workflows/ci-full.yml` : supprimé (créé pour merge queue, inutile sans)
- `.github/workflows/ci-stub.yml` : supprimé (idem)

## Architecture finale

```
jobs:
  changes:          # Toujours tourne (~30s)
    # dorny/paths-filter → outputs: rust=true/false

  lint:             # if: rust || push to main
  test:             # if: rust || push to main
  audit:            # if: rust || push to main
  check-cross:      # if: rust || push to main (macos + windows)
  build-cli:        # if: rust || push to main
  bench:            # if: rust || push to main
  udeps:            # if: rust || push to main
  coverage:         # if: rust || push to main
  fuzz:             # if: rust || push to main (3 targets)
  mutants:          # if: rust || push to main (45 min timeout)
  sbom:             # needs build-cli
  release:          # if: startsWith(github.ref, 'refs/tags/v')
```

## Liens

- Spec: `docs/superpowers/specs/2026-05-25-ci-optimization-merge-queue.md`
- PR: https://github.com/giak/reseau-racine/pull/22
