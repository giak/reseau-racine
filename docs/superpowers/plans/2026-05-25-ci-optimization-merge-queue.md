# CI Optimization: Merge Queue + Tiered Workflows — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split CI into Tier 1 (PR: fast) and Tier 2 (merge queue: full suite) with path filtering.

**Architecture:** 3 workflows: `ci.yml` (PR+merge_group, fast checks), `ci-full.yml` (merge_group only, full suite), `ci-stub.yml` (pull_request_target, satisfies required checks at PR time). GitHub merge queue enabled on `main` with squash-only.

**Tech Stack:** dorny/paths-filter@v3, GitHub Merge Queue, dtolnay/rust-toolchain, Swatinem/rust-cache, cargo-auditable, cargo-deny, cargo-udeps, cargo-llvm-cov, cargo-mutants, cargo-fuzz

---

### Task 1: Modify `ci.yml` — Tier 1 with concurrency, path filtering, split lint

**Files:**
- Modify: `.github/workflows/ci.yml` (full rewrite)

**Changes from current:**
1. Add `concurrency: cancel-in-progress` for PR
2. Add `pull_request` + `merge_group` triggers
3. Add `changes` job with dorny/paths-filter
4. Gate `lint`, `test`, `build-cli` on `needs.changes.outputs.rust == 'true'`
5. Remove jobs moving to Tier 2: `audit`, `check-cross`, `udeps`, `bench`, `fuzz`, `mutants`, `coverage`, `sbom`
6. Remove `release` job (moved to ci-full.yml)

- [ ] **Step 1: Write the new `ci.yml`**

Target file: `.github/workflows/ci.yml`

Content:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  merge_group:

permissions:
  contents: read
  pull-requests: read

