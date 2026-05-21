# Questions ouvertes — RéseauRacine (réponses proposées)

## 1. Quel est le MVP (Minimum Viable Product) ?

**Réponse proposée** : Le MVP n'est pas un réseau complet — c'est **un seul nœud fonctionnel** qui démontre la souveraineté sur les 6 couches de dépendance.

Le MVP = **1 serveur auto-hébergé + 1 instance Matrix + 1 instance PeerTube + 1 relais Nostr + 1 émetteur FM local + 1 porte-monnaie Lightning**.

Ce nœud unique doit prouver que :
- L'hébergement est souverain (pas AWS/Google)
- La messagerie est chiffrée et fédérée (Matrix)
- La vidéo est distribuée (PeerTube + IPFS)
- La publication est résistante à la censure (Nostr)
- La communication fonctionne sans internet (FM)
- Le financement est indépendant (Lightning)

**Critère de succès du MVP** : 10 utilisateurs peuvent communiquer, publier, regarder des vidéos, et recevoir de l'argent — sans aucune dépendance aux GAFAM.

**Coût estimé** : 500-1000 € (serveur + matériel radio + configuration).

---

## 2. Qui sont les premiers utilisateurs cibles ?

**Réponse proposée** : Commencer par les **plus sensibles à la censure**, pas par le grand public.

Ordre de priorité :
1. **Journalistes indépendants** (Glanz, Dufresne, Vidal) — ils ont déjà du contenu souverain mais des infrastructures captives
2. **Lanceurs d'alerte** — besoin de communication chiffrée et de publication résistante
3. **Avocats et chercheurs** — besoin de coordination sécurisée et de stockage distribué
4. **Communautés locales** (agriculteurs, ZAD, associations) — besoin de communication sans dépendance ISP
5. **Grand public** — seulement quand les 4 premiers groupes sont satisfaits

**Pourquoi cet ordre** : les journalistes et lanceurs d'alerte ont une audience. S'ils adoptent le réseau, leur audience suit. C'est la stratégie du « pont » — les early adopters sensibles à la censure servent de pont vers le grand public.

---

## 3. Quel est le modèle économique initial ?

**Réponse proposée** : **Coopérative SCOP** (Société Coopérative et Participative).

