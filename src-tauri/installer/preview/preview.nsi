; Preview of FrameLy's installer screens (installer/ui.nsh) with a simulated install, for
; screenshots on CI (.github/workflows/installer-ui.yml) and quick local checks:
;   makensis preview.nsi  →  framely-installer-preview.exe
;   framely-installer-preview.exe /PAGE=welcome|progress|finish /STATE=new|update|reinstall /LANG=en|pt
; Installing it (e.g. /S) writes uninstall.exe, which takes /PAGE=uninstall|removing|removed.
; The progress pages can't be skipped: /PAGE=progress|removing skips the first screen and shows
; the (simulated) progress, /PAGE=finish|removed also lets it run and then shows the last screen.
Unicode true
ManifestDPIAware true
ManifestDPIAwareness PerMonitorV2
SetCompressor /SOLID lzma
RequestExecutionLevel user

!define PRODUCTNAME "FrameLy"
!define VERSION "0.3.0"
!define UNINSTKEY "Software\FrameLyInstallerPreview"
!define FRAMELY_INSTALLER_DIR "${__FILEDIR__}\.."
!define MUI_ICON "${__FILEDIR__}\..\..\icons\icon.ico"
!define MUI_UNICON "${__FILEDIR__}\..\..\icons\icon.ico"

Name "${PRODUCTNAME}"
OutFile "framely-installer-preview.exe"
InstallDir "$LOCALAPPDATA\FrameLyInstallerPreview"

!include MUI2.nsh
!include FileFunc.nsh

Var PassiveMode
Var UpdateMode
Var PreviewPage

!macro FL_PREVIEW_PAGE NAME
  ${If} $PreviewPage != ""
  ${AndIf} $PreviewPage != "${NAME}"
    Abort
  ${EndIf}
!macroend

!include "${FRAMELY_INSTALLER_DIR}\ui.nsh"

Function RunMainBinary
FunctionEnd

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "PortugueseBR"
!include "${FRAMELY_INSTALLER_DIR}\lang\English.nsh"
!include "${FRAMELY_INSTALLER_DIR}\lang\PortugueseBR.nsh"

!macro PREVIEW_ARGS
  ${GetParameters} $0
  ClearErrors
  ${GetOptions} $0 "/PAGE=" $PreviewPage
  ${GetOptions} $0 "/LANG=" $1
  ${If} $1 == "pt"
    StrCpy $LANGUAGE ${LANG_PORTUGUESEBR}
  ${Else}
    StrCpy $LANGUAGE ${LANG_ENGLISH}
  ${EndIf}
!macroend

Function .onInit
  StrCpy $PassiveMode 0
  StrCpy $UpdateMode 0
  !insertmacro FL_INIT
  !insertmacro PREVIEW_ARGS
  ${GetParameters} $0
  ${GetOptions} $0 "/STATE=" $1
  DeleteRegKey HKCU "${UNINSTKEY}"
  ${If} $1 == "update"
    WriteRegStr HKCU "${UNINSTKEY}" "DisplayVersion" "0.2.0"
  ${ElseIf} $1 == "reinstall"
    WriteRegStr HKCU "${UNINSTKEY}" "DisplayVersion" "${VERSION}"
  ${EndIf}
FunctionEnd

Function un.onInit
  StrCpy $PassiveMode 0
  StrCpy $UpdateMode 0
  !insertmacro FL_INIT
  !insertmacro PREVIEW_ARGS
FunctionEnd

!macro SLOW_STEP
  Sleep 400
!macroend

Section
  SetOutPath $INSTDIR
  DetailPrint "Extract: framely.exe"
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  DetailPrint "Extract: framely-llm.exe"
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  DetailPrint "Extract: resources"
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section Uninstall
  DetailPrint "Remove: framely.exe"
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  !insertmacro SLOW_STEP
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  SetAutoClose true
SectionEnd
