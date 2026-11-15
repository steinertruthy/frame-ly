# Assinatura de código e notarização

Estado atual (v0.1.0): **nenhum certificado configurado**.

| Plataforma | Hoje | Efeito para quem instala |
|---|---|---|
| macOS | Assinatura *ad hoc* (`APPLE_SIGNING_IDENTITY=-`), sem notarização | O Gatekeeper bloqueia a primeira abertura; o usuário libera em Ajustes do Sistema › Privacidade e Segurança › “Abrir Mesmo Assim”. As permissões de Gravação de Tela/Microfone ficam atreladas à assinatura e podem ser pedidas de novo a cada atualização. |
| Windows | Sem assinatura Authenticode | O SmartScreen pode avisar (“Mais informações › Executar assim mesmo”) até o arquivo ganhar reputação. |

O workflow de release já está preparado: quando os *secrets* abaixo existirem no repositório
(*Settings › Secrets and variables › Actions*), ele assina (e no macOS notariza) sem outras mudanças.

## macOS — Developer ID + notarização

Requer conta no Apple Developer Program (pago, anual).

| Secret | Conteúdo |
|---|---|
| `APPLE_CERTIFICATE` | Certificado **Developer ID Application** exportado como `.p12`, em base64 (`base64 -i cert.p12`) |
| `APPLE_CERTIFICATE_PASSWORD` | Senha do `.p12` |
| `APPLE_SIGNING_IDENTITY` | Ex.: `Developer ID Application: Nome (TEAMID)` |
| `APPLE_ID` | Apple ID usado na notarização |
| `APPLE_PASSWORD` | Senha de app específica desse Apple ID |
| `APPLE_TEAM_ID` | Team ID |

O Tauri assina com *hardened runtime* e `src-tauri/Entitlements.plist`, envia para o serviço de
notarização e grampeia o ticket no `.dmg`.

## Windows — Authenticode

Requer um certificado de assinatura de código (OV/EV) de uma autoridade certificadora ou um serviço
de assinatura em nuvem (ex.: Azure Trusted Signing). Com um `.pfx`:

| Secret | Conteúdo |
|---|---|
| `WINDOWS_CERTIFICATE` | `.pfx` em base64 |
| `WINDOWS_CERTIFICATE_PASSWORD` | Senha do `.pfx` |

O passo *Sign Windows installer* usa `signtool` com carimbo de tempo SHA-256. Certificados EV em
token físico/HSM ou assinatura em nuvem exigem adaptar esse passo.

## Verificação

Independente da assinatura, cada release publica `SHA256SUMS.txt`, e o site mostra o SHA-256 de
cada instalador.
