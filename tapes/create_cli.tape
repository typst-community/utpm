Set Width 1300
Set Height 250

Require utpm

Output "assets/gifs/create_cli.gif"

Env UTPM_CURRENT_DIR tmp

Type@100ms "utpm ws create -mf --name egcli --version 1.0.0"

Sleep 500ms

Enter

Sleep 5s