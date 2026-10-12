; Anime Player — Inno Setup installer for the Rust build.
; Build from repo root: cargo build --release, then run ISCC on this file.
; Optionally drop libmpv-2.dll next to the exe for --features video builds.

#define MyAppName "Anime Player"
#define MyAppVersion "2.3.3"
#define MyAppPublisher "Denis Mazur"
#define MyAppURL "https://github.com/Communist02/AnimePlayer"
#define MyAppExeName "anime-player.exe"
#define MyAppAssocName MyAppName + " File"

#define SrcDir "..\.."

[Setup]
AppId=AnimePlayer
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
ChangesAssociations=yes
DisableProgramGroupPage=yes
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=Output
OutputBaseFilename=anime_player_x64_setup
SetupIconFile={#SrcDir}\assets\favicon.ico
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SrcDir}\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SrcDir}\assets\shaders\*"; DestDir: "{app}\shaders"; Flags: ignoreversion recursesubdirs
Source: "{#SrcDir}\assets\images\*"; DestDir: "{app}\images"; Flags: ignoreversion recursesubdirs
Source: "{#SrcDir}\assets\docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion
Source: "{#SrcDir}\assets\favicon.ico"; DestDir: "{app}"; Flags: ignoreversion
; libmpv for --features video builds (ship separately, same as the Python releases)
Source: "{#SrcDir}\target\release\mpv-2.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist

[Registry]
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "{#MyAppName}"; Flags: uninsdeletekey

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
