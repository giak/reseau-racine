# Brainstorm — RéseauRacine

## Contexte

Deux articles convergent vers le même diagnostic :

- **Opposition contrôlée** (2026-05-20) : le système ne censure pas, il absorbe. 88 % de la dissidence apparente est absorbée par l'infrastructure qu'elle prétend combattre. La dissidence dépend des plateformes, des algorithmes, du financement, de la distribution.
- **Effondrement structurel** (2026-05-21) : 5 circuits de capture maintiennent le système. Les solutions existent mais ne valent rien sans la volonté populaire — et la volonté ne naît pas des plans, elle naît du peuple. Le RIC est la condition de possibilité d'une politique adulte.

**Le problème fondamental** : toute opposition qui dépend de YouTube, X, Facebook, Substack, les sondages IFOP, les aides publiques, les milliardaires du mégaphone est mécaniquement absorbable. Le système ne la supprime pas — il la rend dépendante.

**L'objectif** : construire un réseau pérenne, robuste, souverain qui échappe à cette absorption.

---

## Diagnostic des vulnérabilités actuelles

Chaque couche de dépendance est un point de capture :

| Couche | Dépendance actuelle | Vecteur d'absorption |
|--------|-------------------|---------------------|
| **Hébergement** | AWS, Google Cloud, OVH (soumis à la loi française) | Coupure administrative, réquisition |
| **Distribution** | YouTube, X, Facebook, Apple App Store, Google Play | Dépriorisation algorithmique, bannissement |
| **Financement** | Subventions publiques, Tipeee, publicité | Asphyxie, conditionnement idéologique |
| **Identité** | Comptes email GAFAM, numéros de téléphone | Désactivation, surveillance |
| **Communication** | Telegram, WhatsApp, Signal (dépendent d'entreprises centralisées) | Blocage ISP, pression juridique |
| **Coordination** | Discord, Slack, Google Docs | Fermeture de compte, censure |

**Le système n'a pas besoin de censurer.** Il lui suffit de contrôler une couche.

---

## Principes de conception

### Principe 1 : Redondance par la distribution
Aucun point de contrôle unique. Si un nœud tombe, le réseau continue.

### Principe 2 : Souveraineté par l'indépendance économique
Zéro subvention publique. Zéro dépendance à un seul financeur. Modèle : abonnements distribués + dons micro + coopérative.

### Principe 3 : Résilience par l'interopérabilité
Le réseau ne doit pas être une plateforme — il doit être un protocole. Comme l'email : personne ne « possède » l'email.

### Principe 4 : Invisibilité par la normalité
Le réseau ne doit pas ressembler à une « dissidence ». Il doit ressembler à un outil normal. Le système cible l'anomalie, pas la normalité.

### Principe 5 : Capture-proof par la gouvernance distribuée
Pas de leader unique. Pas de fondation contrôlable. Décisions par consensus distribué. Code open-source, auditabilité totale. Cellules de 3 (Trinité), essaims de 10. Pas de traces numériques sensibles. Confiance organique.

### Principe 6 : Polydépendance, pas autonomie
Le réseau ne vise pas l'autonomie totale (fantasme) mais la polydépendance réelle — dépendre de ses pairs, de la matière, du temps, de la terre, pas des algorithmes. Couper un fil par mois, transmettre à une seule personne.

---

## Architecture en 5 couches

### COUCHE 1 : Infrastructure physique — le maillage

**Objectif** : ne dépendre d'aucun hébergeur centralisé.

- **Hébergement distribué** : fédérations de serveurs auto-hébergés (YunoHost, Sandstorm) chez des particuliers, associations, petites entreprises. Chaque nœud héberge une partie du contenu. Si un nœud tombe, les autres continuent.
- **Mesh networks locaux** : réseaux Wi-Fi communautaires (LibreMesh, Althea) dans les zones urbaines. Communication locale sans ISP. Résistant aux coupures nationales.
- **Hébergement souverain** : serveurs dans des juridictions non-alignées (Suisse, Islande, certains États américains). Multiplication des juridictions = aucun État ne peut tout couper.
- **Stockage distribué** : IPFS, Storj, Sia. Le contenu n'est pas sur un serveur — il est répliqué sur des centaines de nœuds. Impossible à supprimer.
- **Radio Maquis** : émetteurs FM/DAB+ mobiles sur batteries pour diffuser de l'information et coordonner sans internet. Backup quand les ISPs sont coupés. Rupture du monopole narratif en zone rurale.

**Coût estimé** : 500-2000 €/an pour un nœud de base. 100 nœuds = réseau résilient.

### COUCHE 2 : Protocoles — l'interopérabilité

**Objectif** : ne dépendre d'aucune plateforme propriétaire.

- **Messagerie** : Matrix (fédéré, E2E, open-source) + Session (P2P, pas de numéro de téléphone, pas de serveur central). Matrix pour la coordination organisée, Session pour la communication anonyme.
- **Publication** : ActivityPub (fédéré, comme Mastodon) + Nostr (P2P, pas de serveur, identité par clé cryptographique). ActivityPub pour les communautés structurées, Nostr pour la publication ouverte.
- **Vidéo** : PeerTube (fédéré, P2P) + IPFS pour le stockage distribué. Pas de dépendance à YouTube.
- **Audio/Podcasts** : RSS + IPFS. Le RSS est indestructible — c'est un protocole ouvert, pas une plateforme.
- **Coordination** : Loomio (décision distribuée) + CryptPad (documents collaboratifs chiffrés, auto-hébergé).

**Le point clé** : ces protocoles sont interopérables. Un utilisateur Matrix peut parler à un utilisateur Session via des bridges. Un utilisateur Mastodon peut lire un article Nostr via des relays. Le réseau n'est pas un silo — c'est un écosystème.

### COUCHE 3 : Applications — l'utilité

**Objectif** : le réseau doit être utile, pas juste « alternatif ».

- **Portail d'information** : agrégateur de contenus hébergés sur IPFS/ActivityPub. Interface web propre, rapide, sans tracking. Alternative à Google News.
- **Plateforme de publication** : outil de blogging auto-hébergé (WriteFreed, Ghost) avec fédération ActivityPub. Alternative à Substack.
- **Réseau social** : instance Mastodon thématique + relais Nostr. Alternative à X/Facebook.
- **Vidéothèque** : instance PeerTube avec catalogage. Alternative à YouTube.
- **Outils de coordination** : Loomio pour les décisions, CryptPad pour les documents, Matrix pour la communication. Alternative à Discord/Slack/Google Docs.
- **Économie** : système de micro-paiements (Lightning Network) + abonnements via Open Collective coopératif. Alternative à Tipeee/Patreon.

**Le point clé** : chaque application doit être **meilleure** que son équivalent GAFAM sur au moins un critère (vie privée, absence de censure, communauté, qualité). Pas « aussi bien mais éthique » — **meilleure**.

### COUCHE 4 : Identité — la souveraineté personnelle

**Objectif** : ne dépendre d'aucun fournisseur d'identité.

- **Identité cryptographique** : clés PGP/Nostr comme identité primaire. Pas de numéro de téléphone, pas d'email GAFAM. La clé cryptographique est l'identité — elle ne peut pas être supprimée par une plateforme.
- **Réputation distribuée** : système de réputation basé sur les interactions vérifiables, pas sur les likes algorithmiques. La réputation est portable — elle suit l'identité, pas la plateforme.
- **Anonymat optionnel** : pseudonymes cryptographiques pour les contributions sensibles. L'anonymat n'est pas la clandestinité — c'est la protection de la source.

### COUCHE 5 : Gouvernance — l'anti-capture

**Objectif** : empêcher la capture par le système ou par des acteurs internes.

- **Pas de leader unique** : gouvernance par collège de 7-15 membres élus pour 18 mois, non renouvelables. Comme le CCP du RIC.
- **Cellules de 3 (Trinité)** : chaque cellule gère un nœud. 10 cellules = 1 essaim = 1 région. Coordination sans hiérarchie.
- **6 règles du ré-enracinement** : (1) Ne jamais devenir leader. (2) Ne rien demander au système. (3) Ne pas se faire connaître. (4) Ne rien promettre. (5) Ne haïr personne. (6) Reconnaître les siens sans s'organiser.
- **Code open-source** : tout le code est public, auditable. Toute modification est tracée. Aucune backdoor possible.
- **Financement distribué** : aucun contributeur ne peut dépasser 5 % du budget annuel. Modèle : 60 % abonnements, 30 % dons micro, 10 % services (hébergement, formation).
- **Transparence radicale** : comptes publiés mensuellement. Décisions de gouvernance publiques. Conflits d'intérêts déclarés.
- **Constitution du réseau** : document fondateur définissant les principes, les règles de gouvernance, les mécanismes de résolution de conflits. Modifiable uniquement par vote distribué (quorum 30 %, majorité 66 %).

---

## Feuille de route — 36 mois

### Phase 1 (0-6 mois) : Fondation
- Constituer le collège de gouvernance (7 membres, 18 mois)
- Rédiger la constitution du réseau
- Déployer les premiers nœuds (10 serveurs auto-hébergés)
- Lancer l'instance Matrix, l'instance PeerTube, le portail ActivityPub
- Recruter les 100 premiers utilisateurs (testeurs, contributeurs)

### Phase 2 (6-18 mois) : Croissance
- Passer à 50 nœuds
- Lancer l'application mobile (F-Droid, pas d'App Store)
- Intégrer le système de micro-paiements Lightning
- Recruter 1000 utilisateurs
- Former les utilisateurs à l'auto-hébergement

