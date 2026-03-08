#!/bin/sh
set -e

systemctl stop hourglass 2>/dev/null || true
systemctl disable hourglass 2>/dev/null || true
rm -f /etc/systemd/system/hourglass.service
systemctl daemon-reload

rm -f /usr/local/bin/hourglass

# Remove PAM hook
PAM_FILE=/etc/pam.d/common-account
if [ -f "$PAM_FILE" ]; then
    sed -i '/hourglass/d' "$PAM_FILE"
    echo "PAM hook removed."
fi

echo "Hourglass uninstalled."
echo "Config and state left in /etc/hourglass and /var/lib/hourglass."
echo "Remove manually if desired."
