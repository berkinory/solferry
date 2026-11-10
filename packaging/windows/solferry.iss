; Installs for the current user only, so the app can replace itself without asking for admin.
#define AppVersion GetEnv("SOLFERRY_VERSION")
#define Platform GetEnv("SOLFERRY_PLATFORM")
#define Root "..\.."

[Setup]
AppId={{6F3A8C52-41D7-4E0B-9B26-5C1D7E2A9F10}
AppName=Solferry
AppVersion={#AppVersion}
AppPublisher=Solferry
AppPublisherURL=https://github.com/berkinory/solferry
DefaultDirName={localappdata}\Programs\Solferry
DefaultGroupName=Solferry
DisableProgramGroupPage=yes
DisableDirPage=yes
PrivilegesRequired=lowest
#if Platform == "windows-aarch64"
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#else
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#endif
OutputDir={#Root}\dist
OutputBaseFilename=Solferry-{#Platform}-setup
SetupIconFile={#Root}\crates\solferry-desktop\assets\icon.ico
UninstallDisplayIcon={app}\Solferry.exe
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
CloseApplications=force

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"

[Files]
Source: "{#Root}\dist\Solferry-{#Platform}.exe"; DestDir: "{app}"; DestName: "Solferry.exe"; Flags: ignoreversion

[Icons]
Name: "{group}\Solferry"; Filename: "{app}\Solferry.exe"
Name: "{userdesktop}\Solferry"; Filename: "{app}\Solferry.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Solferry.exe"; Description: "Open Solferry"; Flags: nowait postinstall skipifsilent