### Phase 3 (18-36 mois) : Résilience
- Passer à 200 nœuds
- Déployer les mesh networks locaux (5 villes)
- Atteindre 10 000 utilisateurs
- Rendre le réseau interopérable avec les protocoles existants (email, RSS, ActivityPub)
- Publier le premier audit de sécurité indépendant

---

## Risques et contre-mesures

| Risque | Probabilité | Impact | Contre-mesure |
|--------|------------|--------|---------------|
| Coupure ISP nationale | Moyenne | Élevé | Mesh networks locaux + satellite (Starlink en backup) |
| Bannissement App Store | Haute | Moyen | Distribution F-Droid + PWA + APK direct |
| Infiltration par Viginum | Haute | Élevé | Identité cryptographique vérifiée + réputation distribuée |
| Épuisement des fondateurs | Haute | Élevé | Gouvernance distribuée dès le jour 1 — pas de dépendance à des individus |
| Manque d'adoption | Haute | Critique | Se concentrer sur l'utilité, pas l'idéologie. Le réseau doit être utile avant d'être souverain. |
| Capture financière | Moyenne | Élevé | Plafond 5 % par contributeur. Transparence radicale. |
| Attaque juridique | Moyenne | Moyen | Hébergement multi-juridictions. Constitution légale solide. |

