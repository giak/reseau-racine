# Spec — RéseauRacine : Architecture Multi-Transport

> **Un réseau de communication souverain, résilient, et sécurisé.**
> Priorités : coordination sécurisée → communication résiliente → autonomie totale → publication souveraine.
> Menace cible : surveillance active + pression juridique (Viginum, réquisitions, infiltration).
> Déploiement : local d'abord (ville par ville), expansion organique.
> Adhésion : hybride (ouverte pour fonctions basiques, cooptation pour cercles sensibles).

---

## §0 Résumé exécutif

RéseauRacine est un réseau de communication où chaque utilisateur possède son propre nœud. L'identité est une clé cryptographique portable qui fonctionne sur tous les transports simultanément : internet (Matrix/Nostr), Reticulum (WiFi/Ethernet/LoRa), et Meshtastic (LoRa texte seul). Le message est chiffré avec la clé du destinataire, quel que soit le transport. Le transport est un détail — l'identité et le chiffrement sont constants.

Quand un transport tombe, le message en prend un autre compatible. La dégradation est automatique et transparente.

---

## §1 Architecture en 5 couches

```
┌─────────────────────────────────────────────────────────┐
│                    COUCHE 0 — IDENTITÉ                   │
│  Clé cryptographique unique (Nostr/PGP-like)            │
│  → Identité portable, indépendante du transport         │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│              COUCHE 1 — MESSAGERIE E2E                   │
│  Chiffrement avec clé du destinataire (X25519+AES)      │
│  → Le message est chiffré AVANT d'entrer dans un transport│
└──────────────────────┬──────────────────────────────────┘
                       │
         ┌─────────────┼─────────────┐
         ▼             ▼             ▼
┌────────────┐ ┌────────────┐ ┌────────────┐
│ COUCHE 2A  │ │ COUCHE 2B  │ │ COUCHE 2C  │
│ Internet   │ │ Reticulum  │ │ Meshtastic │
│ Matrix     │ │ WiFi/Eth   │ │ LoRa       │
│ Nostr      │ │ LoRa       │ │ Texte seul │
│            │ │ I2P        │ │ GPS        │
└─────┬──────┘ └─────┬──────┘ └─────┬──────┘
      │              │              │
      ▼              ▼              ▼
┌─────────────────────────────────────────────────────────┐
│              COUCHE 3 — NŒUD LOCAL (3 variantes)         │
│  Consommateur (PC existant)                              │
│  Relais (Raspberry Pi 5 + LoRa)                          │
│  Créateur (Mini PC + PeerTube + Owncast)                 │
│  → Héberge l'identité, le cache, le routing             │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│              COUCHE 4 — GOUVERNANCE                      │
│  Cellules de 3 → Essaims de 10 → Collège → RIC          │
│  → Gouvernance distribuée, modération par réputation    │
└─────────────────────────────────────────────────────────┘
```

**Principe central** : le message est chiffré au niveau de l'identité (Couche 1). Le transport (Couche 2) n'est qu'un tuyau. Si un tuyau se bouche, le message prend un autre tuyau. L'identité et le chiffrement ne changent jamais.

---

## §2 Les 3 types de nœuds

### Nœud Consommateur

| Élément | Détail |
|---------|--------|
| **Matériel** | PC/Mac existant de l'utilisateur |
| **Logiciel** | Client léger (navigateur ou app native) |
| **Rôle** | Consomme du contenu, participe à la distribution P2P (WebTorrent/IPFS) |
| **Identité** | Clé cryptographique stockée localement |
| **Coût** | 0 € (matériel existant) |
| **Barrière d'entrée** | Aucune — s'installe en 5 minutes |

### Nœud Relais

| Élément | Détail |
|---------|--------|
| **Matériel** | Raspberry Pi 5 (8 Go) + module LoRa RNode + SSD 256 Go |
| **Logiciel** | Reticulum + IPFS pinning + relais Nostr + cache local |
| **Rôle** | Stocke/cache du contenu, sert de relais Reticulum, héberge l'identité |
| **Identité** | Clé cryptographique sur le nœud, toujours actif |
| **Coût** | 150-280 € |
| **Barrière d'entrée** | Moyenne — nécessite configuration initiale |

### Nœud Créateur

