; FrameLy's own installer and uninstaller screens.
;
; Full-window nsDialogs pages drawn in the app's colours (src/lib/design/tokens.css), replacing
; the standard MUI pages. Included by installer.nsi (the Tauri template FrameLy ships) and by
; preview/preview.nsi (screenshots on CI). All install logic stays in installer.nsi: these pages
; only collect choices and show progress, so silent (/S), passive (/P) and update (/UPDATE)
; installs behave exactly like Tauri's.
;
; Needs (declared by the includer): $PassiveMode, $UpdateMode, function RunMainBinary,
; ${PRODUCTNAME}, ${VERSION}, ${UNINSTKEY}, ${FRAMELY_INSTALLER_DIR} (folder of logo.bmp).

!include nsDialogs.nsh
!include LogicLib.nsh
!include WinMessages.nsh
!include WordFunc.nsh

; Colours (RRGGBB) from the app's dark theme.
!define FL_BG "0B0E14"
!define FL_SURFACE "11151F"
!define FL_TEXT "EDF0F7"
!define FL_TEXT2 "8B95A8"
!define FL_PRIMARY "1F3459"
!define FL_PRIMARY_TEXT "E6EDFA"
!define FL_SECONDARY "171D2A"
!define FL_LINK "8BB0EE"
; COLORREF (0x00BBGGRR) for the progress bar.
!define FL_BAR 0x00D97B4C
!define FL_BAR_BG 0x003D271B

!define /ifndef SS_CENTER 0x00000001
!define /ifndef SS_NOTIFY 0x00000100
!define /ifndef SS_CENTERIMAGE 0x00000200
!define /ifndef SS_PATHELLIPSIS 0x00008000
!define /ifndef PBM_SETBARCOLOR 0x0409
!define /ifndef PBM_SETBKCOLOR 0x2001

; preview/preview.nsi defines this to open one screen directly; a no-op in the real installer.
!ifmacrondef FL_PREVIEW_PAGE
  !macro FL_PREVIEW_PAGE NAME
  !macroend
!endif

Var FlFontTitle
Var FlFontBody
Var FlFontSmall
Var FlFontButton
Var FlDialog
Var FlDesktop        ; 1 = create a desktop shortcut
Var FlDesktopBox
Var FlRunNow         ; 1 = open FrameLy after installing
Var FlRunBox
Var FlState          ; new | update | reinstall | downgrade
Var FlOldVersion
Var FlPathLabel
Var FlChangeLink
Var FlLogo
Var DeleteAppDataCheckbox
Var DeleteAppDataCheckboxState

; Shared helpers, defined once for the installer and once (un.) for the uninstaller.
!macro FL_HELPERS UN

; Scales a 96-DPI pixel value on the stack to the window's DPI.
Function ${UN}FlScale
  Exch $0
  Push $1
  System::Call "user32::GetDpiForWindow(p $HWNDPARENT) i .r1"
  ${If} $1 = 0
    StrCpy $1 96
  ${EndIf}
  IntOp $0 $0 * $1
  IntOp $0 $0 / 96
  Pop $1
  Exch $0
FunctionEnd

; Hides the standard frame (header, branding, separators, Back/Next/Cancel). The hidden
; buttons keep working: Enter and Esc still reach them, and FrameLy's buttons press them.
Function ${UN}FlHideChrome
  Push $0
  Push $1
  ${ForEach} $0 1 3 + 1
    GetDlgItem $1 $HWNDPARENT $0
    ShowWindow $1 ${SW_HIDE}
  ${Next}
  ${ForEach} $0 1028 1045 + 1
    GetDlgItem $1 $HWNDPARENT $0
    ShowWindow $1 ${SW_HIDE}
  ${Next}
  GetDlgItem $1 $HWNDPARENT 1256
  ShowWindow $1 ${SW_HIDE}
  ; Dark title bar on Windows 10 2004+ / 11 (ignored elsewhere).
  System::Call "dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i 20, *i 1, i 4)"
  Pop $1
  Pop $0
FunctionEnd

