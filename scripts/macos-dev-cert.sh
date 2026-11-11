#!/usr/bin/env bash
# Creates a stable, self-signed code-signing identity "FrameLy Development" in your login
# keychain. macOS ties Screen Recording / Microphone permissions to the app's signature, so
# signing every dev build with the same identity keeps permissions across rebuilds.
#
# Run once:   ./scripts/macos-dev-cert.sh
# macOS will ask for your login password to trust the certificate for code signing.
# Remove:     security delete-identity -c "FrameLy Development"
set -euo pipefail

NAME="FrameLy Development"
if security find-identity -v -p codesigning | grep -q "\"$NAME\""; then
  echo "✓ '$NAME' already exists and is valid for code signing."
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/cert.cnf" <<CNF
[ req ]
distinguished_name = dn
x509_extensions = ext
prompt = no
[ dn ]
CN = $NAME
[ ext ]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
CNF

openssl req -x509 -newkey rsa:2048 -nodes -days 3650 \
  -keyout "$TMP/key.pem" -out "$TMP/cert.pem" -config "$TMP/cert.cnf" >/dev/null 2>&1
P12_PASS="framely-dev"
openssl pkcs12 -export -legacy -inkey "$TMP/key.pem" -in "$TMP/cert.pem" \
  -out "$TMP/id.p12" -passout "pass:$P12_PASS" -name "$NAME" >/dev/null 2>&1 \
  || openssl pkcs12 -export -inkey "$TMP/key.pem" -in "$TMP/cert.pem" \
       -out "$TMP/id.p12" -passout "pass:$P12_PASS" -name "$NAME" >/dev/null

KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"
security import "$TMP/id.p12" -k "$KEYCHAIN" -P "$P12_PASS" -T /usr/bin/codesign >/dev/null
echo "→ Trusting the certificate for code signing (macOS will ask for your password)…"
security add-trusted-cert -r trustRoot -p codeSign -k "$KEYCHAIN" "$TMP/cert.pem"

security find-identity -v -p codesigning | grep "\"$NAME\"" >/dev/null \
  && echo "✓ '$NAME' is ready. Build with: ./scripts/macos-dev-app.sh" \
  || { echo "✗ Identity not valid for code signing."; exit 1; }
