#!/usr/bin/env bash
set -euo pipefail

# ── helpers ────────────────────────────────────────────────────────────────────
red()   { echo -e "\033[1;31m$*\033[0m"; }
green() { echo -e "\033[1;32m$*\033[0m"; }
cyan()  { echo -e "\033[1;36m$*\033[0m"; }
die()   { red "✗ $*"; exit 1; }

# ── deps check ─────────────────────────────────────────────────────────────────
for cmd in cargo git gh; do
    command -v "$cmd" &>/dev/null || die "'$cmd' not found. Please install it first."
done

# ── parse flags ────────────────────────────────────────────────────────────────
CROSS=false
PRERELEASE=false
NOTES=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --cross)       CROSS=true ;;
        --prerelease)  PRERELEASE=true ;;
        --notes)       shift; NOTES="$1" ;;
        -h|--help)
            echo "Usage: ./release.sh [--cross] [--prerelease] [--notes 'release notes']"
            exit 0 ;;
        *) die "Unknown flag: $1" ;;
    esac
    shift
done

# ── read version from Cargo.toml ───────────────────────────────────────────────
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
TAG="v${VERSION}"

cyan "Releasing ${TAG}…"

# ── dirty tree check ──────────────────────────────────────────────────────────
if [[ -n "$(git status --porcelain)" ]]; then
    die "Working tree is dirty. Commit or stash changes before releasing."
fi

# ── tag must not already exist ────────────────────────────────────────────────
if git rev-parse "$TAG" &>/dev/null; then
    die "Tag $TAG already exists. Bump the version in Cargo.toml first."
fi

# ── build ─────────────────────────────────────────────────────────────────────
TARGETS=()

if $CROSS; then
    command -v cross &>/dev/null || cargo install cross --quiet
    for target in x86_64-unknown-linux-musl aarch64-unknown-linux-musl; do
        cyan "Building for $target…"
        cross build --release --target "$target"
        BIN="target/${target}/release/vortex-dl"
        OUT="vortex-dl-${VERSION}-${target}"
        cp "$BIN" "$OUT"
        TARGETS+=("$OUT")
    done
else
    cyan "Building for host…"
    cargo build --release
    HOST_TARGET=$(rustc -vV | grep host | awk '{print $2}')
    BIN="target/release/vortex-dl"
    OUT="vortex-dl-${VERSION}-${HOST_TARGET}"
    cp "$BIN" "$OUT"
    TARGETS+=("$OUT")
fi

# ── tag & push ────────────────────────────────────────────────────────────────
cyan "Creating tag $TAG…"
git tag -a "$TAG" -m "Release $TAG"
git push origin "$TAG"

# ── github release ────────────────────────────────────────────────────────────
RELEASE_FLAGS=()
$PRERELEASE && RELEASE_FLAGS+=(--prerelease)
[[ -n "$NOTES" ]] && RELEASE_FLAGS+=(--notes "$NOTES") || RELEASE_FLAGS+=(--generate-notes)

cyan "Creating GitHub Release $TAG…"
gh release create "$TAG" "${TARGETS[@]}" \
    --title "VortexDL $TAG" \
    "${RELEASE_FLAGS[@]}"

# ── cleanup ───────────────────────────────────────────────────────────────────
rm -f "${TARGETS[@]}"

green "✓ Released $TAG successfully!"