| Élément | Détail |
|---------|--------|
| **Matériel** | Mini PC (Intel NUC / Ryzen mini) 16 Go RAM + SSD 1 To + GPU optionnel |
| **Logiciel** | PeerTube + Owncast + Reticulum + IPFS + relais Nostr |
| **Rôle** | Crée du contenu (vidéo, stream, articles), héberge son instance |
| **Identité** | Clé cryptographique + identité publique (journaliste, média) |
| **Coût** | 400-800 € |
| **Barrière d'entrée** | Haute — nécessite compétences techniques |

### Tableau récapitulatif

| Fonctionnalité | Consommateur | Relais | Créateur |
|---------------|:---:|:---:|:---:|
| Lire articles | ✅ | ✅ | ✅ |
| Regarder vidéos | ✅ | ✅ | ✅ |
| Écouter podcasts | ✅ | ✅ | ✅ |
| Messagerie E2E | ✅ | ✅ | ✅ |
| Distribution P2P (WebTorrent/IPFS) | ✅ | ✅ | ✅ |
| Cache local (voisins) | — | ✅ | ✅ |
| Relais Reticulum (WiFi/LoRa) | — | ✅ | ✅ |
| Relais Nostr | — | ✅ | ✅ |
| PeerTube (hébergement vidéo) | — | — | ✅ |
| Stream live (Owncast) | — | — | ✅ |
| Publication Nostr signée | — | — | ✅ |
| Feed RSS | — | — | ✅ |

---

## §3 Couche Identité

### Principe

L'identité est une **paire de clés secp256k1** (clé privée + clé publique), standard Nostr.

- **Clé privée** (`nsec`) : signe les messages, prouve "c'est moi". Ne quitte jamais le nœud.
- **Clé publique** (`npub`) : vérifie les signatures, identifiant unique (hex 64 caractères). Partagée publiquement.

**Propriétés** : portable, indépendante du transport, non-supprimable, vérifiable.

### Format

- **Signature** : secp256k1 (standard Nostr) — écosystème existant, outils matures
- **Chiffrement** : X25519 dérivé de secp256k1 (NIP-44) — ChaCha20-Poly1305 AEAD

> **Note** : Nostr utilise secp256k1 pour les signatures (pas Ed25519). Pour le chiffrement E2E, la clé X25519 est dérivée de la clé secp256k1 via NIP-44. C'est le standard actuel (NIP-17 + NIP-44 + NIP-59), qui remplace NIP-04 (déprécié, AES-CBC non authentifié, CVE-2026-41301).

### Fonctionnement

```
1. L'utilisateur génère sa paire de clés secp256k1 (1 clic)
2. La clé publique (npub, hex dérivé de secp256k1) est son identité
3. Pour envoyer un message à Bob :
   a. Le message est chiffré avec NIP-44 (X25519 dérivé + ChaCha20-Poly1305)
   b. Le message chiffré est signé avec secp256k1 (clé privée d'Alice)
   c. Le paquet est enveloppé (NIP-59 gift wrap) pour cacher les métadonnées
   d. Le gift wrap est envoyé via TOUS les transports compatibles avec le type de contenu
4. Bob reçoit le message (par le premier transport qui arrive)
5. Bob unwrap le gift wrap, vérifie la signature secp256k1 et déchiffre
```

### Gestion de la clé privée

| Méthode | Sécurité | UX | Recommandé pour |
|---------|----------|----|----------------|
| **Fichier local chiffré** (NIP-49) | Moyenne | Simple | Nœud consommateur |
| **Hardware wallet** (Nitrokey, YubiKey) | Haute | Moyenne | Nœud relais/créateur |
| **Seed phrase** (12 mots, NIP-06) | Haute | Complexe | Backup |

### Réputation liée à l'identité

- **Signature vérifiable** : chaque message est signé avec secp256k1 → impossible de falsifier l'auteur
- **Historique portable** : l'historique suit la clé, pas le transport
- **Réputation distribuée** : score basé sur les interactions vérifiables
- **Web of trust** : "Je fais confiance à Alice" → si Alice fait confiance à Bob, je peux faire confiance à Bob

---

## §4 Routage multi-transport

### Principe

Le routeur utilise TOUS les transports disponibles simultanément, avec une logique de priorité et de fallback automatique.

### Logique de décision

