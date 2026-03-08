#!/bin/sh
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN="$SCRIPT_DIR/hourglass"
if [ ! -f "$BIN" ]; then
    BIN="$SCRIPT_DIR/target/release/hourglass"
fi
if [ ! -f "$BIN" ]; then
    echo "Binary not found. Run 'cargo build --release' or use package.sh first."
    exit 1
fi

cp "$BIN" /usr/local/bin/hourglass
chmod 755 /usr/local/bin/hourglass

mkdir -p /etc/hourglass /var/lib/hourglass

cp "$SCRIPT_DIR/config/hourglass.service" /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now hourglass

# Add PAM hook if not already present
PAM_LINE="account required pam_exec.so quiet /usr/local/bin/hourglass pam-check"
PAM_FILE=/etc/pam.d/common-account
if ! grep -qF "hourglass" "$PAM_FILE" 2>/dev/null; then
    echo "$PAM_LINE" >> "$PAM_FILE"
    echo "PAM hook added to $PAM_FILE"
else
    echo "PAM hook already present."
fi

echo "Hourglass installed."
