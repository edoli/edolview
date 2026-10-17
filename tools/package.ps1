param(
    [string]$Version,
    [string]$Compiler = $env:INNO_SETUP_COMPILER,
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '..\target\dist'),
    [string]$SourceExe = (Join-Path $PSScriptRoot '..\target\release-dist\edolview.exe'),
    [switch]$NoBuild
)
$ErrorActionPreference = 'Stop'
$projectRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$cargoVersion = (& python -c "import sys, tomllib; print(tomllib.load(open(sys.argv[1], 'rb'))['package']['version'])" (Join-Path $projectRoot 'Cargo.toml'))
if ($LASTEXITCODE -ne 0) { throw 'Cannot read Cargo.toml version' }
if ($PSBoundParameters.ContainsKey('Version') -and $Version -cne $cargoVersion) {
    throw "Requested version '$Version' does not match Cargo.toml version '$cargoVersion'"
}
$Version = $cargoVersion
if ($Version -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' -or
    @($Version.Split('.') | Where-Object { [long]$_ -gt 65535 }).Count -ne 0) {
    throw 'Version must be major.minor.patch, with each component <= 65535'
}
if (-not $Compiler) {
    $candidates = @((Join-Path $projectRoot 'target\tools\InnoSetup\ISCC.exe'),
        'C:\Program Files (x86)\Inno Setup 6\ISCC.exe', 'C:\Program Files\Inno Setup 7\ISCC.exe')
    $Compiler = $candidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
}
if (-not $Compiler -or -not (Test-Path -LiteralPath $Compiler)) {
    throw 'Install Inno Setup first: powershell -File tools/install_inno.ps1'
}
$SourceExe = [IO.Path]::GetFullPath($SourceExe)
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
Push-Location $projectRoot
try {
    if (-not $NoBuild) {
        & cargo run -p xtask -- icons
        if ($LASTEXITCODE -ne 0) { throw 'Asset generation failed' }
        & cargo build --profile release-dist
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
    }
    if ((Get-Item -LiteralPath $SourceExe).VersionInfo.ProductVersion -ne $Version) {
        throw 'EXE version does not match Cargo.toml version'
    }
    & $Compiler "/DAppVersion=$Version" "/DSourceExe=$SourceExe" "/DOutputPath=$OutputDirectory" "/DProjectRoot=$projectRoot" (Join-Path $projectRoot 'installer\edolview.iss')
    if ($LASTEXITCODE -ne 0) { throw 'Installer compilation failed' }
    $installer = Join-Path $OutputDirectory 'edolview-installer-exe.exe'
    Get-Item -LiteralPath $installer | Select-Object Name, Length
} finally {
    Pop-Location
}