| Critère | Poids | Détail |
|---------|-------|--------|
| **Disponibilité** | Prioritaire | Le transport est-il actif ? |
| **Type de contenu** | Élevé | Texte court → tous. Vidéo → internet seul. |
| **Urgence** | Élevé | Message critique → envoyer sur TOUS les transports |
| **Bande passante** | Moyen | Le transport a-t-il assez de débit ? |
| **Latence** | Moyen | Le transport est-il rapide ? |

### Matrice de routage

| Contenu | Mode Normal | Mode Dégradé | Mode Critique | Mode Extrême |
|---------|------------|-------------|--------------|-------------|
| **Message urgent** | Internet + Reticulum + Meshtastic | Reticulum + Meshtastic | Reticulum LoRa | Meshtastic |
| **Message normal** | Internet | Reticulum WiFi | Reticulum LoRa | — |
| **Article** | Internet (Nostr + IPFS) | Reticulum WiFi | Reticulum LoRa (texte) | — |
| **Vidéo** | Internet (PeerTube + WebTorrent) | Cache local WiFi | — | — |
| **Stream live** | Internet (Owncast + WebTorrent) | — | — | — |
| **Podcast** | Internet (RSS + IPFS) | Cache local WiFi | — | — |
| **Fichier** | Internet (IPFS) | Reticulum WiFi | — | — |

### Détection automatique de dégradation

Le routeur surveille chaque transport en continu (ping toutes les 30s). Temps de détection : 30-60 secondes.

### Découverte de pairs (Peer Discovery)

| Transport | Mécanisme | Détail |
|-----------|-----------|--------|
| **Internet** | Relais Nostr + serveurs Matrix | Les pairs sont découverts via les relais publics ou privés |
| **Reticulum WiFi/Ethernet** | Announce packets (broadcast UDP) | Découverte automatique sur le réseau local |
| **Reticulum LoRa** | Beacon périodique sur fréquence partagée | Chaque nœud diffuse son existence toutes les 60s |
| **Meshtastic** | NodeDB + position GPS partagée | Découverte automatique via le protocole Meshtastic |

**Bootstrap initial** : un nouvel utilisateur configure manuellement l'adresse d'au moins 1 nœud relais connu (QR code, lien, saisie manuelle). Après ça, la découverte est automatique.

```
Internet actif ? → OUI = Mode Normal
    │
    └─ NON → Reticulum WiFi actif ? → OUI = Mode Dégradé
                 │
                 └─ NON → Reticulum LoRa actif ? → OUI = Mode Critique
                              │
                              └─ NON → Meshtastic actif ? → OUI = Mode Extrême
                                           │
                                           └─ NON → Mode Hors-ligne (cache local seul)
```

### Gestion des doublons

Chaque message a un ID unique (hash du contenu + timestamp + signature). Le client deduplique automatiquement — il garde la version la plus complète (ex: internet avec fichier > LoRa texte seul). Si la première réception est une version dégradée, elle est remplacée quand la version complète arrive.

### Synchronisation post-dégradation

Quand internet revient :
1. Le nœud détecte qu'internet est de nouveau actif
2. Il synchronise tous les messages accumulés en mode dégradé/critique
3. Les messages sont publiés sur Nostr/Matrix avec leur timestamp de création original (champ `created_at` dans le header, pas le timestamp de publication). En cas de conflit (message modifié pendant la coupure), la version avec le `created_at` le plus récent prevaut.
4. Les fichiers lourds (vidéos, podcasts) sont uploadés en arrière-plan
5. **Gestion de conflits** : si un message a été modifié ou supprimé pendant la coupure, la version avec le timestamp le plus récent prevaut. Les suppressions sont répliquées (tombstone avec signature).

---

## §5 Messagerie E2E (Coordination sécurisée — Priorité 1)

### Structure du message (NIP-17)

Le message passe par 3 couches (NIP-17) :

