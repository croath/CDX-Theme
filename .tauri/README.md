# Local signing material

## Windows Authenticode (self-signed)

| File | Purpose |
|------|---------|
| `windows-codesign.pfx` | PKCS#12 cert + key (for `signtool` / CI) |
| `windows-codesign.password` | PFX password |
| `windows-codesign.pfx.base64` | Base64 of the PFX (GitHub secret `WINDOWS_CERTIFICATE`) |
| `windows-codesign.crt` | Public certificate only |
| `windows-codesign.key` | Private key (PEM) |
| `windows-codesign.thumbprint` | SHA-1 thumbprint (Windows cert store) |

**Do not commit** `.key`, `.pfx`, `.password`, or `.pfx.base64`.

### Local / CI env (Tauri 2)

```bash
# From repo root
export WINDOWS_CERTIFICATE="$(cat .tauri/windows-codesign.pfx.base64)"
export WINDOWS_CERTIFICATE_PASSWORD="$(tr -d '\n' < .tauri/windows-codesign.password)"
```

Or helper:

```bash
source .tauri/export-windows-signing.sh
```

Tauri reads those variables when bundling Windows installers.  
`app-tauri/tauri.conf.json` sets `digestAlgorithm` + `timestampUrl` under `bundle.windows`.

### Windows host: install cert for thumbprint signing

1. Copy `windows-codesign.pfx` to the Windows machine.
2. Double-click → import into **Current User → Personal**.
3. Trust (optional, test PCs only): export `.crt` → Trusted Root.
4. Confirm thumbprint matches `windows-codesign.thumbprint`.

Self-signed certs **do not** clear SmartScreen for end users; use a CA-issued code-signing cert for public releases.

### Regenerate

```bash
# From repo root (macOS/Linux with OpenSSL)
./.tauri/generate-windows-codesign.sh
```