---

## Le point aveugle — ce que personne ne dit

Le plus grand obstacle n'est pas technique. **C'est social.**

Les gens n'abandonneront pas YouTube, X, WhatsApp parce que c'est « éthique ». Ils le feront si l'alternative est **meilleure** — plus rapide, plus fiable, plus utile, plus communautaire.

Le réseau souverain ne doit pas être un « réseau de dissidents ». Il doit être un **réseau de citoyens qui veulent reprendre le contrôle**. La différence est fondamentale : la dissidence est une niche, la souveraineté est un besoin universel.

**La stratégie d'adoption** : commencer par les usages les plus sensibles à la censure (journalistes, lanceurs d'alerte, chercheurs, avocats), puis élargir progressivement aux usages quotidiens (famille, travail, communauté). Le réseau grandit par la confiance, pas par le marketing.

---

## Lien avec les articles existants

### Effondrement structurel → RéseauRacine
- Les 5 circuits de capture identifiés dans l'article sont tous brisés par le réseau :
  - Circuit interne (ENA/pantouflage/médias) → brisé par la gouvernance distribuée
  - Circuit externe (Françafrique) → brisé par l'hébergement multi-juridictions
  - Circuit de reproduction (éducation) → brisé par la formation à l'auto-hébergement
  - Circuit de spoliation (fiscal) → brisé par le financement distribué
  - Circuit du vide (État absent) → brisé par les mesh networks locaux