```
Couche 1 — Rumor (kind 14 — PrivateDirectMessage)
  Header:
    - kind: 14
    - created_at: timestamp
    - tags: [["p", "npub_destinataire"], ["e", "event_id_réponse"] (optionnel)]
  Payload:
    - Contenu (texte, fichier, coordonnée GPS)
    - TTL (NIP-40 expiration timestamp)

Couche 2 — Seal (kind 13 — signé avec clé éphémère)
  - Le rumor est signé avec une clé secp256k1 éphémère (pas la clé principale)
  - Cache l'identité de l'expéditeur aux relais

Couche 3 — Gift Wrap (kind 1059 — chiffré NIP-44 pour le destinataire)
  - Le seal est chiffré avec NIP-44 (X25519 + ChaCha20-Poly1305)
  - Seule métadonnée visible : ["p", "npub_destinataire"]
  - Le relais ne voit pas l'expéditeur, ni le contenu
```

> **Pourquoi 3 couches** : NIP-04 (kind 4, déprécié) exposait les métadonnées (qui envoie à qui). NIP-17 résout ça avec le gift wrap (NIP-59) qui cache l'expéditeur, et NIP-44 qui remplace AES-CBC par ChaCha20-Poly1305 AEAD.

### Types de conversations

| Type | Description | Chiffrement | Usage |
|------|-------------|-------------|-------|
| **1:1** | Conversation privée entre 2 personnes | NIP-44 (X25519 + ChaCha20-Poly1305) | Communication sensible, sources |
| **Cellule (3)** | Groupe fermé de 3 personnes | NIP-44 + clé de groupe X25519 | Coordination d'action directe |
| **Essaim (10)** | Groupe de 10 cellules | NIP-44 + clé de groupe X25519 | Coordination régionale |
| **Broadcast** | Message public signé | Signature secp256k1 seule | Publication, annonces |
| **Canal** | Canal thématique ouvert | Signature secp256k1 | Discussion communautaire |

### Fonctionnalités de sécurité

| Fonctionnalité | Comment | Pourquoi |
|---------------|---------|----------|
| **Forward secrecy** | Clés de session éphémères (ratchet) | Si une clé est compromise, les messages passés restent protégés |
| **Suppression sécurisée** | Les messages supprimés sont écrasés (overwrite 3 passes) | Pas de preuve récupérable par saisie de matériel |
| **Auto-destruction** | TTL configurable (1h, 24h, 7j, 30j) | Les messages sensibles expirent automatiquement |
| **Pas de métadonnées** | Pas de "vu à", pas de "en ligne", pas de typing indicator | Réduit la surface d'attaque informationnelle |
| **Vérification d'identité** | Comparaison de fingerprint (QR code ou 12 mots) | Prévention des attaques MITM |
| **Alerte de compromission** | Notification si la signature d'un contact change | Détection d'infiltration |

### Protection contre l'infiltration

| Menace | Contre-mesure |
|--------|--------------|
| **Infiltration Viginum** | Web of trust : chaque membre d'une cellule doit être vérifié en personne ou par parrainage croisé |
| **Faux compte** | Signature secp256k1 obligatoire + vérification de fingerprint |
| **Capture de nœud** | Forward secrecy + auto-destruction + NIP-44 AEAD = les messages capturés sont illisibles et non falsifiables |
| **Analyse de trafic** | Reticulum n'a pas d'adresses source + padding des messages (uniquement sur transport Reticulum). Sur internet (Matrix/Nostr), les métadonnées sont visibles par les serveurs — le contenu reste chiffré E2E mais pas l'anonymat. |
| **Social engineering** | Cellules de 3 : un infiltré ne connaît que 2 personnes, pas l'essaim |
| **Spam** | Rate limiting : Niveau 0 = 10 messages/min, Niveau 1 = 50 messages/min, Niveau 2+ = illimité. Les messages en excès sont rejetés localement. |

### Stockage local

| Élément | Stockage | Chiffré ? |
|---------|----------|-----------|
| Messages actifs | SQLite local | Oui (SQLCipher) |
| Messages expirés | Supprimés (overwrite) | — |
| Clés privées (secp256k1) | Fichier chiffré (NIP-49) ou hardware wallet | Oui (AES-256-GCM) |
| Cache contenu | IPFS local | Non (contenu public) |
| Contacts | SQLite local | Oui |

### Protocole de groupe

1. **Création** : le créateur génère une clé de groupe X25519 (dérivée de secp256k1)
2. **Invitation** : chaque membre reçoit la clé de groupe chiffrée avec NIP-44 (sa clé publique individuelle)
3. **Communication** : les messages de groupe sont chiffrés avec NIP-44 (clé de groupe X25519 + ChaCha20-Poly1305)
4. **Départ** : si un membre quitte, la clé de groupe est regenerée et redistribuée (post-compromise security)
5. **Exclusion** : un membre exclu ne peut plus déchiffrer les nouveaux messages

