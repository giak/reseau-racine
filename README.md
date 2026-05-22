# RéseauRacine

Messagerie pair-à-pair chiffrée de bout en bout, basée sur le protocole Nostr (NIP-17). Racines françaises, réseau mondial.

## Architecture

```
reseau-racine/
├── crates/
│   ├── rr-core/        # Bibliothèque centrale : crypto, identité, messages, transport
│   ├── rr-cli/         # Client CLI (POC)
│   └── rr-tauri/       # Application de bureau (Tauri v2)
├── scripts/            # Utilitaires de build et check
└── .github/workflows/  # CI/CD
```

## Prérequis

- [Rust](https://rustup.rs/) 1.85+
- [just](https://github.com/casey/just) (optionnel, via `make` alternatif)

## Démarrage rapide

```bash
cargo build --release
./target/release/rr init        # Créer une identité
./target/release/rr identity    # Voir sa clé publique
./target/release/rr help        # Voir les commandes
```

## Commandes CLI

| Commande | Description |
|----------|-------------|
| `rr init` | Créer une nouvelle identité Nostr |
| `rr identity` | Afficher sa clé publique (npub) |
| `rr add-contact <npub> <nom>` | Ajouter un contact |
| `rr contacts` | Lister ses contacts |
| `rr restore <seed-phrase>` | Restaurer une identité depuis une seed phrase de 12 mots |
| `rr send <nom> <message>` | Envoyer un message (EPIC 1) |
| `rr sync` | Synchroniser les messages (EPIC 1) |

## Tests

```bash
cargo test --workspace
```

## Feuille de route

- **Phase 0** — Fondations : crypto, identité, transport Nostr ✓
- **EPIC 1** — Messagerie P2P : envoi/sync via NIP-17
- **EPIC 2** — Contacts et discovery
- **EPIC 3** — Groupes et salons
- **EPIC 4** — Interface graphique (Tauri)
- **EPIC 5** — Version mobile

## Licence

AGPL-3.0-or-later — voir [LICENSE](LICENSE).