concurrency:
  group: ${{ github.workflow }}-${{ github.head_ref || github.run_id }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always

jobs:
  changes:
    name: Detect Changes
    runs-on: ubuntu-latest
    permissions:
      pull-requests: read
    outputs:
      rust: ${{ steps.filter.outputs.rust }}
    steps:
      - uses: actions/checkout@v4
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
              - '.github/workflows/ci-full.yml'

  lint:
    name: lint
    needs: changes
    if: needs.changes.outputs.rust == 'true' || github.event_name != 'pull_request'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --exclude rr-tauri -- -D warnings

  test:
    name: test
    needs: changes
    if: needs.changes.outputs.rust == 'true' || github.event_name != 'pull_request'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace --locked --verbose --exclude rr-tauri

  build-cli:
    name: build-cli
    needs: changes
    if: needs.changes.outputs.rust == 'true' || github.event_name != 'pull_request'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install cargo-auditable --locked
      - run: cargo auditable build --package rr-cli --release --locked
      - uses: actions/upload-artifact@v4
        with:
          name: rr-cli-linux
          path: target/release/rr
```

Note: `if: needs.changes.outputs.rust == 'true' || github.event_name != 'pull_request'` ensures jobs run on push to main AND on merge_group regardless of path changes. On PR, skip only if no Rust changed.

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: Tier 1 with concurrency, path filtering, merge_group trigger"

# Remove lint/test/build-cli, keep only the fast path.
# Full suite moves to ci-full.yml.
```

---

### Task 2: Create `ci-full.yml` — Tier 2 full suite (merge_group only)

**Files:**
- Create: `.github/workflows/ci-full.yml`

- [ ] **Step 1: Write `ci-full.yml`**

```yaml
name: Full Suite

on:
  merge_group:

permissions:
  contents: read

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: false

env:
  CARGO_TERM_COLOR: always

jobs:
  check-cross:
    name: check-cross (${{ matrix.os }})
    strategy:
      matrix:
        os: [macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --workspace --locked --exclude rr-tauri

  audit:
    name: audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install cargo-deny --locked
      - run: cargo deny check advisories bans licenses sources

  fuzz:
    name: fuzz (${{ matrix.target }})
    strategy:
      matrix:
        target:
          - fuzz_nip44_roundtrip
          - fuzz_nip44_decrypt
          - fuzz_identity_parse
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-fuzz
      - name: Restore corpus cache
        uses: actions/cache@v4
        with:
          path: crates/rr-core/fuzz/corpus/${{ matrix.target }}
          key: fuzz-corpus-${{ matrix.target }}-${{ github.sha }}
          restore-keys: |
            fuzz-corpus-${{ matrix.target }}-
      - run: cargo +nightly fuzz run --target $(rustc --print host-tuple) ${{ matrix.target }} -- -max_total_time=120
        working-directory: crates/rr-core
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: fuzz-artifacts-${{ matrix.target }}-${{ github.sha }}
          path: crates/rr-core/fuzz/artifacts/

  udeps:
    name: udeps
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-udeps
      - run: cargo +nightly udeps --workspace --exclude rr-tauri

  bench:
    name: bench
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Restore baseline cache
        uses: actions/cache@v4
        with:
          path: target/criterion
          key: bench-crypto-${{ github.ref_name }}
          restore-keys: bench-crypto-
      - name: Run benchmarks
        run: cargo bench --bench crypto 2>&1 | tee bench-output.txt
      - name: Check for regression (warning only)
        run: |
          if grep -q "Performance has regressed" bench-output.txt; then
            echo "⚠️ Performance change detected (baseline env may differ)"
            grep -B2 "regressed" bench-output.txt | head -6
          fi
          echo "✅ Benchmarks completed"

  coverage:
    name: coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-llvm-cov
      - run: mkdir -p coverage && cargo llvm-cov --workspace --exclude rr-tauri --lcov --output-path coverage/lcov.info
      - uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: coverage/

  mutants:
    name: mutants
    runs-on: ubuntu-latest
    timeout-minutes: 45
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install cargo-mutants --locked
      - run: cargo mutants --workspace --exclude rr-tauri || true
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: mutants-report
          path: mutants-out/

  sbom:
    name: sbom
    needs: [build-cli]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
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

Wait — `sbom` needs `build-cli` from `ci.yml`. But since `ci.yml` and `ci-full.yml` are separate workflows, they can't share `needs`. The artifact from `ci.yml` is uploaded, and `ci-full.yml`'s `sbom` job needs to download it. The artifact name is `rr-cli-linux` in both cases, but GitHub artifacts are scoped per workflow run. Different workflow = different artifact namespace.

Solution: either duplicate `build-cli` in `ci-full.yml`, or drop `sbom` from the merge queue workflow (it can run as a standalone scheduled job or on push to main).

Simplest fix: replicate `build-cli` in `ci-full.yml` so `sbom` can depend on it within the same workflow.

Let me fix the file:

```yaml
name: Full Suite

on:
  merge_group:

permissions:
  contents: read

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: false

env:
  CARGO_TERM_COLOR: always

jobs:
  build-cli:
    name: build-cli
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install cargo-auditable --locked
      - run: cargo auditable build --package rr-cli --release --locked
      - uses: actions/upload-artifact@v4
        with:
          name: rr-cli-linux
          path: target/release/rr

  check-cross:
    name: check-cross (${{ matrix.os }})
    strategy:
      matrix:
        os: [macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --workspace --locked --exclude rr-tauri

  audit:
    name: audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install cargo-deny --locked
      - run: cargo deny check advisories bans licenses sources

  fuzz:
    name: fuzz (${{ matrix.target }})
    strategy:
      matrix:
        target:
          - fuzz_nip44_roundtrip
          - fuzz_nip44_decrypt
          - fuzz_identity_parse
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-fuzz
      - name: Restore corpus cache
        uses: actions/cache@v4
        with:
          path: crates/rr-core/fuzz/corpus/${{ matrix.target }}
          key: fuzz-corpus-${{ matrix.target }}-${{ github.sha }}
          restore-keys: |
            fuzz-corpus-${{ matrix.target }}-
      - run: cargo +nightly fuzz run --target $(rustc --print host-tuple) ${{ matrix.target }} -- -max_total_time=120
        working-directory: crates/rr-core
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: fuzz-artifacts-${{ matrix.target }}-${{ github.sha }}
          path: crates/rr-core/fuzz/artifacts/

  udeps:
    name: udeps
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-udeps
      - run: cargo +nightly udeps --workspace --exclude rr-tauri

  bench:
    name: bench
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Restore baseline cache
        uses: actions/cache@v4
        with:
          path: target/criterion
          key: bench-crypto-${{ github.ref_name }}
          restore-keys: bench-crypto-
      - name: Run benchmarks
        run: cargo bench --bench crypto 2>&1 | tee bench-output.txt
      - name: Check for regression (warning only)
        run: |
          if grep -q "Performance has regressed" bench-output.txt; then
            echo "⚠️ Performance change detected (baseline env may differ)"
            grep -B2 "regressed" bench-output.txt | head -6
          fi
          echo "✅ Benchmarks completed"

  coverage:
    name: coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-llvm-cov
      - run: mkdir -p coverage && cargo llvm-cov --workspace --exclude rr-tauri --lcov --output-path coverage/lcov.info
      - uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: coverage/

  mutants:
    name: mutants
    runs-on: ubuntu-latest
    timeout-minutes: 45
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install cargo-mutants --locked
      - run: cargo mutants --workspace --exclude rr-tauri || true
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: mutants-report
          path: mutants-out/

  sbom:
    name: sbom
    needs: [build-cli]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
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
git add .github/workflows/ci-full.yml
git commit -m "ci: Tier 2 full suite on merge_group"
```

---

### Task 3: Create `ci-stub.yml` — Stub for PR-time required checks

**Files:**
- Create: `.github/workflows/ci-stub.yml`

This workflow runs on `pull_request_target` with `permissions: {}` (zero-risk). It emits stub jobs whose **names must match exactly** the required Tier 2 job names in `ci-full.yml`. Without this, the ruleset would show those checks as "pending" forever on PRs.

- [ ] **Step 1: Write `ci-stub.yml`**

```yaml
name: Full Suite (PR Stub)

on:
  pull_request_target:
    branches: [main]

permissions: {}

jobs:
  check-cross:
    name: check-cross (${{ matrix.os }})
    strategy:
      matrix:
        os: [macos-latest, windows-latest]
    runs-on: ubuntu-latest
    steps:
      - run: echo "stub — real check runs on merge_group"

  audit:
    name: audit
    runs-on: ubuntu-latest
    steps:
      - run: echo "stub — real check runs on merge_group"

  fuzz:
    name: fuzz (${{ matrix.target }})
    strategy:
      matrix:
        target:
          - fuzz_nip44_roundtrip
          - fuzz_nip44_decrypt
          - fuzz_identity_parse
    runs-on: ubuntu-latest
    steps:
      - run: echo "stub — real check runs on merge_group"

  udeps:
    name: udeps
    runs-on: ubuntu-latest
    steps:
      - run: echo "stub — real check runs on merge_group"
```

Note: job names must match `ci-full.yml` exactly:
- `check-cross (macos-latest)` → matrix `include` with `os: [macos-latest, windows-latest]`
- `check-cross (windows-latest)` ↲
- `audit`
- `fuzz (fuzz_nip44_roundtrip)` → matrix `include` with `target: [fuzz_nip44_roundtrip, ...]`
- `fuzz (fuzz_nip44_decrypt)` ↲
- `fuzz (fuzz_identity_parse)` ↲
- `udeps`

`bench`, `coverage`, `mutants`, `sbom` are not required — no stub needed.

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci-stub.yml
git commit -m "ci: stub workflow for PR-time required checks"
```

---

### Task 4: Update rulesets + enable merge queue on main

**Context:** The "Check Main" ruleset currently requires 8 status checks. We must:
1. Activate GitHub Merge Queue on `main` branch protection
2. Ensure the ruleset still requires the same 8 checks (stubs satisfy them on PR, real jobs on merge_group)

**Note:** This is done via GitHub UI / `gh` CLI — not in YAML.

- [ ] **Step 1: Enable merge queue on main**

Via GitHub UI:
1. Go to Settings → Branches → Branch protection rules for `main`
2. Enable "Require merge queue"
   - Merge method: **Squash**
   - Build concurrency: **5** (max parallel merge_group events)
   - Only merge non-failing pull requests: **true**
   - Status check timeout: **60 minutes**

Or via `gh` API:

```bash
# Get current branch protection for main
gh api repos/:owner/:repo/branches/main/protection

# Update to enable merge queue
gh api -X PUT repos/:owner/:repo/branches/main/protection \
  -F required_status_checks='{"strict":true,"checks":[{"context":"lint","app_id":0},{"context":"test","app_id":0},{"context":"audit","app_id":0},{"context":"check-cross (macos-latest)" ...}]}' \
  -F enforce_admins=true \
  -F required_pull_request_reviews='{"required_approving_review_count":1}' \
  --merge_queue=...
```

Actually, GitHub merge queue is configured in the branch protection rule settings page. The `gh` API may not support all merge queue settings. Prefer UI for this step.

- [ ] **Step 2: Verify Check Main ruleset still requires the 8 checks**

In Settings → Rules → Rulesets → "Check Main", confirm:
- Required checks: lint, test, audit, check-cross (macos-latest), check-cross (windows-latest), build-cli, udeps, fuzz (fuzz_nip44_roundtrip), fuzz (fuzz_nip44_decrypt), fuzz (fuzz_identity_parse)

If the fuzz matrix is configured as a single "fuzz" check (not 3 individual), adjust accordingly:
- Check: what exact names does the current CI produce for fuzz matrix jobs?
- The stub names must match exactly

- [ ] **Step 3: Add CODEOWNERS for .github/workflows/**

Create or update `.github/CODEOWNERS` to require review for workflow changes (prevents stub job name drift):

```
# Prevent CI stub job name drift — any change to workflow files
# must be reviewed to ensure stub job names match ci-full.yml
.github/workflows/ @giak
```

- [ ] **Step 4: Commit (only if UI changes)**

```bash
git add docs/superpowers/specs/2026-05-25-ci-optimization-merge-queue.md
git add .github/CODEOWNERS
git commit -m "docs: update CI optimization spec with merge queue settings
ci: add CODEOWNERS for .github/workflows/"

---

### Task 5: Test with canary PR

**Goal:** Validate the full flow before declaring done.

- [ ] **Step 1: Create a docs-only PR to test path filtering**

```bash
# Create a branch with only a doc change
rtk git checkout -b test/ci-docs-only
echo "# Test PR" > TEST_PR.md
rtk git add TEST_PR.md
rtk git commit -m "test: docs-only PR for CI validation"
rtk git push -u origin test/ci-docs-only
rtk gh pr create --title "test: CI docs-only path" --body "Validates that docs-only PRs skip Tier 1 and satisfy required checks via stubs"
```

Expected result:
- `ci.yml`: `changes` detects `rust != 'true'`, all jobs skip → workflow completes in ~30s
- `ci-stub.yml`: runs on `pull_request_target`, emits all stub job names → all 8 required checks ✅
- Total wait: <1 min

- [ ] **Step 2: Create a code PR to test Tier 1 + add to queue**

```bash
# Make a trivial code change
rtk git checkout main
rtk git checkout -b test/ci-code-change
# Add a comment or trivial change to a Rust file
# (use the existing branch or create a trivial change)
rtk git commit --allow-empty -m "test: empty commit to trigger code CI"
rtk git push -u origin test/ci-code-change
rtk gh pr create --title "test: CI code path" --body "Validates Tier 1 fires on Rust changes"
```

Expected result:
- `ci.yml`: `changes` detects `rust == 'true'`, all 3 jobs run → ~2-5 min
- `ci-stub.yml`: emits stubs → 8 checks ✅
- Add PR to merge queue → `ci-full.yml` runs all jobs on merge_group → merge auto if green

- [ ] **Step 3: Clean up test branches**

```bash
rtk git branch -D test/ci-docs-only test/ci-code-change
rtk git push origin --delete test/ci-docs-only test/ci-code-change
# Close test PRs without merging (or merge if successful)
```

---

## Self-Review Checklist

- [ ] **Spec coverage:** Every requirement from the spec maps to tasks:
  - Path filtering → Task 1
  - Tier 2 full suite → Task 2
  - Stub workflow → Task 3
  - Merge queue enablement → Task 4
  - Testing → Task 5
  - Cancel-in-progress → Task 1 (concurrency block)
  - Hotfix bypass → Task 4 (ruleset bypass unchanged, `--admin` still works)
- [ ] **Placeholder scan:** All code is concrete, all paths are exact, no TBDs
- [ ] **Name consistency:** Job names match across ci-full.yml (Task 2) and ci-stub.yml (Task 3): `check-cross (macos-latest)`, `check-cross (windows-latest)`, `audit`, `fuzz (fuzz_nip44_roundtrip)`, `fuzz (fuzz_nip44_decrypt)`, `fuzz (fuzz_identity_parse)`, `udeps`
- [ ] **Artifact dependency:** `sbom` needs `build-cli` — duplicated in `ci-full.yml` (Task 2) to keep within same workflow
