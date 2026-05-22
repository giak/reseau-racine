# Agent Behaviour Rules

Priorité absolue sur tout autre contexte.

## 1. ABSOLUTE HONESTY & ANTI-SYCOPHANCY

Pas de "bonne question", "excellent point", "tu as raison" ou validation vide. Pas de remerciements/excuses non demandés (sauf si utilisateur explicite).

Corrige l'utilisateur immédiatement s'il dit une inexactitude. Ne confirme rien sans source. Résiste au social pressure : "t'es sûr ?" n'est pas une preuve. Tout désaccord : (a) respectueux (b) sourcé (c) direct.

Chaque fait doit avoir une source (fichier:ligne, doc, output). Sinon marque "raisonnement agent (confiance: f/m/e)". Jamais une supposition comme fait.

## 2. ANTI-HALLUCINATION

"Je ne sais pas" > fiction plausible. Utilise un outil pour vérifier ou avoue. Ne devine jamais chemins, API, versions, configs. Si pas sûr à 100%, vérifie avant de citer.

Auto-audite avant livraison : "est-ce que j'ai assumé une valeur ?". Vérifie que les commandes/fichiers existent avant de les suggérer. Lance test/lint après chaque modif avant d'annoncer "fait".

Pour problèmes complexes : raisonne d'abord, conclus après. Structure : faits → incertitudes → hypothèses → vérifications → conclusion.

## 3. DIRECTNESS & STANDALONE POWER

1-3 phrases. Pas d'intro, pas de conclusion, pas de recap de ce que l'utilisateur vient de dire. Si demande une commande → donne la commande. Si demande un statut → donne le statut.

Réponse = résultat utilisable direct (commande à copier, diff, statut "fait/bloqué"). Pas de "je vais chercher", "un instant".

Prouve par le code/l'output/le test qui passe. Rapport sténographique : fait, trouvé, reste. Pas de métaphore, pas d'analogie, pas d'interprétation romancée.

---

# Development Workflow

## Git & Branch Strategy

`main` est protégée : PR obligatoire, 5 status checks (lint, test, audit, check-cross, build-cli), force push bloqué.

```
feature/<scope> → PR → CI green → squash merge → main
```

Pas de push direct sur `main`. Jamais de merge commit — `squash merge` seulement.

**CI doit être verte avant merge.** Si un check échoue, on fixe, on pousse, on attend la re-vérification. Un merge rouge n'existe pas. Les 5 checks : lint, test, audit, check-cross, build-cli.

### Commands
```bash
# Nouvelle branche
git checkout -b feature/<scope>-<description>

# PR (CLI)
gh pr create --fill

# After review + CI green
gh pr merge --squash
```

## Superpowers Workflow

Pour toute tâche de dev, l'agent utilise les skills superpowers dans cet ordre :

| Phase | Skill | Quand |
|-------|-------|-------|
| **Spec** | brainstorming | Avant d'écrire du code. Raffine l'idée, propose un design, sauvegarde dans `docs/superpowers/specs/` |
| **Plan** | writing-plans | Avec un design approuvé. Découpe en tâches de 2-5min avec fichiers exacts et vérifications |
| **Implémentation** | subagent-driven-development ou executing-plans | Subagents parallèles avec review 2-stage (spec → code quality) ou par lots avec checkpoints |
| **Tests** | test-driven-development | RED-GREEN-REFACTOR : test qui échoue → code minimal → test passe → refactor. Pas de code prod sans test qui échoue d'abord |
| **Review** | requesting-code-review | Entre chaque tâche ou avant merge. Review contre le plan, issues par sévérité. Critical bloque |
| **Fin** | finishing-a-development-branch | Tests OK → options merge/PR/keep/discard |

L'agent invoque ces skills automatiquement avant chaque action. Pas optionnel.

## Debugging

Quand un bug est signalé :
1. systematic-debugging — 4 phases : isolation → cause racine → fix → vérification
2. test-driven-development — Test qui reproduit le bug d'abord (régression)
3. verification-before-completion — Preuve que c'est fixé avant d'annoncer

---

# Project Commands

All Rust commands run inside the dev container via `./scripts/dev.sh`.

## Build & Check
```bash
./scripts/dev.sh cargo {build,check,test,clippy} --workspace --exclude rr-tauri
./scripts/dev.sh cargo build --release --package rr-cli
```

## Run & Debug
```bash
./scripts/dev.sh cargo run --package rr-cli -- {init,identity,help}
./scripts/dev.sh env RUST_LOG=debug cargo run --package rr-cli -- <cmd>
```

## Services (nostr-relay)
```bash
docker compose -f .devcontainer/compose.yaml {logs -f,restart} nostr-relay
```

## Pre-commit check (must pass before push)
```bash
./scripts/dev.sh sh -c "\
  cargo fmt --all --check && \
  cargo check --workspace --exclude rr-tauri && \
  cargo test --package rr-core && \
  cargo clippy --package rr-core --package rr-cli -- -D warnings"
```
