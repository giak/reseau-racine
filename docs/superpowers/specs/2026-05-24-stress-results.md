# rr-stress — Résultats de charge

**Date :** 2026-05-24
**Relais :** nostr-rs-relay (Docker, dev machine)
**Message :** `"stress test payload"` via `send_private_msg`
**Parallélisme :** 4 workers

## Courbe de charge

| Users | Messages | Total | p50    | p95    | p99    | Débit    | Échecs |
|-------|----------|-------|--------|--------|--------|----------|--------|
| 50    | 10       | 500   | 34 ms  | 55 ms  | 57 ms  | 1165/s   | 0 |
| 200   | 10       | 2000  | 138 ms | 160 ms | 166 ms | 1121/s   | 0 |
| 500   | 10       | 5000  | 411 ms | 420 ms | 429 ms | 1045/s   | 0 |
| 1000  | 10       | 10000 | 939 ms | 960 ms | 971 ms | 938/s    | 0 |

## Observations

- **0 échecs** sur tous les tests (0 timeout, 0 reject, 0 disconnect)
- Latence **linéaire** avec le nombre d'utilisateurs (~0.9 ms/user supplémentaire)
- Débit se dégrade lentement : ~20% entre 50 et 1000 users
- **Pas de point de rupture atteint** — le relais se dégrade gracieusement
- La spéculation "nostr-rs-relay sature à ~50-100 clients" est **infirmée** — le goulot est ailleurs (CPU WebSocket ? binaire Rust optimisé ?)

## Commande

```bash
cargo run --release --package rr-stress -- --users N --messages 10 --interval 0
```