---

## §6 Publication de contenu

### Articles

| Élément | Détail |
|---------|--------|
| **Format** | Markdown + signature Nostr (nip-01) |
| **Distribution** | Nostr relays + IPFS (CID) |
| **Vérification** | Signature secp256k1 = l'auteur est vérifiable cryptographiquement |
| **Résilience** | Disponible sur TOUS les modes (même LoRa pour les articles courts) |
| **Censure** | Impossible à supprimer (IPFS + Nostr distribué) |

### Vidéos

| Élément | Détail |
|---------|--------|
| **Encodage** | Pré-transcodage local (avant upload) — H.264/H.265, multiples qualités |
| **Hébergement** | PeerTube local sur le nœud créateur |
| **Distribution** | WebTorrent (P2P entre viewers) + IPFS (mirror) |
| **Fédération** | ActivityPub (les instances PeerTube se synchronisent) |
| **Résilience** | Mode normal uniquement. Mode dégradé = cache local WiFi |

**Note** : un Raspberry Pi 5 ne peut pas transcoder en temps réel. L'auteur transcode sur son PC **avant** d'uploader. Le Pi ne sert que les fichiers déjà encodés.

### Streams live

| Élément | Détail |
|---------|--------|
| **Technologie** | Owncast (self-hosted, RTMP in → HLS out) |
| **Distribution** | WebTorrent P2P (les viewers partagent les segments) |
| **Matériel** | Mini PC (pas un Pi) — 8 Go RAM minimum |
| **Résilience** | Mode normal uniquement. Pas de fallback. |

**Limitation** : un stream live nécessite internet. Si internet tombe, le stream s'arrête. Le contenu peut être enregistré et redistribué en VOD après coup.

### Podcasts audio

| Élément | Détail |
|---------|--------|
| **Format** | MP3/Opus + feed RSS |
| **Distribution** | RSS + IPFS |
| **Résilience** | Mode normal + mode dégradé (cache local WiFi) |
| **Taille** | 30-100 Mo par épisode — transférable via Reticulum WiFi en quelques minutes |

### Tableau récapitulatif par mode

| Contenu | Normal (Internet) | Dégradé (Reticulum WiFi) | Critique (LoRa) | Extrême (Meshtastic) |
|---------|------------------|-------------------------|-----------------|---------------------|
| Articles | ✅ Nostr + IPFS | ✅ Cache local | ✅ Texte court | ✅ Texte court |
| Vidéos | ✅ PeerTube + WebTorrent | ⚠️ Cache local (pré-chargé) | ❌ | ❌ |
| Streams | ✅ Owncast + WebTorrent | ❌ | ❌ | ❌ |
| Podcasts | ✅ RSS + IPFS | ⚠️ Cache local | ❌ | ❌ |
| Messagerie | ✅ E2E multi-transport | ✅ E2E Reticulum | ✅ Texte seul | ✅ Texte seul |

---

## §7 Gouvernance et modération

### Structure

| Niveau | Structure | Rôle | Taille |
|--------|-----------|------|--------|
| **Cellule** | 3 personnes | Coordination locale, prise de décision rapide | 3 |
| **Essaim** | 10 cellules | Coordination régionale, arbitrage | 30 |
| **Collège** | 7-15 membres élus | Gouvernance globale, modification de la Charte | 7-15 |
| **Réseau** | Tous les nœuds | Vote sur les modifications majeures (RIC distribué) | Illimité |

### Mécanisme de décision

| Type de décision | Qui décide | Quorum | Majorité |
|-----------------|-----------|--------|----------|
| **Modification mineure** (documentation, paramètres) | Collège | 50 % | 66 % |
| **Modification majeure** (protocole, Charte) | RIC distribué | 30 % des nœuds actifs | 66 % |
| **Exclusion d'un nœud** | Essaim + appel au Collège | — | Unanimité essaim |
| **Budget** | Collège + transparence publique | — | 66 % |

### Modération distribuée