; Makes the window on the stack cover the whole client area of the installer window.
Function ${UN}FlFullWindow
  Exch $0
  Push $1
  Push $2
  Push $3
  System::Call "*(i,i,i,i)p.r1"
  System::Call "user32::GetClientRect(p $HWNDPARENT, p r1)"
  System::Call "*$1(i,i,i.r2,i.r3)"
  System::Free $1
  System::Call "user32::SetWindowPos(p r0, p 0, i 0, i 0, i r2, i r3, i 0x14)"
  SetCtlColors $0 "" ${FL_BG}
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; Rounds the corners of the control on the stack (8 px at 96 DPI), like the app's buttons.
Function ${UN}FlRound
  Exch $0
  Push $1
  Push $2
  Push $3
  Push $4
  System::Call "*(i,i,i,i)p.r1"
  System::Call "user32::GetClientRect(p r0, p r1)"
  System::Call "*$1(i,i,i.r2,i.r3)"
  System::Free $1
  IntOp $2 $2 + 1
  IntOp $3 $3 + 1
  Push 9
  Call ${UN}FlScale
  Pop $4
  System::Call "gdi32::CreateRoundRectRgn(i 0, i 0, i r2, i r3, i r4, i r4) p.r1"
  System::Call "user32::SetWindowRgn(p r0, p r1, i 1)"
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

Function ${UN}FlFonts
  ${If} $FlFontTitle == ""
    CreateFont $FlFontTitle "Segoe UI Semibold" 17 600
    CreateFont $FlFontBody "Segoe UI" 10 400
    CreateFont $FlFontSmall "Segoe UI" 9 400
    CreateFont $FlFontButton "Segoe UI Semibold" 10 600
  ${EndIf}
FunctionEnd

; A new full-window page with the logo at the top left. Leaves the dialog in $FlDialog.
Function ${UN}FlPage
  Call ${UN}FlFonts
  Call ${UN}FlHideChrome
  nsDialogs::Create 1044
  Pop $FlDialog
  ${If} $FlDialog == error
    Abort
  ${EndIf}
  Push $FlDialog
  Call ${UN}FlFullWindow
  ${NSD_CreateBitmap} 28u 24u 30u 30u ""
  Pop $0
  ${NSD_SetStretchedImage} $0 "$PLUGINSDIR\framely-logo.bmp" $FlLogo
FunctionEnd

; Pressing FrameLy's buttons presses the hidden standard ones (Next = 1, Cancel = 2).
Function ${UN}FlNext
  Pop $0
  SendMessage $HWNDPARENT ${WM_COMMAND} 1 0
FunctionEnd
Function ${UN}FlCancel
  Pop $0
  SendMessage $HWNDPARENT ${WM_COMMAND} 2 0
FunctionEnd

!macroend

!insertmacro FL_HELPERS ""
!insertmacro FL_HELPERS "un."

; Text: x y w h text font colour
!macro FL_TEXT X Y W H TEXT FONT COLOR
  ${NSD_CreateLabel} ${X} ${Y} ${W} ${H} "${TEXT}"
  Pop $0
  SendMessage $0 ${WM_SETFONT} ${FONT} 0
  SetCtlColors $0 ${COLOR} ${FL_BG}
!macroend

; Flat, rounded button in the app's primary or secondary style.
!macro FL_BUTTON UN X Y W H TEXT KIND CALLBACK
  nsDialogs::CreateControl STATIC ${__NSD_Label_STYLE}|${SS_CENTER}|${SS_CENTERIMAGE}|${SS_NOTIFY} 0 ${X} ${Y} ${W} ${H} "${TEXT}"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FlFontButton 0
  !if "${KIND}" == "primary"
    SetCtlColors $0 ${FL_PRIMARY_TEXT} ${FL_PRIMARY}
  !else
    SetCtlColors $0 ${FL_TEXT} ${FL_SECONDARY}
  !endif
  Push $0
  Call ${UN}FlRound
  ${NSD_OnClick} $0 ${CALLBACK}
!macroend

; Checkbox with a separate label, so the text keeps the dark theme's colour.
!macro FL_CHECK X LX Y TEXT VAR STATE TOGGLE
  ${NSD_CreateCheckbox} ${X} ${Y} 10u 10u ""
  Pop ${VAR}
  SetCtlColors ${VAR} "" ${FL_BG}
  ${If} ${STATE} = 1
    ${NSD_Check} ${VAR}
  ${EndIf}
  ${NSD_CreateLabel} ${LX} ${Y} -70u 11u "${TEXT}"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FlFontBody 0
  SetCtlColors $0 ${FL_TEXT} ${FL_BG}
  ${NSD_AddStyle} $0 ${SS_NOTIFY}
  ${NSD_OnClick} $0 ${TOGGLE}
!macroend

; ───────────────────────────── Installer ─────────────────────────────

Function FlDetectState
  StrCpy $FlState "new"
  ReadRegStr $FlOldVersion SHCTX "${UNINSTKEY}" "DisplayVersion"
  ${If} $FlOldVersion != ""
    ${VersionCompare} "${VERSION}" "$FlOldVersion" $0
    ${If} $0 = 0
      StrCpy $FlState "reinstall"
    ${ElseIf} $0 = 1
      StrCpy $FlState "update"
    ${Else}
      StrCpy $FlState "downgrade"
    ${EndIf}
  ${EndIf}
