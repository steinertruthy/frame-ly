; FrameLy additions to the NSIS installer (bundle > windows > nsis > installerHooks), included
; near the top of installer/installer.nsi.

; Where ui.nsh and logo.bmp live (installer.nsi itself is rendered into target/).
!define FRAMELY_INSTALLER_DIR "${__FILEDIR__}"

; The local AI worker runs as a separate process (framely-llm.exe). Tauri's installer closes
; FrameLy itself; this also stops a worker left behind, which would lock its file on update.
!macro FRAMELY_STOP_WORKER
  nsExec::Exec 'taskkill /F /IM framely-llm.exe'
  Pop $0
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro FRAMELY_STOP_WORKER
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro FRAMELY_STOP_WORKER
!macroend

; "Delete the application data" also removes what FrameLy keeps in the Windows Credential
; Manager (src-tauri/src/platform/windows/keystore.rs: "FrameLy/ai/<account>"). Never on updates.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    nsExec::Exec 'cmdkey /delete:FrameLy/ai/anthropic-api-key'
    Pop $0
    nsExec::Exec 'cmdkey /delete:FrameLy/ai/openai-api-key'
    Pop $0
    nsExec::Exec 'cmdkey /delete:FrameLy/ai/framely-cloud-device-v2'
    Pop $0
  ${EndIf}
!macroend
