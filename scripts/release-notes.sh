#!/bin/sh
# Prints release notes for a tag: the commits since the previous tag.
# Lists up to 10 commits; if there are more, the rest are linked.
#   scripts/release-notes.sh v0.2.0 [owner/repo]
set -eu

tag="$1"
repo="${2:-dsim01/podatki}"
limit=10

if prev=$(git describe --tags --abbrev=0 "$tag^" 2>/dev/null); then
    range="$prev..$tag"
else
    prev=""
    range="$tag"
fi

count=$(git rev-list --no-merges --count "$range")

echo "## What's changed"
echo
git log --no-merges --max-count="$limit" --format='- %s (%h)' "$range"

if [ "$count" -gt "$limit" ]; then
    echo "- …and $((count - limit)) more"
fi

if [ -n "$prev" ]; then
    echo
    echo "**Full changelog:** https://github.com/$repo/compare/$prev...$tag"
fi