| Mécanisme | Comment |
|-----------|---------|
| **Signature obligatoire** | Chaque message est signé → pas d'anonymat total pour les messages publics |
| **Signalement** | Tout utilisateur peut signaler un contenu. Vérifié par 3 pairs aléatoires (cellule) |
| **Décision** | Si 2/3 confirment, le contenu est marqué (pas supprimé) — réduit en visibilité |
| **Appel** | Jugé par un essaim de 10 pairs |
| **Transparence** | Toutes les décisions de modération sont publiques (sauf données personnelles) |

### Web of Trust pour l'adhésion

```
Niveau 0 — Inscrit ouvert
  │
  ├─ Peut lire les contenus publics
  ├─ Peut envoyer des messages 1:1
  ├─ Peut publier des messages broadcast (signés, publics)
  ├─ Ne peut PAS rejoindre de cellules
  ├─ Ne peut PAS créer de canaux
  └─ Identité: clé cryptographique seule
  │
  ▼
Niveau 1 — Vérifié (1 parrain)
  │
  ├─ Peut rejoindre des cellules (sur invitation)
  ├─ Peut participer aux canaux
  ├─ Peut créer des canaux
  └─ Identité: parrainé par un membre Niveau 2+
  │
  ▼
Niveau 2 — Fiable (2 parrains croisés)
  │
  ├─ Peut créer des cellules
  ├─ Peut héberger un nœud relais
  └─ Identité: vérifié par 2 membres indépendants
  │
  ▼
Niveau 3 — Collège (élection)
  │
  ├─ Peut participer à la gouvernance
  ├─ Peut modifier la Charte (via RIC)
  └─ Identité: élu par le réseau
```

### Protection contre la capture

| Risque | Contre-mesure |
|--------|--------------|
| **Capture financière** | Plafond 5 % par contributeur au budget |
| **Capture par infiltration** | Cellules de 3 : un infiltré ne connaît que 2 personnes |
| **Capture par fork** | Forks autorisés (structure gazeuse) |
| **Capture juridique** | Multi-juridictions (France + Suisse + Islande) |
| **Capture technique** | Code open-source, auditabilité totale |

---

## §8 Analyse de faisabilité

### Faisabilité par couche

| Couche | Faisabilité | Risque | Détail |
|--------|------------|--------|--------|
| **Identité (Nostr secp256k1)** | ✅ Haute | Faible | Nostr existe, secp256k1 standard. Client à construire. |
| **Messagerie E2E** | ✅ Haute | Faible | NIP-17 + NIP-44 + NIP-59 = standard actuel. Crate `nostr` v0.44.2 supporte nativement. |
| **Routage multi-transport** | ⚠️ Moyenne | Moyen | Reticulum gère le multi-transport nativement. Bridge internet à construire. |
| **Nœud Consommateur** | ✅ Haute | Faible | App légère + WebTorrent + IPFS desktop. Existe déjà. |
| **Nœud Relais** | ✅ Haute | Faible | Reticulum + IPFS + Nostr relay sur Pi 5. Faisable. |
| **Nœud Créateur** | ⚠️ Moyenne | Moyen | PeerTube + Owncast sur mini PC. Faisable mais configuration nécessaire. |
| **Publication vidéo P2P** | ✅ Haute | Faible | PeerTube utilise déjà WebTorrent. |
| **Stream live P2P** | ⚠️ Moyenne | Moyen | Owncast + WebTorrent = faisable mais latence 10-30s. |
| **Dégradation auto** | ⚠️ Moyenne | Moyen | Détection facile. Bascule transparente = plus complexe. |
| **Gouvernance distribuée** | ✅ Haute | Faible | Loomio/CryptPad existent. RIC distribué = vote sur Nostr. |

### Alternatives et remplacements dégradés

