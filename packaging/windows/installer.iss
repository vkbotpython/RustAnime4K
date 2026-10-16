; Anime4K Player — Inno Setup installer (полный визард: согласие с лицензией
; GPL-3.0, выбор каталога, меню «Пуск», ярлык на рабочем столе, ассоциации
; медиафайлов, запуск после установки).
; Локально: .\build.ps1 -Installer (версия подставляется из Cargo.toml
; через /DMyAppVersion). Вручную: cargo build --release, затем ISCC на этом файле.
; For --features video builds, place mpv.lib/mpv.dll next to the exe
; (Linux CI uses libmpv-dev; see README).

#define MyAppName "Anime4K Player"
#ifndef MyAppVersion
#define MyAppVersion "1.0.1"
#endif
#define MyAppPublisher "avencores"
#define MyAppURL "https://gitlab.com/avencores/RustAnime4KPlayer"
#define MyAppExeName "anime4k-player.exe"
#define MyAppAssocName MyAppName + " File"

#define SrcDir "..\.."

#define mp4AssocExt ".mp4"
#define mp4AssocKey StringChange(MyAppAssocName, " ", "") + mp4AssocExt

#define mkvAssocExt ".mkv"
#define mkvAssocKey StringChange(MyAppAssocName, " ", "") + mkvAssocExt

#define m4aAssocExt ".m4a"
#define m4aAssocKey StringChange(MyAppAssocName, " ", "") + m4aAssocExt

#define webmAssocExt ".webm"
#define webmAssocKey StringChange(MyAppAssocName, " ", "") + webmAssocExt

#define mp3AssocExt ".mp3"
#define mp3AssocKey StringChange(MyAppAssocName, " ", "") + mp3AssocExt

#define tsAssocExt ".ts"
#define tsAssocKey StringChange(MyAppAssocName, " ", "") + tsAssocExt

#define movAssocExt ".mov"
#define movAssocKey StringChange(MyAppAssocName, " ", "") + movAssocExt

#define aviAssocExt ".avi"
#define aviAssocKey StringChange(MyAppAssocName, " ", "") + aviAssocExt

#define wavAssocExt ".wav"
#define wavAssocKey StringChange(MyAppAssocName, " ", "") + wavAssocExt

#define oggAssocExt ".ogg"
#define oggAssocKey StringChange(MyAppAssocName, " ", "") + oggAssocExt

#define flacAssocExt ".flac"
#define flacAssocKey StringChange(MyAppAssocName, " ", "") + flacAssocExt

#define aacAssocExt ".aac"
#define aacAssocKey StringChange(MyAppAssocName, " ", "") + aacAssocExt

#define opusAssocExt ".opus"
#define opusAssocKey StringChange(MyAppAssocName, " ", "") + opusAssocExt

#define mpegAssocExt ".mpeg"
#define mpegAssocKey StringChange(MyAppAssocName, " ", "") + mpegAssocExt

#define wmvAssocExt ".wmv"
#define wmvAssocKey StringChange(MyAppAssocName, " ", "") + wmvAssocExt

[Setup]
AppId=Anime4KPlayer
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
AppCopyright=Copyright (C) 2026 {#MyAppPublisher}
VersionInfoVersion={#MyAppVersion}
; Страница лицензии: показ LICENSE (GPL-3.0) с обязательным согласием.
LicenseFile={#SrcDir}\LICENSE
; Страница «информация после установки» перед финишем.
InfoAfterFile={#SrcDir}\README.md
; Страница выбора каталога установки (+ показ пути на странице Ready).
DefaultDirName={autopf}\{#MyAppName}
DisableDirPage=no
AlwaysShowDirOnReadyPage=yes
; Страница выбора папки в меню «Пуск».
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
AlwaysShowGroupOnReadyPage=yes
ChangesAssociations=yes
; Без UAC для установки текущему пользователю, с диалогом выбора
; «только я / все пользователи».
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
; Выбор языка визарда (EN/RU/JA) при старте установки.
ShowLanguageDialog=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes
RestartApplications=no
OutputDir=Output
OutputBaseFilename=anime4k_player_x64_setup
SetupIconFile={#SrcDir}\assets\favicon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"

[CustomMessages]
english.AssocFileExtension=Associate media files with {#MyAppName}
russian.AssocFileExtension=Ассоциировать медиафайлы с {#MyAppName}
japanese.AssocFileExtension=メディアファイルを{#MyAppName}に関連付ける

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "fileassoc"; Description: "{cm:AssocFileExtension}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#SrcDir}\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SrcDir}\assets\shaders\*"; DestDir: "{app}\shaders"; Flags: ignoreversion recursesubdirs
Source: "{#SrcDir}\assets\images\*"; DestDir: "{app}\images"; Flags: ignoreversion recursesubdirs
Source: "{#SrcDir}\assets\docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion
Source: "{#SrcDir}\assets\favicon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SrcDir}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SrcDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
; libmpv for --features video builds (bundled by CI, see release.yml)
Source: "{#SrcDir}\target\release\libmpv-2.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Registry]
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "{#MyAppName}"; Flags: uninsdeletekey

Root: HKA; Subkey: "Software\Classes\{#mp4AssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#mp4AssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mp4AssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mp4AssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mp4AssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#mkvAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#mkvAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mkvAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mkvAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mkvAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#m4aAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#m4aAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#m4aAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#m4aAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#m4aAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#webmAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#webmAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#webmAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#webmAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#webmAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#mp3AssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#mp3AssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mp3AssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mp3AssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mp3AssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#tsAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#tsAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#tsAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#tsAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#tsAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#movAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#movAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#movAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#movAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#movAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#aviAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#aviAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#aviAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#aviAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#aviAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#wavAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#wavAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#wavAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#wavAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#wavAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#oggAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#oggAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#oggAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#oggAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#oggAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#flacAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#flacAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#flacAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#flacAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#flacAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#aacAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#aacAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#aacAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#aacAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#aacAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#opusAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#opusAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#opusAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#opusAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#opusAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#mpegAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#mpegAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mpegAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mpegAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#mpegAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

Root: HKA; Subkey: "Software\Classes\{#wmvAssocExt}\OpenWithProgids"; ValueType: string; ValueName: "{#wmvAssocKey}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#wmvAssocKey}"; ValueType: string; ValueName: ""; ValueData: "{#MyAppAssocName}"; Flags: uninsdeletekey; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#wmvAssocKey}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: fileassoc
Root: HKA; Subkey: "Software\Classes\{#wmvAssocKey}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassoc

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
