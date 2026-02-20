#!/usr/bin/env bash
# pre-push-docs.sh — Auto-generate docs before push
# Setup: cp scripts/pre-push-docs.sh .git/hooks/pre-push && chmod +x .git/hooks/pre-push

set -e

echo "🔄 Auto-generating documentation..."

AID="./compiler/target/release/aid"

# Build compiler if needed
if [ ! -f "$AID" ]; then
    echo "  Building compiler..."
    (cd compiler && cargo build --release)
fi

# Generate docs for all examples
mkdir -p docs/generated
for f in examples/*.aid; do
    name=$(basename "$f" .aid)
    echo "  📄 Generating docs for $name..."
    $AID docs "$f" --format html 2>/dev/null || true
    $AID docs "$f" --format json 2>/dev/null || true
done

# Webhook classifier
if [ -f examples/webhook-classifier/main.aid ]; then
    echo "  📄 Generating docs for webhook-classifier..."
    $AID docs examples/webhook-classifier/main.aid --format html 2>/dev/null || true
    $AID docs examples/webhook-classifier/main.aid --format json 2>/dev/null || true
fi

# Auto-commit if docs changed
if ! git diff --quiet docs/generated/ 2>/dev/null; then
    echo "  📝 Docs changed — auto-committing..."
    git add docs/generated/
    git commit -m "docs: auto-generated documentation update" --no-verify
    echo "  ✅ Docs committed"
else
    echo "  ✅ Docs up to date"
fi