- **60 % abonnements** : 4,99 €/mois par utilisateur (comme Putsch, mais pour l'infrastructure, pas le contenu)
- **30 % dons micro** : Lightning Network, pas de plateforme centrale
- **10 % services** : hébergement pour tiers, formation à l'auto-hébergement, audit de sécurité

**Règles** :
- Aucun contributeur ne peut dépasser 5 % du budget annuel
- Comptes publiés mensuellement
- Décisions de budget par vote distribué (quorum 30 %, majorité 66 %)
- Pas de subvention publique — jamais

**Pourquoi une SCOP** : structure légale française, gouvernance démocratique (1 personne = 1 voix), pas d'actionnaires externes, bénéfices réinvestis. C'est l'équivalent légal de la gouvernance distribuée.

---

## 4. Comment recruter le collège de gouvernance ?

**Réponse proposée** : **Cooptation initiale → élection distribuée après 12 mois.**

**Phase 1 (0-12 mois)** : 7 membres cooptés. Critères :
- Compétence technique (hébergement, cryptographie, réseaux)
- Indépendance financière (pas de subvention publique, pas de lien avec les GAFAM)
- Intégrité vérifiable (pas de condamnation, pas de conflit d'intérêts)
- Disponibilité (18 mois, non renouvelable)

**Phase 2 (12+ mois)** : élection distribuée par les utilisateurs actifs. Chaque utilisateur avec 6+ mois d'ancienneté peut voter. 7 membres élus pour 18 mois, non renouvelables.

**Pourquoi cooptation d'abord** : au démarrage, il n'y a pas assez d'utilisateurs pour élire. La cooptation garantit la compétence. L'élection après 12 mois garantit la légitimité.

---

## 5. Quelle est la première application « killer » ?

**Réponse proposée** : **Le portail d'information souverain.**

Pas un réseau social. Pas une messagerie. Un **portail d'information** qui agrège des contenus hébergés sur IPFS/ActivityPub, avec une interface web propre, rapide, sans tracking.

**Pourquoi** : c'est l'application la plus visible, la plus facile à comprendre, et la plus utile immédiatement. Les gens cherchent de l'information fiable. Le portail leur donne ça — sans GAFAM, sans tracking, sans censure algorithmique.

**Fonctionnalités MVP** :
- Agrégation de contenus RSS + ActivityPub + Nostr
- Interface web propre (pas de pub, pas de tracking)
- Hébergement IPFS (contenu distribué, impossible à supprimer)
- Recherche full-text (pas d'algorithme de recommandation)
- Export PDF/EPUB (lecture hors ligne)

**Nom proposé** : « La Source » — parce que c'est là que l'information prend sa source, avant d'être déformée par les algorithmes.

---

## 6. Comment mesurer la résilience ?

**Réponse proposée** : **Score de résilience en 5 dimensions.**

| Dimension | Métrique | Seuil critique | Seuil acceptable |
|-----------|----------|----------------|------------------|
| **Hébergement** | % de nœuds actifs | <50 % | >80 % |
| **Distribution** | % de contenu répliqué | <3 copies | >7 copies |
| **Financement** | % du budget d'un seul contributeur | >20 % | <5 % |
| **Communication** | Latence moyenne du réseau | >10s | <2s |
| **Gouvernance** | % de membres du collège actifs | <50 % | >80 % |

**Test de résilience mensuel** : simuler la chute de 30 % des nœuds. Le réseau doit rester fonctionnel.

---

## 7. Comment gérer la modération ?

**Réponse proposée** : **Modération distribuée par réputation, pas par censure.**

Pas de modération centralisée. Pas de « police du réseau ». À la place :

- **Système de réputation** : chaque utilisateur a un score de réputation basé sur les interactions vérifiables (pas sur les likes). La réputation est portable — elle suit l'identité, pas la plateforme.
- **Signalement distribué** : tout utilisateur peut signaler un contenu. Le signalement est vérifié par 3 pairs aléatoires (cellule de 3). Si 2/3 confirment, le contenu est marqué — pas supprimé, mais réduit en visibilité.
- **Appel** : tout utilisateur peut faire appel d'une décision. L'appel est jugé par un essaim de 10 pairs.
- **Transparence** : toutes les décisions de modération sont publiques (sauf les données personnelles).

**Pourquoi pas de censure** : la censure centralisée est un point de capture. Si le réseau a une « police », le système peut la capturer. La modération distribuée par réputation est plus résiliente.

---

## 8. Quelle est la stratégie juridique ?

**Réponse proposée** : **Coopérative SCOP + fondation de droit suisse pour l'hébergement.**

- **France** : SCOP pour les opérations françaises (abonnements, formation, services). Gouvernance démocratique, pas d'actionnaires externes.
- **Suisse** : fondation de droit suisse pour l'hébergement des données critiques. La Suisse a des lois fortes sur la protection des données et n'est pas dans l'UE.
- **Islande** : serveur backup en Islande pour la redondance juridique.

**Pourquoi cette structure** : aucune juridiction unique ne peut tout couper. La France peut fermer la SCOP — la fondation suisse continue. La Suisse peut coopérer — le serveur islande continue.

---

## 9. Comment articuler avec le RIC ?

**Réponse proposée** : **Le réseau est l'infrastructure technique du RIC. Le RIC est la gouvernance politique du réseau.**

- Le **CCP** (Conseil Citoyen Permanent) du RIC est le collège de gouvernance du réseau.
- Les **Assemblées Citoyennes** du RIC sont les instances de décision du réseau.
- Le **RIC** est le mécanisme de modification de la constitution du réseau.
- **SecNumCloud + X-Road FR** sont l'infrastructure technique du RIC.

**Les deux faces d'une même pièce** :
- Le RIC reprend le contrôle politique → le réseau reprend le contrôle informationnel
- Le RIC a besoin d'une infrastructure souveraine → le réseau a besoin d'une gouvernance démocratique
- Le RIC sans réseau = dépendance aux plateformes GAFAM
- Le réseau sans RIC = risque de capture interne

---

## 10. Quel est le nom de la constitution du réseau ?

**Réponse proposée** : **« La Charte des Racines »**

Parce que le réseau s'appelle RéseauRacine, et que l'enracinement est le besoin le plus important et le plus méconnu de l'âme humaine (Simone Weil).

**Structure de la Charte** :
- **Préambule** : pourquoi le réseau existe (diagnostic de la dépossession systémique)
- **Article 1** : principes fondateurs (redondance, souveraineté, interopérabilité, invisibilité, capture-proof, polydépendance)
- **Article 2** : gouvernance (collège, cellules, essaims, élection, transparence)
- **Article 3** : financement (abonnements, dons, services, plafonds, transparence)
- **Article 4** : modération (réputation, signalement, appel, transparence)
- **Article 5** : modification (RIC distribué, quorum 30 %, majorité 66 %)
- **Annexes** : stack technique, protocoles, procédures de sécurité

---

## Prochaines étapes prioritaires

1. **Rédiger la Charte des Racines** (1-2 semaines)
2. **Identifier les 7 premiers membres du collège** (cooptation initiale)
3. **Déployer le MVP** (1 serveur + Matrix + PeerTube + Nostr + FM + Lightning)
4. **Recruter les 10 premiers utilisateurs** (journalistes indépendants)
5. **Lancer « La Source »** (portail d'information souverain)
