# Code signing and notarization

Current state: **no certificates configured.**

| Platform | Today | Effect for people installing |
|---|---|---|
| macOS | Ad-hoc signature (`APPLE_SIGNING_IDENTITY=-`), not notarized | Gatekeeper blocks the first launch; the user allows it in System Settings › Privacy & Security › "Open Anyway". Screen Recording/Microphone permissions are tied to the signature and may be asked again after each update. |
| Windows | No Authenticode signature | SmartScreen may warn ("More info › Run anyway") until the file gains reputation. |

The release workflow is ready: once the secrets below exist (*Settings › Secrets and variables ›
Actions*), it signs (and on macOS notarizes) with no other change. Updater signing is separate and
already required — see [release.md](release.md#signing-keys).

## macOS — Developer ID + notarization

Requires an Apple Developer Program membership.

| Secret | Contents |
|---|---|
| `APPLE_CERTIFICATE` | **Developer ID Application** certificate exported as `.p12`, base64 (`base64 -i cert.p12`) |
| `APPLE_CERTIFICATE_PASSWORD` | `.p12` password |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Name (TEAMID)` |
| `APPLE_ID` | Apple ID used for notarization |
| `APPLE_PASSWORD` | App-specific password for that Apple ID |
| `APPLE_TEAM_ID` | Team ID |

Tauri signs with the Hardened Runtime and `src-tauri/Entitlements.plist`, submits for notarization
and staples the ticket to the `.dmg`.

## Windows — Authenticode

Requires a code-signing certificate (OV/EV) from a CA or a cloud signing service (e.g. Azure
Trusted Signing). With a `.pfx`:

| Secret | Contents |
|---|---|
| `WINDOWS_CERTIFICATE` | `.pfx`, base64 |
| `WINDOWS_CERTIFICATE_PASSWORD` | `.pfx` password |

The *Sign Windows installer* step uses `signtool` with a SHA-256 timestamp. EV certificates on a
hardware token/HSM or cloud signing need that step adapted.

## Verification

Regardless of signing, every release publishes `SHA256SUMS.txt`, and the website shows each
installer's SHA-256.
