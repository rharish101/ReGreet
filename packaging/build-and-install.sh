#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGE="localhost/regreet-builder:latest"
OUTPUT_DIR="$SCRIPT_DIR/output"

mkdir -p "$OUTPUT_DIR"

echo "==> Building regreet image (first run ~10-15 minutes for cargo deps)..."
podman build \
    --network=host \
    --pull=missing \
    --tag "$IMAGE" \
    --file "$SCRIPT_DIR/Containerfile" \
    "$REPO_ROOT"

echo "==> Extracting RPM from container..."
podman run --rm \
    -v "$OUTPUT_DIR:/out:z" \
    "$IMAGE" \
    bash -c 'cp /output/*.rpm /out/'

RPM=$(ls -t "$OUTPUT_DIR"/regreet*.rpm 2>/dev/null | head -1)
if [[ -z "$RPM" ]]; then
    echo "ERROR: No RPM found in $OUTPUT_DIR"
    exit 1
fi
echo "==> Built: $RPM"

echo "==> Installing greetd and cage from Fedora repos..."
sudo dnf install -y greetd greetd-selinux cage

echo "==> Installing ReGreet RPM..."
sudo rpm -Uvh --force "$RPM"

echo "==> Installing systemd-tmpfiles config (log/state dirs for greetd user)..."
sudo cp "$REPO_ROOT/systemd-tmpfiles.conf" /etc/tmpfiles.d/regreet.conf
sudo systemd-tmpfiles --create /etc/tmpfiles.d/regreet.conf

echo ""
echo "==> Done. Next steps:"
echo "    1. Configure greetd:  sudo cp $SCRIPT_DIR/config/greetd.toml /etc/greetd/config.toml"
echo "    2. Configure regreet: sudo cp $SCRIPT_DIR/config/regreet.toml /etc/greetd/regreet.toml"
echo "    3. Install theme CSS: sudo cp $SCRIPT_DIR/config/regreet.css /etc/greetd/regreet.css"
echo "    4. Copy wallpaper:    sudo cp ~/Pictures/wallpapers/<file> /usr/share/greetd/wallpaper.png"
echo "                          sudo chmod 644 /usr/share/greetd/wallpaper.png"
echo "    5. Test before switching: sudo systemctl start greetd (on a free VT)"
echo "    6. sudo systemctl disable sddm && sudo systemctl enable greetd"
