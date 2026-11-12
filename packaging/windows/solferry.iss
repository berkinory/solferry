; Installs for the current user only, so the app can replace itself without asking for admin.
#define AppVersion GetEnv("SOLFERRY_VERSION")
; The folder with the built solferry-desktop.exe and solferry.exe.
#define Release GetEnv("SOLFERRY_RELEASE")
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
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#Root}\dist
OutputBaseFilename=Solferry-windows-setup
SetupIconFile={#Root}\crates\solferry-desktop\assets\icon.ico
UninstallDisplayIcon={app}\Solferry.exe
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
CloseApplications=force
ChangesEnvironment=yes

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"

[Files]
Source: "{#Release}\solferry-desktop.exe"; DestDir: "{app}"; DestName: "Solferry.exe"; Flags: ignoreversion
; The CLI gets its own folder on PATH, since Windows wouldn't tell solferry.exe from Solferry.exe.
Source: "{#Release}\solferry.exe"; DestDir: "{app}\bin"; Flags: ignoreversion

[Icons]
Name: "{group}\Solferry"; Filename: "{app}\Solferry.exe"
Name: "{userdesktop}\Solferry"; Filename: "{app}\Solferry.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Solferry.exe"; Description: "Open Solferry"; Flags: nowait postinstall skipifsilent

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}\bin"; Check: NeedsPath

[Code]
function NeedsPath: Boolean;
var
  Path: string;
begin
  if not RegQueryStringValue(HKCU, 'Environment', 'Path', Path) then
    Path := '';
  Result := Pos(';' + Uppercase(ExpandConstant('{app}\bin')) + ';', ';' + Uppercase(Path) + ';') = 0;
end;
