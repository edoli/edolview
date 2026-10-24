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
    # Remove only obsolete generated artifacts left by earlier local packaging.
    Get-ChildItem -LiteralPath $OutputDirectory -File | Where-Object {
        $_.Name -match '^SHA256SUMS(?:-(windows|mac|linux))?\.txt$' -or
        $_.Name -eq 'edot-update.txt' -or $_.Name -match '^Edot-Setup-[0-9]+\.[0-9]+\.[0-9]+-windows-x64\.exe$'
    } | ForEach-Object { Remove-Item -LiteralPath $_.FullName -Force }

Push-Location $projectRoot
try {
    if (-not $NoBuild) {
        & cargo run -p xtask -- icons
        if ($LASTEXITCODE -ne 0) { throw 'Asset generation failed' }
        & cargo build --profile release-dist --locked
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
    }
    if ((Get-Item -LiteralPath $SourceExe).VersionInfo.ProductVersion -ne $Version) {
        throw 'EXE version does not match Cargo.toml version'
    }
    & $Compiler "/DAppVersion=$Version" "/DSourceExe=$SourceExe" "/DOutputPath=$OutputDirectory" "/DProjectRoot=$projectRoot" (Join-Path $projectRoot 'installer\edolview.iss')
    if ($LASTEXITCODE -ne 0) { throw 'Installer compilation failed' }
    $installer = Join-Path $OutputDirectory 'edolview-installer-exe.exe'
    $stage = Join-Path $projectRoot ('target\portable-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $stage | Out-Null
    try {
        Copy-Item -LiteralPath $SourceExe -Destination (Join-Path $stage 'edolview.exe')
        Copy-Item -LiteralPath (Join-Path $projectRoot 'colormap') -Destination $stage -Recurse
        $portable = Join-Path $OutputDirectory 'edolview-windows.zip'
        Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $portable -Force
    } finally {
        $targetRoot = [IO.Path]::GetFullPath((Join-Path $projectRoot 'target')) + [IO.Path]::DirectorySeparatorChar
        if (-not [IO.Path]::GetFullPath($stage).StartsWith($targetRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Invalid stage path' }
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
    $checksums = @{}
    foreach ($file in @($installer, $portable)) {
        $checksums[[IO.Path]::GetFileName($file)] = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    [IO.File]::WriteAllText((Join-Path $OutputDirectory 'package-checksums-windows.json'), ($checksums | ConvertTo-Json), [Text.Encoding]::ASCII)
    Get-Item -LiteralPath $installer, $portable | Select-Object Name, Length
} finally {
    Pop-Location
}
