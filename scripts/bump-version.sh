#!/usr/bin/env bash
set -euo pipefail

BUMP_TYPE="${1:-}"
CARGO_TOML="${2:-Cargo.toml}"

if [[ -z "$BUMP_TYPE" ]]; then
    echo "Uso: $0 <major|minor|patch> [caminho/Cargo.toml]"
    exit 1
fi

CURRENT=$(grep -m1 '^version' "$CARGO_TOML" | sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+)".*/\1/')
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

case "$BUMP_TYPE" in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
    *)
        echo "Tipo de bump inválido: $BUMP_TYPE. Use major, minor ou patch."
        exit 1
        ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"

# Atualiza apenas a primeira ocorrência de version = "..." no [package]
sed -i -E "0,/^version = \"[^\"]+\"/s//version = \"${NEW_VERSION}\"/" "$CARGO_TOML"

echo "Versão atualizada: ${CURRENT} -> ${NEW_VERSION}"