FunctionEnd

Function FlToggleDesktop
  Pop $0
  ${NSD_GetState} $FlDesktopBox $1
  ${If} $1 = ${BST_CHECKED}
    ${NSD_Uncheck} $FlDesktopBox
  ${Else}
    ${NSD_Check} $FlDesktopBox
  ${EndIf}
FunctionEnd

Function FlChangeFolder
  Pop $0
  nsDialogs::SelectFolderDialog "$(flLocation)" "$INSTDIR"
  Pop $0
  ${If} $0 != error
    ${WordFind} "$0" "\" "-1" $1
    ${If} $1 != "${PRODUCTNAME}"
      StrCpy $0 "$0\${PRODUCTNAME}"
    ${EndIf}
    StrCpy $INSTDIR $0
    ${NSD_SetText} $FlPathLabel $INSTDIR
  ${EndIf}
FunctionEnd

Function FlWelcome
  !insertmacro FL_PREVIEW_PAGE "welcome"
  ${IfThen} $PassiveMode = 1 ${|} Abort ${|}
  Call FlDetectState
  Call FlPage

  ${If} $FlState == "update"
    StrCpy $1 "$(flUpdateTitle)"
    StrCpy $2 "$(flUpdateText)"
    StrCpy $3 "$(flUpdate)"
  ${ElseIf} $FlState == "reinstall"
    StrCpy $1 "$(flReinstallTitle)"
    StrCpy $2 "$(flReinstallText)"
    StrCpy $3 "$(flInstall)"
  ${ElseIf} $FlState == "downgrade"
    StrCpy $1 "$(flDowngradeTitle)"
    StrCpy $2 "$(flDowngradeText)"
    StrCpy $3 "$(flInstall)"
  ${Else}
    StrCpy $1 "$(flWelcomeTitle)"
    StrCpy $2 "$(flWelcomeText)"
    StrCpy $3 "$(flInstall)"
  ${EndIf}

  !insertmacro FL_TEXT 28u 66u -56u 22u "$1" $FlFontTitle ${FL_TEXT}
  !insertmacro FL_TEXT 28u 90u -68u 44u "$2" $FlFontBody ${FL_TEXT2}

  ; Where it goes (fixed when updating an existing installation). No overlapping controls: a
  ; caption, the path and a "Change" link on the right.
  ${NSD_CreateLabel} 28u 134u 150u 10u "$(flLocation)"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FlFontSmall 0
  SetCtlColors $0 ${FL_TEXT2} ${FL_BG}
  nsDialogs::CreateControl STATIC ${__NSD_Label_STYLE}|${SS_PATHELLIPSIS} 0 28u 145u -132u 12u "$INSTDIR"
  Pop $FlPathLabel
  SendMessage $FlPathLabel ${WM_SETFONT} $FlFontBody 0
  SetCtlColors $FlPathLabel ${FL_TEXT} ${FL_BG}
  ${If} $FlState == "new"
    nsDialogs::CreateControl STATIC ${__NSD_Label_STYLE}|${SS_RIGHT} 0 -96u 145u 68u 12u "$(flChange)"
    Pop $FlChangeLink
    SendMessage $FlChangeLink ${WM_SETFONT} $FlFontButton 0
    SetCtlColors $FlChangeLink ${FL_LINK} ${FL_BG}
    ${NSD_OnClick} $FlChangeLink FlChangeFolder
  ${EndIf}

  ${If} $FlState == "new"
    StrCpy $FlDesktop 1
  ${EndIf}
  ${If} $UpdateMode <> 1
    !insertmacro FL_CHECK 28u 42u 168u "$(flDesktop)" $FlDesktopBox $FlDesktop FlToggleDesktop
  ${EndIf}

  !insertmacro FL_BUTTON "" -168u -34u 64u 20u "$(flCancel)" secondary FlCancel
  !insertmacro FL_BUTTON "" -96u -34u 68u 20u "$3" primary FlNext

  nsDialogs::Show
  ${NSD_FreeImage} $FlLogo
FunctionEnd

Function FlWelcomeLeave
  ${If} $FlDesktopBox != ""
    ${NSD_GetState} $FlDesktopBox $0
    ${If} $0 = ${BST_CHECKED}
      StrCpy $FlDesktop 1
    ${Else}
      StrCpy $FlDesktop 0
    ${EndIf}
  ${EndIf}
