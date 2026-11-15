# Security

## Reporting a vulnerability

Please report security issues privately through GitHub:
**Security › Report a vulnerability** in this repository
(<https://github.com/steinertruthy/frame-ly/security/advisories/new>). Don't open a public issue for
them. You'll get an answer as soon as possible; fixes ship in a new release noted in `CHANGELOG.md`.

## Supported versions

Only the latest release receives fixes.

## Verifying downloads

Every release publishes `SHA256SUMS.txt`. Compare it with your file:

```bash
shasum -a 256 -c SHA256SUMS.txt --ignore-missing   # macOS / Linux
```

```powershell
Get-FileHash .\FrameLy_<version>_x64-setup.exe      # Windows
```

Installers are not yet signed with an Apple Developer ID or a Windows code-signing certificate —
see [docs/signing.md](docs/signing.md).
