#!/usr/bin/env bash
#
# Restore the out-of-repo provisioning that arena-tests needs but does not track:
#   1. Sibling Reflective-Lab repos (cargo path deps + eager [patch.crates-io]).
#   2. The `arena` CLI workspace-root marker (/MASTERPLAN.md + /KB).
#   3. The KB applet manifests the `intent_codec_applets` smoke test include_str!s.
#
# These normally persist in the Cursor Cloud VM snapshot. Run this only if a
# fresh VM is missing them (e.g. `cargo` cannot resolve, or the arena CLI prints
# "could not locate reflective workspace root"). The script is idempotent: it
# clones only what is absent and always refreshes the markers/fixtures.
#
# Requires: `gh` authenticated, and `sudo` (the container dirs live at the
# filesystem root, the parent of /workspace).
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OWNER="$(id -un):$(id -gn)"

clone_if_missing() {
  local repo="$1" dest="$2"
  if [ -f "$dest/Cargo.toml" ]; then
    echo "ok    $dest"
  else
    echo "clone Reflective-Lab/$repo -> $dest"
    gh repo clone "Reflective-Lab/$repo" "$dest" -- --depth 1
  fi
}

echo "==> ensuring container dirs exist (needs sudo)"
sudo mkdir -p /bedrock-platform /mosaic-extensions /atelier-showcase /KB/02-product/applets
sudo chown -R "$OWNER" /bedrock-platform /mosaic-extensions /atelier-showcase /KB

echo "==> cloning sibling repos (skips ones already present)"
clone_if_missing converge          /bedrock-platform/converge
clone_if_missing organism          /bedrock-platform/organism
clone_if_missing helms             /bedrock-platform/helms
clone_if_missing arbiter-policy    /mosaic-extensions/arbiter-policy
clone_if_missing mnemos-knowledge  /mosaic-extensions/mnemos-knowledge
clone_if_missing prism-analytics   /mosaic-extensions/prism-analytics
clone_if_missing manifold-adapters /mosaic-extensions/manifold-adapters
clone_if_missing embassy-ports     /mosaic-extensions/embassy-ports
clone_if_missing atelier-showcase  /atelier-showcase

echo "==> writing arena workspace-root marker"
if [ ! -f /MASTERPLAN.md ]; then
  printf '# Reflective Workspace Root\n\nLocal marker so the `arena` CLI can locate the reflective workspace root.\nSibling repos: arena-tests (/workspace), bedrock-platform, mosaic-extensions, atelier-showcase.\n' \
    | sudo tee /MASTERPLAN.md >/dev/null
  sudo chown "$OWNER" /MASTERPLAN.md
fi

echo "==> installing KB applet fixtures"
cp "$REPO_DIR"/bootstrap/KB/02-product/applets/*.intent.json /KB/02-product/applets/

echo "==> done. Verify with: cargo test --workspace && cargo run --bin arena -- report"
