#!/usr/bin/env bash
# Generate a self-signed Windows code-signing certificate into this directory.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

PASS="$(openssl rand -base64 24 | tr -d '/+=' | head -c 24)"

openssl req -x509 -newkey rsa:4096 -sha256 -days 1825 -nodes \
  -keyout windows-codesign.key \
  -out windows-codesign.crt \
  -subj "/CN=CDXTheme/O=CDXTheme/C=US" \
  -addext "extendedKeyUsage=codeSigning" \
  -addext "keyUsage=digitalSignature"

openssl pkcs12 -export \
  -out windows-codesign.pfx \
  -inkey windows-codesign.key \
  -in windows-codesign.crt \
  -passout pass:"$PASS"

printf '%s\n' "$PASS" > windows-codesign.password
openssl base64 -A -in windows-codesign.pfx > windows-codesign.pfx.base64
openssl x509 -in windows-codesign.crt -fingerprint -sha1 -noout \
  | sed 's/^.*=//;s/://g' > windows-codesign.thumbprint

chmod 600 windows-codesign.key windows-codesign.pfx windows-codesign.password windows-codesign.pfx.base64
chmod 644 windows-codesign.crt windows-codesign.thumbprint

echo "Created Windows self-signed code-signing materials in $DIR"
echo "  pfx:         windows-codesign.pfx"
echo "  password:    windows-codesign.password"
echo "  base64:      windows-codesign.pfx.base64  (→ WINDOWS_CERTIFICATE secret)"
echo "  thumbprint:  $(cat windows-codesign.thumbprint)"
echo
echo "Usage:"
echo "  source .tauri/export-windows-signing.sh"
echo "  # then: cargo tauri build --target x86_64-pc-windows-msvc  (or CI with secrets)"
