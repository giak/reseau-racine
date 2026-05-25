# CI Optimization: sccache Cache Layer

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ajouter `mozilla-actions/sccache-action` à 8 jobs du CI pour partager les objets compilés entre jobs du même run et entre runs.

**Architecture:** `Swatinem/rust-cache` reste pour le cache au niveau crate (target/). sccache ajoute un cache objet (`.o`) complémentaire. Les jobs séquentiels (lint → test → audit → ...) réutilisent les objets déjà compilés par le job précédent.

**Tech Stack:** GitHub Actions, `mozilla-actions/sccache-action@v0`, Rust

---

### Task 1: Ajouter sccache aux 8 jobs Rust

**Files:**
- Modify: `.github/workflows/ci.yml` (jobs: lint, test, audit, check-cross ×2, build-cli, bench, coverage)

Chaque job reçoit 2 changements identiques :
1. Un step `- uses: mozilla-actions/sccache-action@v0` après checkout/rust-toolchain
2. Les env vars `RUSTC_WRAPPER: sccache` et `SCCACHE_GHA_ENABLED: "true"` au niveau job

Pattern pour chaque job (ex: `lint`) :

```yaml
  lint:
    name: lint
    needs: changes
    if: needs.changes.outputs.rust == 'true' || github.event_name == 'push'
    runs-on: ubuntu-latest
    env:
      RUSTC_WRAPPER: sccache
      SCCACHE_GHA_ENABLED: "true"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: mozilla-actions/sccache-action@v0
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --exclude rr-tauri -- -D warnings
```

- [ ] **Step 1: Add sccache to `lint` job**

  Éditer `ci.yml` : ajouter `env:` bloc avec `RUSTC_WRAPPER` et `SCCACHE_GHA_ENABLED`, ajouter step `mozilla-actions/sccache-action@v0` après rust-toolchain.

- [ ] **Step 2: Add sccache to `test` job**

  Même pattern — env + step sccache-action.

- [ ] **Step 3: Add sccache to `audit` job**

  Même pattern.

- [ ] **Step 4: Add sccache to `check-cross (macos-latest)` job**

  `runs-on: macos-latest` — env + step sccache-action.

- [ ] **Step 5: Add sccache to `check-cross (windows-latest)` job**

  `runs-on: windows-latest` — env + step sccache-action.

- [ ] **Step 6: Add sccache to `build-cli` job**

  `runs-on: ubuntu-latest` — env + step sccache-action.

- [ ] **Step 7: Add sccache to `bench` job**

  `runs-on: ubuntu-latest` — env + step sccache-action.

- [ ] **Step 8: Add sccache to `coverage` job**

  `runs-on: ubuntu-latest` — env + step sccache-action.

- [ ] **Step 9: Verify YAML syntax**

  Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); print('OK')"`
  Expected: `OK`

- [ ] **Step 10: Commit**

  ```bash
  rtk git add .github/workflows/ci.yml
  rtk git commit -m "ci: add sccache cache layer for faster compilation"
  ```