FunctionEnd

; Restyles the standard progress page: title, status line and a slim progress bar in the app's
; accent; the detail list stays hidden unless something fails (.onInstFailed shows it).
!macro FL_PROGRESS_SHOW UN TITLE
  Call ${UN}FlFonts
  Call ${UN}FlHideChrome
  FindWindow $R0 "#32770" "" $HWNDPARENT
  Push $R0
  Call ${UN}FlFullWindow

  GetDlgItem $R1 $R0 1016
  ShowWindow $R1 ${SW_HIDE}
  GetDlgItem $R1 $R0 1027
  ShowWindow $R1 ${SW_HIDE}

  ; Layout at 96 DPI, scaled: x = 42, title y = 120, status y = 168, bar y = 196.
  Push 42
  Call ${UN}FlScale
  Pop $R2
  Push 420
  Call ${UN}FlScale
  Pop $R3

  Push 120
  Call ${UN}FlScale
  Pop $R4
  Push 34
  Call ${UN}FlScale
  Pop $R5
  IntOp $R8 ${WS_CHILD} | ${WS_VISIBLE}
  System::Call 'user32::CreateWindowEx(i 0, w "STATIC", w "${TITLE}", i R8, i R2, i R4, i R3, i R5, p R0, i 0, i 0, i 0) p .R6'
  SendMessage $R6 ${WM_SETFONT} $FlFontTitle 1
  SetCtlColors $R6 ${FL_TEXT} ${FL_BG}

  Push 52
  Call ${UN}FlScale
  Pop $R4
  Push 48
  Call ${UN}FlScale
  Pop $R5
  IntOp $R8 $R8 | 0x0E ; SS_BITMAP
  System::Call 'user32::CreateWindowEx(i 0, w "STATIC", w "", i R8, i R2, i R4, i R5, i R5, p R0, i 0, i 0, i 0) p .R6'
  System::Call 'user32::LoadImage(p 0, w "$PLUGINSDIR\framely-logo.bmp", i 0, i R5, i R5, i 0x10) p .R7'
  SendMessage $R6 0x0172 0 $R7

  GetDlgItem $R1 $R0 1006
  Push 168
  Call ${UN}FlScale
  Pop $R4
  Push 22
  Call ${UN}FlScale
  Pop $R5
  System::Call "user32::SetWindowPos(p R1, p 0, i R2, i R4, i R3, i R5, i 0x14)"
  SendMessage $R1 ${WM_SETFONT} $FlFontBody 1
  SetCtlColors $R1 ${FL_TEXT2} ${FL_BG}

  GetDlgItem $R1 $R0 1004
  Push 200
  Call ${UN}FlScale
  Pop $R4
  Push 6
  Call ${UN}FlScale
  Pop $R5
  System::Call "user32::SetWindowPos(p R1, p 0, i R2, i R4, i R3, i R5, i 0x14)"
  System::Call 'uxtheme::SetWindowTheme(p R1, w " ", w " ")'
  SendMessage $R1 ${PBM_SETBARCOLOR} 0 ${FL_BAR}
  SendMessage $R1 ${PBM_SETBKCOLOR} 0 ${FL_BAR_BG}
!macroend

Function FlProgressShow
  !insertmacro FL_PROGRESS_SHOW "" "$(flInstalling)"
FunctionEnd

Function FlToggleRun
  Pop $0
  ${NSD_GetState} $FlRunBox $1
  ${If} $1 = ${BST_CHECKED}
    ${NSD_Uncheck} $FlRunBox
  ${Else}
    ${NSD_Check} $FlRunBox
  ${EndIf}
FunctionEnd

Function FlFinish
  !insertmacro FL_PREVIEW_PAGE "finish"
  ${IfThen} $PassiveMode = 1 ${|} Abort ${|}
  Call FlPage
  !insertmacro FL_TEXT 28u 66u -56u 22u "$(framelyFinishTitle)" $FlFontTitle ${FL_TEXT}
  !insertmacro FL_TEXT 28u 90u -68u 36u "$(framelyFinishText)" $FlFontBody ${FL_TEXT2}
  StrCpy $FlRunNow 1
  !insertmacro FL_CHECK 28u 42u 134u "$(framelyRunNow)" $FlRunBox $FlRunNow FlToggleRun
  !insertmacro FL_BUTTON "" -96u -34u 68u 20u "$(flFinish)" primary FlNext
  nsDialogs::Show
  ${NSD_FreeImage} $FlLogo
FunctionEnd

