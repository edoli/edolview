# Constant helper; all release metadata and paths are data, never interpolated code.
$ErrorActionPreference = 'Stop'
$taskDirectory = [IO.Path]::GetFullPath($env:EDOLVIEW_UPDATE_DIR)
if ([IO.Path]::GetFileName($taskDirectory) -notmatch '^edolview-update-[A-Za-z0-9-]+$') {
    throw 'Invalid update directory'
}
$context = [IO.File]::ReadAllText((Join-Path $taskDirectory 'context.json')) | ConvertFrom-Json
$executable = [IO.Path]::GetFullPath($context.executable)
$package = [IO.Path]::GetFullPath($context.package)
if ([IO.Path]::GetDirectoryName($package) -ne $taskDirectory) { throw 'Invalid package path' }
$installDirectory = [IO.Path]::GetDirectoryName($executable)
$resultFile = Join-Path $context.config 'update-result.txt'
$committed = $false
$parent = $null
$rollback = @()
$succeeded = $false

function Write-Failure([string]$Message) {
    [IO.File]::WriteAllText($resultFile, ('ERROR' + [Environment]::NewLine + 'Update failed: ' + $Message), (New-Object Text.UTF8Encoding($false)))
}

function Verify-Package {
    $algorithm = [Security.Cryptography.SHA256]::Create()
    $stream = $null
    try {
        $stream = [IO.File]::OpenRead($package)
        $hash = [BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace('-', '')
        if ($hash -ne $context.sha256) { throw 'Downloaded package SHA-256 does not match.' }
    } finally {
        if ($stream) { $stream.Dispose() }
        $algorithm.Dispose()
    }
}

function Get-InstallScope {
    foreach ($scope in @('user', 'machine')) {
        $hive = if ($scope -eq 'user') { 'HKEY_CURRENT_USER' } else { 'HKEY_LOCAL_MACHINE' }
        $record = Get-ItemProperty -LiteralPath ('Registry::' + $hive + '\Software\Edoli\Edolview') -ErrorAction SilentlyContinue
        if ($record -and $record.InstallLocation -and
            (Resolve-Path -LiteralPath $record.InstallLocation -ErrorAction SilentlyContinue).ProviderPath.TrimEnd('\') -eq
            (Resolve-Path -LiteralPath $installDirectory).ProviderPath.TrimEnd('\')) {
            return $scope
        }
    }
    throw 'This installer copy is not registered at its current location. Reinstall it using Setup.'
}

function Assert-Unlocked([string]$Path) {
    if ([IO.File]::Exists($Path)) {
        try {
            $probe = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Write, [IO.FileShare]::Read)
            $probe.Dispose()
        } catch { throw 'Close other Edolview windows before updating.' }
    }
}

function Quote-Argument([string]$Argument) {
    $escaped = [regex]::Replace($Argument, '(\\*)"', '$1$1\"')
    $escaped = [regex]::Replace($escaped, '(\\+)$', '$1$1')
    return '"' + $escaped + '"'
}

try {
    if ($context.version -notmatch '^\d+\.\d+\.\d+$' -or $context.sha256 -notmatch '^[0-9a-f]{64}$') {
        throw 'Invalid release metadata'
    }
    $parent = [Diagnostics.Process]::GetProcessById([int]$context.parent)
    $null = $parent.Handle # Acquire a real handle before announcing readiness.
    if ($parent.MainModule.FileName -ne $executable) { throw 'Parent executable does not match.' }
    Verify-Package
    $portable = [IO.Path]::GetFileName($package) -eq 'edolview-windows.zip'
    if (-not $portable) {
        if ([IO.Path]::GetFileName($package) -ne 'edolview-installer-exe.exe') { throw 'Unsupported package' }
        $scope = Get-InstallScope
    }
    [IO.File]::WriteAllText((Join-Path $taskDirectory 'ready'), 'ready')
    while (-not [IO.File]::Exists((Join-Path $taskDirectory 'go'))) {
        if ($parent.HasExited) {
            if ([IO.File]::Exists((Join-Path $taskDirectory 'go'))) { break }
            return
        }
        Start-Sleep -Milliseconds 25
    }
    $committed = $true
    if (-not $parent.WaitForExit(60000)) {
        $committed = $false
        throw 'Edolview did not close. Try updating again.'
    }
    Verify-Package
    if ($portable) {
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $stage = Join-Path $taskDirectory 'stage'
        $backup = Join-Path $taskDirectory 'backup'
        [IO.Directory]::CreateDirectory($stage) | Out-Null
        [IO.Directory]::CreateDirectory($backup) | Out-Null
        $archive = [IO.Compression.ZipFile]::OpenRead($package)
        try {
            $names = @{}
            foreach ($entry in $archive.Entries) {
                # Only the flat executable/runtime payload is installed. No archive
                # path, symlink, or unrelated user file can become a destination.
                if ($entry.FullName -cne 'edolview.exe' -and $entry.FullName -cnotmatch '^colormap/(mono|rgb)/[A-Za-z0-9_-]+\.glsl$') { continue }
                if ($names.ContainsKey($entry.FullName)) { throw 'Duplicate package entry' }
                if (($entry.ExternalAttributes -shr 16 -band 0xF000) -eq 0xA000) { throw 'Package symlink is not allowed' }
                $names[$entry.FullName] = $true
                $payloadPath = Join-Path $stage $entry.FullName
                [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($payloadPath)) | Out-Null
                [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $payloadPath)
            }
        } finally { $archive.Dispose() }
        $newExecutable = Join-Path $stage 'edolview.exe'
        if (-not [IO.File]::Exists($newExecutable) -or
            (Get-Item -LiteralPath $newExecutable).VersionInfo.ProductVersion -ne $context.version) {
            throw 'The downloaded executable version does not match the release.'
        }
        $files = @(Get-ChildItem -LiteralPath $stage -File -Recurse)
        foreach ($file in $files) {
            $relative = $file.FullName.Substring($stage.Length + 1)
            $destination = if ($relative -eq 'edolview.exe') { $executable } else { Join-Path $installDirectory $relative }
            Assert-Unlocked $destination
            $saved = Join-Path $backup $relative
            $existed = [IO.File]::Exists($destination)
            if ($existed) {
                [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($saved)) | Out-Null
                [IO.File]::Copy($destination, $saved)
            }
            $rollback += [pscustomobject]@{ Destination = $destination; Saved = $saved; Existed = $existed }
        }
        for ($i = 0; $i -lt $files.Count; $i++) {
            [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($rollback[$i].Destination)) | Out-Null
            [IO.File]::Copy($files[$i].FullName, $rollback[$i].Destination, $true)
        }
    } else {
        if ($scope -eq 'user') { Assert-Unlocked $executable }
        $scopeArgument = if ($scope -eq 'machine') { '/ALLUSERS' } else { '/CURRENTUSER' }
        $arguments = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/SP-', '/NORESTART',
            '/RESTARTEXITCODE=3010', '/NOCLOSEAPPLICATIONS', '/NORESTARTAPPLICATIONS',
            $scopeArgument, ('/DIR="' + $installDirectory + '"'),
            ('/LOG="' + (Join-Path $taskDirectory 'install.log') + '"'))
        $options = @{ FilePath = $package; ArgumentList = $arguments; Wait = $true; PassThru = $true }
        if ($scope -eq 'machine') { $options.Verb = 'RunAs' } else { $options.WindowStyle = 'Hidden' }
        $setup = Start-Process @options
        if ($setup.ExitCode -ne 0) { throw "Setup did not complete (exit code $($setup.ExitCode))." }
    }
    if ((Get-Item -LiteralPath $executable).VersionInfo.ProductVersion -ne $context.version) {
        throw 'The installed version does not match the release.'
    }
    $succeeded = $true
    [IO.File]::WriteAllText($resultFile, ('OK' + [Environment]::NewLine + 'Update complete: Edolview v' + $context.version + ' is ready to use.'), (New-Object Text.UTF8Encoding($false)))
} catch {
    foreach ($file in $rollback) {
        try {
            if ($file.Existed) { [IO.File]::Copy($file.Saved, $file.Destination, $true) }
            else { [IO.File]::Delete($file.Destination) }
        } catch { }
    }
    try { Write-Failure $_.Exception.Message } catch { }
} finally {
    if ($parent) { $parent.Dispose() }
    if ($committed) {
        # The helper is never elevated; restart retains the original user and config.
        try {
            $options = @{ FilePath = $executable; WorkingDirectory = $installDirectory; WindowStyle = 'Normal' }
            if (@($context.arguments).Count) {
                $options.ArgumentList = (@($context.arguments) | ForEach-Object { Quote-Argument $_ }) -join ' '
            }
            Start-Process @options | Out-Null
        } catch { try { Write-Failure ('Could not reopen Edolview: ' + $_.Exception.Message) } catch { } }
    }
    foreach ($name in @('stage', 'backup')) {
        $owned = [IO.Path]::GetFullPath((Join-Path $taskDirectory $name))
        if ([IO.Path]::GetDirectoryName($owned) -eq $taskDirectory) {
            Remove-Item -LiteralPath $owned -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
    foreach ($name in @('context.json', 'update.ps1', 'ready', 'go.pending', 'go', [IO.Path]::GetFileName($package))) {
        Remove-Item -LiteralPath (Join-Path $taskDirectory $name) -Force -ErrorAction SilentlyContinue
    }
    if ($succeeded) { Remove-Item -LiteralPath (Join-Path $taskDirectory 'install.log') -Force -ErrorAction SilentlyContinue }
    Remove-Item -LiteralPath $taskDirectory -ErrorAction SilentlyContinue
}
