# Bump the version, commit, tag, and push.
# Usage:
#   just
#   just release
#   just release bump=minor
#   just release bump=major
release bump="patch":
    #!/usr/bin/env bash
    set -euo pipefail

    current=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    IFS='.' read -r major minor patch <<< "$current"

    case "{{ bump }}" in
        patch)
            next="$major.$minor.$((patch + 1))"
            ;;
        minor)
            next="$major.$((minor + 1)).0"
            ;;
        major)
            next="$((major + 1)).0.0"
            ;;
        *)
            echo "invalid bump '{{ bump }}'; expected patch, minor, or major" >&2
            exit 1
            ;;
    esac

    echo "Bumping $current -> $next"

    sed -i.bak "s/^version = \"$current\"/version = \"$next\"/" Cargo.toml
    rm Cargo.toml.bak

    cargo update --workspace --quiet

    git add Cargo.toml Cargo.lock
    git commit -m "chore: bump version to $next"

    git tag -a "v$next" -m "v$next"

    git push origin main
    git push origin "v$next"

    echo "Released v$next"

# Update the Homebrew tap formula with release SHAs.
# Usage:
#   just update-tap
#   just update-tap version=0.1.0
#   just update-tap version=0.1.0 owner_repo=mlnja/llmfmt tap_dir=../homebrew-tap
update-tap version="" owner_repo="mlnja/llmfmt" tap_dir="../homebrew-tap":
    #!/usr/bin/env bash
    set -euo pipefail

    if [[ -z "{{ version }}" ]]; then
        VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    else
        VERSION="{{ version }}"
    fi

    FORMULA="{{ tap_dir }}/Formula/llmfmt.rb"
    TMPDIR=$(mktemp -d)
    trap "rm -rf $TMPDIR" EXIT

    if [[ ! -f "$FORMULA" ]]; then
        echo "formula not found: $FORMULA" >&2
        exit 1
    fi

    echo "Updating Homebrew tap for v$VERSION from {{ owner_repo }}..."

    for platform in darwin-arm64 darwin-amd64 linux-arm64 linux-amd64; do
        url="https://github.com/{{ owner_repo }}/releases/download/v$VERSION/llmfmt-$platform.tar.gz.sha256"
        sha=$(curl -fsSL "$url" | cut -d' ' -f1)
        echo "  $platform  $sha"
        awk -v sha="$sha" -v marker="# $platform" \
            '$0 ~ marker { sub(/"[0-9a-f]+"/, "\"" sha "\"") } { print }' \
            "$FORMULA" > "$TMPDIR/formula.tmp" && mv "$TMPDIR/formula.tmp" "$FORMULA"
    done

    echo "  downloading source tarball for SHA..."
    curl -fsSL "https://github.com/{{ owner_repo }}/archive/refs/tags/v$VERSION.tar.gz" \
        -o "$TMPDIR/source.tar.gz"
    source_sha=$(shasum -a 256 "$TMPDIR/source.tar.gz" | cut -d' ' -f1)
    echo "  source  $source_sha"
    awk -v sha="$source_sha" -v marker="# source" \
        '$0 ~ marker { sub(/"[0-9a-f]+"/, "\"" sha "\"") } { print }' \
        "$FORMULA" > "$TMPDIR/formula.tmp" && mv "$TMPDIR/formula.tmp" "$FORMULA"

    sed -i.bak "s/version \"[^\"]*\"/version \"$VERSION\"/" "$FORMULA"
    rm "$FORMULA.bak"

    (
        cd "{{ tap_dir }}"
        git add Formula/llmfmt.rb
        git commit -m "chore: update llmfmt to v$VERSION"
        git push origin main
    )

    echo "Homebrew tap updated for v$VERSION"