| Composant | Primaire | Alternative | Remplacement dégradé |
|-----------|----------|-------------|---------------------|
| **Identité** | Nostr (nsec/npub) | PGP | Seed phrase 12 mots (backup) |
| **Messagerie 1:1** | NIP-17 (NIP-44 + NIP-59) | Signal | Reticulum Sideband |
| **Messagerie groupe** | NIP-44 + clé de groupe X25519 | Matrix | Reticulum Sideband |
| **Articles** | Nostr + IPFS | WriteFreely | Reticulum NomadNet |
| **Vidéos** | PeerTube + WebTorrent | IPFS seul | Cache local WiFi |
| **Streams** | Owncast + WebTorrent | PeerTube live | Enregistré → VOD |
| **Podcasts** | RSS + IPFS | RSS seul | Cache local |
| **Transport internet** | Matrix + Nostr | Email chiffré | — |
| **Transport local** | Reticulum | Yggdrasil | WiFi ad-hoc |
| **Transport off-grid** | Reticulum LoRa | Meshtastic | — |
| **Transport extrême** | Meshtastic | Packet radio (AX.25) | FM (illégal) |

### Coûts réels

| Type | Matériel | Coût | Coût annuel |
|------|----------|------|------------|
| **Consommateur** | PC existant | 0 € | 0 € |
| **Relais** | Pi 5 + LoRa + SSD | 150-280 € | 50-150 € (électricité) |
| **Créateur** | Mini PC 16 Go + SSD 1 To | 400-800 € | 100-300 € (électricité + bande passante) |

**Budget Phase 1 (10 relais + 3 créateurs + 100 consommateurs)** :
- Matériel : 10 × 150-280 € + 3 × 400-800 € = **2 700-5 200 €**
- Annuel : 10 × 50-150 € + 3 × 100-300 € = **800-2 400 €/an**

---

## §9 Spécifications techniques du client

### Architecture du client