### Opposition contrôlée → RéseauRacine
- Les 88 % de dissidence absorbée le sont parce qu'ils dépendent des plateformes
- RéseauRacine inverse la logique : **l'infrastructure d'abord, le contenu ensuite**
- Putsch, Tocsin, Blast produisent du contenu souverain sur des infrastructures captives
- RéseauRacine produit des infrastructures souveraines pour du contenu libre
- Le système ne censure pas — il rend dépendant. Chaque couche de dépendance doit avoir une alternative souveraine.

### RIC (Substack) → RéseauRacine
- Le RIC n'est pas qu'un outil politique — c'est aussi un outil de gouvernance pour le réseau
- Le CCP (Conseil Citoyen Permanent) peut être le collège de gouvernance du réseau
- Les Assemblées Citoyennes peuvent être les instances de décision du réseau
- Le RIC peut être le mécanisme de modification de la constitution du réseau
- **Le RIC et le réseau souverain sont les deux faces d'une même pièce** : l'un reprend le contrôle politique, l'autre reprend le contrôle informationnel

### Asphyxie du Golem → RéseauRacine
- **Radio Maquis** : émetteurs FM/DAB+ mobiles comme backup quand internet est coupé
- **Structure gazeuse** : protocoles mutables, pas de centre, forks autorisés
- **Trinité/Essaim** : cellules de 3 pour la gouvernance locale, essaims de 10 pour la coordination régionale
- **Ischémie/Engorgement/Cécité** : le réseau brise ces trois piliers en étant distribué, décentralisé, chiffré

### Protocole du ré-enracinement → RéseauRacine
- **Polydépendance** : dépendre de ses pairs, de la matière, du temps — pas des algorithmes
- **Couper un fil par mois** : adoption progressive, pas révolutionnaire
- **Transmettre à une seule personne** : croissance par confiance organique, pas par marketing
- **6 règles** : ne jamais devenir leader, ne rien demander au système, ne pas se faire connaître, ne rien promettre, ne haïr personne, reconnaître les siens sans s'organiser

### Ingénierie de l'Enclos → RéseauRacine
- La domestication est numérique, financière, cognitive. Le réseau est une sortie de l'enclos.

### Ingénierie de la Possession → RéseauRacine
- 3 siècles de dépossession (enclosures → colonisation → financiarisation). Le réseau est un outil de ré-appropriation de la souveraineté informationnelle.

---

## Questions ouvertes

1. **Quel est le MVP (Minimum Viable Product) ?** Quelle est la plus petite version du réseau qui soit utile ?
2. **Qui sont les premiers utilisateurs cibles ?** Journalistes ? Lanceurs d'alerte ? Communautés locales ?
3. **Quel est le modèle économique initial ?** Qui paie les premiers serveurs ?
4. **Comment recruter le collège de gouvernance ?** Par cooptation ? Par élection ? Par tirage au sort ?
5. **Quelle est la première application « killer » ?** Celle qui fait venir les gens.
6. **Comment mesurer la résilience ?** Quel est le seuil de nœuds tombés avant que le réseau ne devienne inutilisable ?
7. **Comment gérer la modération ?** Sans censure centralisée, comment éviter le spam, la désinformation, les contenus illégaux ?
8. **Quelle est la stratégie juridique ?** Association ? Coopérative ? Fondation ? Décentralisée pure ?
9. **Comment articuler avec le RIC ?** Le réseau est-il l'infrastructure technique du RIC, ou un projet parallèle ?
10. **Quel est le nom de la constitution du réseau ?** Quel est le document fondateur ?

---

## Prochaines étapes

- [ ] Définir le MVP
- [ ] Identifier les 7 premiers membres du collège de gouvernance
- [ ] Rédiger la constitution du réseau
- [ ] Déployer les 10 premiers nœuds
- [ ] Lancer les premières applications (Matrix, PeerTube, ActivityPub)
- [ ] Recruter les 100 premiers utilisateurs
