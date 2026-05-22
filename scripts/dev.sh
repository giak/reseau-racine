#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="$(dirname "$0")/../.devcontainer/compose.yaml"
COMPOSE_PROJECT="reseau-racine"

if ! docker compose -f "$COMPOSE_FILE" ps --services 2>/dev/null | grep -q .; then
  echo "→ Démarrage des services Docker..." >&2
  docker compose -f "$COMPOSE_FILE" -p "$COMPOSE_PROJECT" up -d
fi

tty_args=""
if [ -t 0 ]; then
  tty_args="--tty"
fi

if [ $# -eq 0 ]; then
  exec docker compose -f "$COMPOSE_FILE" -p "$COMPOSE_PROJECT" exec ${tty_args} dev bash
fi

exec docker compose -f "$COMPOSE_FILE" -p "$COMPOSE_PROJECT" exec ${tty_args} dev "$@"
