{pkgs, lib, ...}: 
{
  # Fix for Cursor desktop entry
  environment.systemPackages = with pkgs; [
    (pkgs.writeTextFile {
      name = "cursor-desktop-file";
      destination = "/share/applications/cursor.desktop";
      text = ''
        [Desktop Entry]
        Name=Cursor
        Comment=The AI Code Editor.
        GenericName=Text Editor
        Exec=cursor %F
        Icon=co.anysphere.cursor
        Type=Application
        StartupNotify=false
        StartupWMClass=Cursor
        Categories=TextEditor;Development;IDE;
        MimeType=application/x-cursor-workspace;
        Actions=new-empty-window;
        Keywords=cursor;

        [Desktop Action new-empty-window]
        Name=New Empty Window
        Exec=cursor --new-window %F
        Icon=co.anysphere.cursor
      '';
    })
  ];
} 