```
┌─────────────────────────────────────────────────────┐
│                    CLIENT UNIFIÉ                     │
│                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │   UI Layer  │  │  Core Logic │  │  Transport  │ │
│  │  (React/    │  │  (Rust)     │  │  Adapters   │ │
│  │   Tauri)    │  │             │  │             │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘ │
│         │                │                │         │
│  ┌──────▼────────────────▼────────────────▼──────┐ │
│  │              Identity Manager                  │ │
│  │  - Key generation/storage                      │ │
│  │  - Sign/verify                                 │ │
│  │  - Encrypt/decrypt                             │ │
│  └──────────────────────┬────────────────────────┘ │
│                         │                           │
│  ┌──────────────────────▼────────────────────────┐ │
│  │              Message Router                    │ │
│  │  - Multi-transport dispatch                    │ │
│  │  - Deduplication                               │ │
│  │  - Priority/urgency handling                   │ │
│  │  - Degradation detection                       │ │
│  └───────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

### Technologies proposées

| Composant | Technologie | Pourquoi |
|-----------|------------|----------|
| **UI** | Tauri + React | Léger, cross-platform (Windows/macOS/Linux), bundle < 10 Mo |
| **Core** | Rust | Performance, sécurité mémoire, pure Rust crypto (pas de FFI C) |
| **Crypto** | Crate `nostr` v0.44.2 (secp256k1 + NIP-44 ChaCha20-Poly1305 + NIP-59) | Support natif NIP-17, crates pures Rust auditables |
| **Crypto (custom)** | `x25519-dalek` + `chacha20poly1305` + `ed25519-dalek` | Si besoin de crypto hors Nostr (Reticulum, groupes custom) |
| **Storage** | SQLite + SQLCipher | Chiffré au repos, léger |
| **P2P** | libp2p (IPFS) + WebTorrent | Mature, écosystème large |
| **Reticulum** | pythonreticulum (subprocess) ou reticulum-rs | Stack réseau off-grid |
| **Nostr** | Crate `nostr` v0.44.2 + tokio-tungstenite | Écosystème existant, async WebSocket |

### Interfaces

| Interface | Description |
|-----------|-------------|
| `IdentityManager` | Génère, stocke, utilise les clés. Signe, vérifie, chiffre, déchiffre. |
| `MessageRouter` | Reçoit un message chiffré, choisit le(s) transport(s), gère la déduplication. |
| `TransportAdapter` | Interface commune pour chaque transport (send, receive, status, bandwidth). |
| `ContentCache` | Gère le cache local (articles, vidéos, podcasts). TTL, eviction, pinning. |
| `GroupManager` | Crée, gère les clés de groupe. Invitation, exclusion, rotation de clé. |

---

## §10 Risques et mitigations

| Risque | Probabilité | Impact | Mitigation |
|--------|------------|--------|------------|
| **Utilisation de NIP-04 (déprécié) par erreur** | Moyenne | Critique | Utiliser NIP-17 + NIP-44 + NIP-59. Crate `nostr` v0.44.2 supporte nativement. |
| **Bridge internet ↔ Reticulum non fonctionnel** | Moyenne | Élevé | Commencer avec internet seul, ajouter Reticulum en Phase 2 |
| **UX trop complexe pour le grand public** | Haute | Critique | Client Tauri simple, installation en 1 clic, documentation visuelle |
| **Adoption insuffisante** | Haute | Critique | Cibler d'abord les cellules militantes (besoin réel), pas le grand public |
| **Reticulum abandonné (1 dev)** | Moyenne | Élevé | Fork communautaire, implémentations Rust/Go en cours |
| **Infiltration Viginum** | Haute | Élevé | Web of trust + cellules de 3 + forward secrecy |
| **Coût réel > budget** | Moyenne | Moyen | Budget Phase 1 = 5 200 € (réaliste). Ne pas sur-dimensionner. |
| **Réglementation FM** | Haute | Moyen | FM = Phase 3 uniquement, avec avocat. Pas dans le MVP. |

---

## §11 Feuille de route

### Phase 1 (0-3 mois) — MVP Coordination

- [ ] Client Tauri + Rust core (identité Nostr secp256k1)
- [ ] Messagerie 1:1 E2E sur internet (NIP-17 + NIP-44 + NIP-59)
- [ ] Cellules de 3 (création, invitation, clé de groupe)
- [ ] Web of trust Niveau 0-1 (inscription ouverte + parrainage)
- [ ] 10 testeurs (cellules militantes)

**Budget** : 280 € (1 nœud relais de test)
**Critère de succès** : 3 cellules communiquent en E2E sur internet, avec forward secrecy et auto-destruction.

### Phase 2 (3-6 mois) — Nœuds Relais

- [ ] Package nœud relais (Raspberry Pi 5 + Reticulum + IPFS + Nostr relay)
- [ ] Cache local pour articles et podcasts
- [ ] Détection automatique de dégradation internet
- [ ] Bascule automatique vers Reticulum WiFi
- [ ] 10 nœuds relais déployés

**Budget** : 1 500-2 800 € (10 relais × 150-280 €)
**Critère de succès** : quand internet est coupé, les messages texte continuent via Reticulum WiFi.

### Phase 3 (6-12 mois) — Publication

- [ ] Nœud créateur (PeerTube + Owncast sur mini PC)
- [ ] Pré-transcodage vidéo + distribution WebTorrent
- [ ] Feed RSS + IPFS pour podcasts
- [ ] Articles Nostr + IPFS avec signature vérifiable
- [ ] 3 nœuds créateurs déployés

**Budget** : 1 200-2 400 € (3 créateurs × 400-800 €)
**Critère de succès** : 1 créateur publie une vidéo + un article + un podcast, distribués P2P.

### Phase 4 (12-18 mois) — Résilience off-grid

- [ ] Reticulum LoRa (RNode) sur les nœuds relais
- [ ] Meshtastic en fallback extrême
- [ ] Synchronisation post-dégradation automatique
- [ ] 100 consommateurs + 10 relais + 3 créateurs

**Budget** : 800-2 400 €/an (électricité + maintenance)
**Critère de succès** : test de résilience — couper internet, les messages texte continuent via LoRa.

---

## §12 Spécifications non-fonctionnelles

| Exigence | Valeur |
|----------|--------|
| **Taille du client** | < 15 Mo (Tauri) |
| **Mémoire RAM** | < 200 Mo (consommateur), < 1 Go (relais), < 4 Go (créateur) |
| **CPU** | < 5 % idle (consommateur), < 20 % idle (relais) |
| **Démarrage** | < 3 secondes |
| **Latence messagerie** | < 1s (internet), < 5s (Reticulum WiFi), < 30s (LoRa) |
| **Disponibilité nœud relais** | > 99 % (24/7) |
| **Chiffrement** | secp256k1 (signature) + NIP-44 ChaCha20-Poly1305 (E2E) |
| **Stockage local** | SQLite chiffré (SQLCipher) |
| **Compatibilité** | Windows 10+, macOS 12+, Linux (Debian 12+, Ubuntu 22.04+, Arch) |
| **Mobile** | Phase 5 (après validation desktop) |
