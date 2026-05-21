# RéseauRacine

> **Un réseau de communication souverain, résilient, et sécurisé.**

RéseauRacine est un réseau où chaque utilisateur possède son propre nœud. L'identité est une clé cryptographique portable qui fonctionne sur tous les transports simultanément : internet (Matrix/Nostr), Reticulum (WiFi/Ethernet/LoRa), et Meshtastic (LoRa texte seul). Le message est chiffré avec la clé du destinataire, quel que soit le transport.

Quand un transport tombe, le message en prend un autre compatible. La dégradation est automatique et transparente.

## Priorités

1. **Coordination sécurisée** — communication E2E pour cellules militantes
2. **Communication résiliente** — fonctionne quand internet est coupé
3. **Autonomie totale** — réseau parallèle indépendant d'internet
4. **Publication souveraine** — articles, vidéos, streams sans dépendance GAFAM

## Menace cible

Surveillance active + pression juridique (Viginum, réquisitions, infiltration).

## Architecture

5 couches :

```
Couche 0 — Identité (clé cryptographique Nostr/PGP)
Couche 1 — Messagerie E2E (X25519 + Ed25519)
Couche 2 — Transports (Internet / Reticulum / Meshtastic)
Couche 3 — Nœud local (Consommateur / Relais / Créateur)
Couche 4 — Gouvernance (Cellules → Essaims → Collège → RIC)
```

## 3 types de nœuds

| Type | Matériel | Coût | Rôle |
|------|----------|------|------|
| **Consommateur** | PC existant | 0 € | Consomme, distribue P2P |
| **Relais** | Pi 5 + LoRa + SSD | 150-280 € | Cache, relais Reticulum, identité 24/7 |
| **Créateur** | Mini PC 16 Go + SSD 1 To | 400-800 € | PeerTube, Owncast, publication |

## Résilience dégradée

| Mode | Transport | Contenu disponible |
|------|-----------|-------------------|
| **Normal** | Internet | Tout (articles, vidéos, streams, podcasts, messagerie) |
| **Dégradé** | Reticulum WiFi/Ethernet | Articles, cache local, messagerie |
| **Critique** | Reticulum LoRa | Texte court, messagerie |
| **Extrême** | Meshtastic | Texte seul, messagerie |

## Stack

| Composant | Technologie |
|-----------|------------|
| Identité | Nostr (Ed25519) + X25519 |
| Messagerie | Client custom (libsodium) |
| Transport internet | Nostr + Matrix |
| Transport local | Reticulum (WiFi/Ethernet/LoRa) |
| Transport off-grid | Meshtastic (LoRa texte seul) |
| Vidéos | PeerTube + WebTorrent |
| Streams | Owncast + WebTorrent |
| Articles | Nostr + IPFS |
| Podcasts | RSS + IPFS |
| Client | Tauri + React (UI) + Rust (core) |

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture spec](docs/superpowers/specs/2026-05-21-reseau-racine-architecture-design.md) | Spec complète — 5 couches, 3 nœuds, routage, messagerie, publication, gouvernance, faisabilité, roadmap |
| [EPIC POC](docs/superpowers/specs/2026-05-21-poc-premier-message-chiffr-epic.md) | POC fil rouge — "Premier Message Chiffré" (16h, CLI Rust + Nostr) |
| [Vision Dashboard](VISION_DASHBOARD.md) | Dashboard vision — architecture, stack, coûts, risques, métriques |
| [Brainstorm](brainstorm/) | Notes de brainstorm — architecture, synthèse Substack, questions ouvertes, étude technologique |

## Budget Phase 1

- **Matériel** : 2 700-5 200 € (10 relais + 3 créateurs + 100 consommateurs)
- **Annuel** : 800-2 400 €/an (électricité + maintenance)

## Feuille de route

| Phase | Durée | Objectif | Budget |
|-------|-------|----------|--------|
| **1 — MVP Coordination** | 0-3 mois | Messagerie E2E sur internet, cellules de 3 | 280 € |
| **2 — Nœuds Relais** | 3-6 mois | Pi 5 + Reticulum + cache, dégradation auto | 1 500-2 800 € |
| **3 — Publication** | 6-12 mois | PeerTube + Owncast + RSS + IPFS | 1 200-2 400 € |
| **4 — Résilience off-grid** | 12-18 mois | Reticulum LoRa + Meshtastic | 800-2 400 €/an |

## Licence

Ce projet est sous licence **AGPL-3.0**. Voir [LICENSE](LICENSE) pour le texte complet.

Toute modification, même déployée en réseau, doit être partagée sous la même licence.