Function FlFinishLeave
  ${NSD_GetState} $FlRunBox $0
  ${If} $0 = ${BST_CHECKED}
    Call RunMainBinary
  ${EndIf}
FunctionEnd

; If something fails, show what happened and a way out.
Function .onInstFailed
  FindWindow $0 "#32770" "" $HWNDPARENT
  GetDlgItem $1 $0 1016
  ShowWindow $1 ${SW_SHOW}
  GetDlgItem $1 $HWNDPARENT 2
  ShowWindow $1 ${SW_SHOW}
FunctionEnd

; ──────────────────────────── Uninstaller ────────────────────────────

Function un.FlToggleData
  Pop $0
  ${NSD_GetState} $DeleteAppDataCheckbox $1
  ${If} $1 = ${BST_CHECKED}
    ${NSD_Uncheck} $DeleteAppDataCheckbox
  ${Else}
    ${NSD_Check} $DeleteAppDataCheckbox
  ${EndIf}
FunctionEnd

Function un.FlConfirm
  !insertmacro FL_PREVIEW_PAGE "uninstall"
  ${IfThen} $PassiveMode = 1 ${|} Abort ${|}
  Call un.FlPage
  !insertmacro FL_TEXT 28u 66u -56u 22u "$(flUninstallTitle)" $FlFontTitle ${FL_TEXT}
  !insertmacro FL_TEXT 28u 90u -68u 36u "$(flUninstallText)" $FlFontBody ${FL_TEXT2}
  StrCpy $DeleteAppDataCheckboxState 0
  ${NSD_CreateCheckbox} 28u 134u 10u 10u ""
  Pop $DeleteAppDataCheckbox
  SetCtlColors $DeleteAppDataCheckbox "" ${FL_BG}
  ${NSD_CreateLabel} 42u 134u -82u 22u "$(deleteAppData)"
  Pop $0
  SendMessage $0 ${WM_SETFONT} $FlFontBody 0
  SetCtlColors $0 ${FL_TEXT} ${FL_BG}
  ${NSD_AddStyle} $0 ${SS_NOTIFY}
  ${NSD_OnClick} $0 un.FlToggleData
  !insertmacro FL_BUTTON "un." -168u -34u 64u 20u "$(flCancel)" secondary un.FlCancel
  !insertmacro FL_BUTTON "un." -96u -34u 68u 20u "$(flUninstall)" primary un.FlNext
  nsDialogs::Show
  ${NSD_FreeImage} $FlLogo
FunctionEnd

Function un.FlConfirmLeave
  ${NSD_GetState} $DeleteAppDataCheckbox $0
  ${If} $0 = ${BST_CHECKED}
    StrCpy $DeleteAppDataCheckboxState 1
  ${Else}
    StrCpy $DeleteAppDataCheckboxState 0
  ${EndIf}
FunctionEnd

Function un.FlProgressShow
  !insertmacro FL_PROGRESS_SHOW "un." "$(flRemoving)"
FunctionEnd

Function un.FlDone
  !insertmacro FL_PREVIEW_PAGE "removed"
  ${IfThen} $PassiveMode = 1 ${|} Abort ${|}
  ${IfThen} $UpdateMode = 1 ${|} Abort ${|}
  Call un.FlPage
  !insertmacro FL_TEXT 28u 66u -56u 22u "$(flRemovedTitle)" $FlFontTitle ${FL_TEXT}
  !insertmacro FL_TEXT 28u 90u -68u 36u "$(flRemovedText)" $FlFontBody ${FL_TEXT2}
  !insertmacro FL_BUTTON "un." -96u -34u 68u 20u "$(flClose)" primary un.FlNext
  nsDialogs::Show
  ${NSD_FreeImage} $FlLogo
FunctionEnd

; ─────────────────────────────── Pages ───────────────────────────────
; The instfiles pages move on by themselves to the last screen.
AutoCloseWindow true

Page custom FlWelcome FlWelcomeLeave
!define MUI_PAGE_CUSTOMFUNCTION_SHOW FlProgressShow
!insertmacro MUI_PAGE_INSTFILES
Page custom FlFinish FlFinishLeave

UninstPage custom un.FlConfirm un.FlConfirmLeave
!define MUI_PAGE_CUSTOMFUNCTION_SHOW un.FlProgressShow
!insertmacro MUI_UNPAGE_INSTFILES
UninstPage custom un.FlDone

; Puts the logo where the pages read it (called from .onInit / un.onInit).
!macro FL_INIT
  InitPluginsDir
  File "/oname=$PLUGINSDIR\framely-logo.bmp" "${FRAMELY_INSTALLER_DIR}\logo.bmp"
!macroend
