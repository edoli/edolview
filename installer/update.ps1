# Runs in a separate, hidden Windows PowerShell process after the download.
# The application sends data through environment variables, never script interpolation.
$ErrorActionPreference = 'Stop'
$taskDirectory = [IO.Path]::GetFullPath($env:EDOLVIEW_UPDATE_DIR)
$temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\') + '\'
if (-not $taskDirectory.StartsWith($temporaryRoot, [StringComparison]::OrdinalIgnoreCase) -or
    [IO.Path]::GetFileName($taskDirectory) -notmatch '^edolview-update-\d+-\d+-\d+$') {
    throw 'Invalid update directory'
}
$installer = Join-Path $taskDirectory 'setup.exe'
$executable = [IO.Path]::GetFullPath($env:EDOLVIEW_UPDATE_EXE)
$installDirectory = [IO.Path]::GetDirectoryName($executable)
$resultFile = Join-Path $env:EDOLVIEW_CONFIG_DIR 'update-result.txt'
$committed = $false
$succeeded = $false
$parent = $null

function Write-Result([string]$Message) {
    # Keep the result across restart, including installer failure or a canceled UAC prompt.
    [IO.File]::WriteAllText($resultFile, $Message, (New-Object Text.UTF8Encoding($false)))
}

try {
    if ($env:EDOLVIEW_UPDATE_SCOPE -notin @('user', 'machine') -or
        $env:EDOLVIEW_UPDATE_VERSION -notmatch '^\d+\.\d+\.\d+$') {
        throw '잘못된 업데이트 정보입니다.'
    }
    if (-not [IO.File]::Exists($installer)) {
        throw 'Downloaded installer was not found.'
    }
    $parent = [Diagnostics.Process]::GetProcessById([int]$env:EDOLVIEW_UPDATE_PARENT)
    # Acquire a real process handle before announcing readiness, avoiding PID reuse races.
    $null = $parent.Handle
    if ($parent.MainModule.FileName -ne $executable) {
        throw '업데이트할 프로그램 경로가 일치하지 않습니다.'
    }
    [IO.File]::WriteAllText((Join-Path $taskDirectory 'ready'), 'ready')
    while (-not [IO.File]::Exists((Join-Path $taskDirectory 'go'))) {
        if ($parent.HasExited) {
            # The app creates go before exiting; recheck after observing exit.
            if ([IO.File]::Exists((Join-Path $taskDirectory 'go'))) { break }
            return
        }
        Start-Sleep -Milliseconds 50
    }
    $committed = $true
    # UI readiness and process shutdown can be delayed. Keep the prepared
    # installer until the actual parent exits; never terminate the viewer.
    $parent.WaitForExit()
    # Fail before running Setup if another viewer process still has this EXE loaded.
    if ($env:EDOLVIEW_UPDATE_SCOPE -eq 'user') {
        try {
            $probe = [IO.File]::Open($executable, [IO.FileMode]::Open, [IO.FileAccess]::Write, [IO.FileShare]::Read)
            $probe.Dispose()
        } catch {
            throw 'Close other Edolview windows before updating.'
        }
    }
    # For machine installs, Setup checks the loaded EXE after elevation; an
    # unelevated write probe would fail solely because Program Files is protected.
    $scopeArgument = if ($env:EDOLVIEW_UPDATE_SCOPE -eq 'machine') { '/ALLUSERS' } else { '/CURRENTUSER' }
    $arguments = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/SP-', '/NORESTART',
        '/RESTARTEXITCODE=3010', '/NOCLOSEAPPLICATIONS', '/NORESTARTAPPLICATIONS',
        $scopeArgument, ('/DIR="' + $installDirectory + '"'),
        ('/LOG="' + (Join-Path $taskDirectory 'install.log') + '"'))
    $options = @{ FilePath = $installer; ArgumentList = $arguments; Wait = $true; PassThru = $true }
    if ($env:EDOLVIEW_UPDATE_SCOPE -eq 'machine') {
        # Keep the helper itself unelevated, so restart always uses the original user.
        $options.Verb = 'RunAs'
    } else {
        $options.WindowStyle = 'Hidden'
    }
    $setup = Start-Process @options
    if ($setup.ExitCode -ne 0) { throw "설치가 완료되지 않았습니다 (코드 $($setup.ExitCode))." }
    if ((Get-Item -LiteralPath $executable).VersionInfo.ProductVersion -ne $env:EDOLVIEW_UPDATE_VERSION) {
        throw '설치한 버전이 일치하지 않습니다. Release 페이지에서 다시 설치하세요.'
    }
    $succeeded = $true
    Write-Result ("OK`nUpdated to v" + $env:EDOLVIEW_UPDATE_VERSION + '.')
} catch {
    try { Write-Result ("ERROR`nUpdate failed: " + $_.Exception.Message + ' Log: ' + (Join-Path $taskDirectory 'install.log')) } catch { }
} finally {
    if ($parent) { $parent.Dispose() }
    if ($committed) {
        # Restart in the original user context; the helper itself is never elevated.
        try {
            Start-Process -FilePath $executable -WorkingDirectory $installDirectory -WindowStyle Normal
        } catch {
            try { Write-Result ("ERROR`nCould not reopen Edolview: " + $_.Exception.Message) } catch { }
        }
    }
    # Delete only files owned by this operation; never recursively delete an install/data folder.
    foreach ($name in @('setup.exe', 'update.ps1', 'ready', 'go.pending', 'go')) {
        Remove-Item -LiteralPath (Join-Path $taskDirectory $name) -Force -ErrorAction SilentlyContinue
    }
    if ($succeeded) { Remove-Item -LiteralPath (Join-Path $taskDirectory 'install.log') -Force -ErrorAction SilentlyContinue }
    Remove-Item -LiteralPath $taskDirectory -ErrorAction SilentlyContinue
}
