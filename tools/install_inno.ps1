param([string]$Destination = (Join-Path $PSScriptRoot '..\target\tools\InnoSetup'))
$ErrorActionPreference = 'Stop'
$Destination = [IO.Path]::GetFullPath($Destination)
$compiler = Join-Path $Destination 'ISCC.exe'
if (-not (Test-Path -LiteralPath $compiler)) {
    New-Item -ItemType Directory -Path $Destination -Force | Out-Null
    $bootstrap = Join-Path $Destination 'innosetup-6.7.3.exe'
    Invoke-WebRequest 'https://github.com/jrsoftware/issrc/releases/download/is-6_7_3/innosetup-6.7.3.exe' -OutFile $bootstrap
    if ((Get-FileHash -LiteralPath $bootstrap -Algorithm SHA256).Hash -ne
        '9c73c3bae7ed48d44112a0f48e66742c00090bdb5bef71d9d3c056c66e97b732') {
        throw 'Inno Setup download checksum does not match'
    }
    $process = Start-Process -FilePath $bootstrap -WindowStyle Hidden -Wait -PassThru -ArgumentList @(
        '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-', '/CURRENTUSER',
        '/NOICONS', '/TASKS=', ('/DIR="' + $Destination + '"'))
    if ($process.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $compiler)) {
        throw "Inno Setup installation failed: $($process.ExitCode)"
    }
}
Write-Output $compiler
