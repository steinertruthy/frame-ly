; FrameLy additions to Tauri's NSIS installer (bundle > windows > nsis > installerHooks).
; Included near the top of Tauri's installer.nsi, before the pages are declared.

; Welcome and Finish pages in FrameLy's words (strings in installer/lang/*.nsh).
!define MUI_WELCOMEPAGE_TITLE "$(framelyWelcomeTitle)"
!define MUI_WELCOMEPAGE_TEXT "$(framelyWelcomeText)"
!define MUI_FINISHPAGE_TITLE "$(framelyFinishTitle)"
!define MUI_FINISHPAGE_TEXT "$(framelyFinishText)"
!define MUI_FINISHPAGE_RUN_TEXT "$(framelyRunNow)"

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
