# Phase 2 Sécurité Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add coverage, mutation testing, and SBOM CycloneDX to CI pipeline.

**Architecture:** 3 independent CI jobs added to `.github/workflows/ci.yml`. `coverage` and `mutants` run in parallel with existing jobs; `sbom` depends on `build-cli` artifact. No code changes to Rust crates. No new Ruleset checks.

**Tech Stack:** cargo-llvm-cov (via taiki-e/install-action), cargo-mutants (via cargo install), auditable2cdx (via cargo install), GitHub Actions upload/download-artifact.

---

### Task 1: Coverage CI job

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add `coverage` job to ci.yml**

Insert after the `udeps` job block (before `fuzz`), add:

```yaml
  coverage:
    name: coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-llvm-cov
      - run: cargo llvm-cov --workspace --exclude rr-tauri --lcov --output-dir coverage
      - uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: coverage/
```

- [ ] **Step 2: Verify CI syntax**

Run: `./scripts/dev.sh cargo check` — ensures ci.yml change doesn't break anything (the change is only YAML, but confirm project still builds). Actually, this step is just a YAML validation check. Use `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` or just confirm visually.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(ci): add cargo-llvm-cov coverage job"
```

---

### Task 2: Mutants CI job

**Files:**
- Create: `.cargo/mutants.toml`
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Create `.cargo/mutants.toml`**

Exclude binaries and test-only paths to speed up mutations:

```toml
# Exclude binary entrypoints and fuzz targets — library code only
exclude = ["**/main.rs", "**/fuzz_targets/**"]
# Skip mutations in test code itself
skip_test = true
```

- [ ] **Step 2: Add `mutants` job to ci.yml**

Insert after the `coverage` job block:

```yaml
  mutants:
    name: mutants
    runs-on: ubuntu-latest
    timeout-minutes: 45
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: swatinem/rust-cache@v2
      - run: cargo install cargo-mutants --locked
      - run: cargo mutants --workspace --exclude rr-tauri
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: mutants-report
          path: mutants-out/
```

- [ ] **Step 3: Commit**

```bash
git add .cargo/mutants.toml .github/workflows/ci.yml
git commit -m "feat(ci): add cargo-mutants mutation testing job"
```

---

### Task 3: SBOM CycloneDX CI job

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Add `sbom` job to ci.yml**

Insert after the `mutants` job block:

```yaml
  sbom:
    name: sbom
    runs-on: ubuntu-latest
    needs: [build-cli]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: swatinem/rust-cache@v2
      - uses: actions/download-artifact@v4
        with:
          name: rr-cli-linux
          path: target/release/
      - run: chmod +x target/release/rr
      - run: cargo install auditable2cdx --locked
      - run: auditable2cdx target/release/rr > sbom-cyclonedx.json
      - uses: actions/upload-artifact@v4
        with:
          name: sbom-cyclonedx
          path: sbom-cyclonedx.json
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "feat(ci): add SBOM CycloneDX job via auditable2cdx"
```

---

### Task 4: Create branch + PR

- [ ] **Step 1: Create feature branch**

```bash
git checkout -b feature/security-phase-2
```

(Do this before the commits above, or move existing commits to this branch.)

- [ ] **Step 2: Create PR**

```bash
gh pr create --fill
```

Expected output: PR URL (e.g. `https://github.com/giak/reseau-racine/pull/5`)

- [ ] **Step 3: Wait for CI green, then squash merge**

```bash
gh pr merge --squash --admin
```
