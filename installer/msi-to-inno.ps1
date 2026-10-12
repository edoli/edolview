# Run as the original user; elevate only the removal of the legacy MSI.
$ErrorActionPreference = 'Stop'
$taskDirectory = [IO.Path]::GetFullPath($env:EDOLVIEW_UPDATE_DIR)
$temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\') + '\'
if (-not $taskDirectory.StartsWith($temporaryRoot, [StringComparison]::OrdinalIgnoreCase) -or
    [IO.Path]::GetFileName($taskDirectory) -notmatch '^edolview-update-\d+-\d+-\d+$') {
    throw 'Invalid update directory'
}
$installer = Join-Path $taskDirectory 'setup.exe'
$executable = [IO.Path]::GetFullPath($env:EDOLVIEW_UPDATE_EXE)
$resultFile = Join-Path $env:EDOLVIEW_CONFIG_DIR 'update-result.txt'
$newDirectory = Join-Path $env:LOCALAPPDATA 'Programs\Edolview'
$newExecutable = Join-Path $newDirectory 'edolview.exe'
$restartExecutable = $executable
$parent = $null
$committed = $false
$succeeded = $false

function Write-Result([string]$Message) {
    [IO.File]::WriteAllText($resultFile, $Message, (New-Object Text.UTF8Encoding($false)))
}

function Get-MsiProperty([string]$Name, [object[]]$Arguments) {
    $msi.GetType().InvokeMember($Name, [Reflection.BindingFlags]::GetProperty, $null, $msi, $Arguments)
}

function Install-NewViewer([string]$Tasks) {
    $arguments = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/SP-', '/NORESTART',
        '/RESTARTEXITCODE=3010', '/NOCLOSEAPPLICATIONS', '/NORESTARTAPPLICATIONS',
        '/CURRENTUSER', ('/DIR="' + $newDirectory + '"'), ('/TASKS=' + $Tasks),
        ('/LOG="' + (Join-Path $taskDirectory 'install.log') + '"'))
    $setup = Start-Process -FilePath $installer -ArgumentList $arguments -WindowStyle Hidden -Wait -PassThru
    if ($setup.ExitCode -ne 0) { throw "Inno installation failed (code $($setup.ExitCode))." }
    if ((Get-Item -LiteralPath $newExecutable).VersionInfo.ProductVersion -ne $env:EDOLVIEW_UPDATE_VERSION) {
        throw 'Installed version does not match the downloaded release.'
    }
    $registered = (Get-ItemProperty -LiteralPath 'Registry::HKEY_CURRENT_USER\Software\Edoli\Edolview').InstallLocation
    if ([IO.Path]::GetFullPath($registered).TrimEnd('\') -ne $newDirectory.TrimEnd('\')) {
        throw 'The new per-user installation was not registered.'
    }
}

try {
    if ($env:EDOLVIEW_UPDATE_VERSION -notmatch '^\d+\.\d+\.\d+$' -or -not [IO.File]::Exists($installer)) {
        throw 'Invalid downloaded update.'
    }
    $parent = [Diagnostics.Process]::GetProcessById([int]$env:EDOLVIEW_UPDATE_PARENT)
    $null = $parent.Handle
    if ($parent.MainModule.FileName -ne $executable) { throw 'Update parent path does not match.' }
    # Verify identity using the same UpgradeCode and main component as v1.0.2.
    # Never select unrelated MSI products by display name alone.
    $msi = New-Object -ComObject WindowsInstaller.Installer
    $products = Get-MsiProperty 'RelatedProducts' @('{9B95884C-31AA-4A5C-8209-D7CE6CA9C2D8}')
    $matching = @()
    foreach ($product in $products) {
        $installedPath = Get-MsiProperty 'ComponentPath' @($product, '{D7781E26-5FAE-4F3F-8B92-BD40530A4628}')
        if ($installedPath -and [IO.Path]::GetFullPath($installedPath) -eq $executable) {
            $matching += $product
        }
    }
    if ($matching.Count -ne 1) { throw 'Could not identify this legacy MSI installation.' }
    $productCode = $matching[0]
    if ($productCode -notmatch '^\{[0-9A-Fa-f-]{36}\}$') { throw 'Invalid MSI product code.' }
    $tasks = @()
    if ((Get-MsiProperty 'FeatureState' @($productCode, 'StartMenuShortcutsFeature')) -eq 3) {
        $tasks += 'startmenuicon'
    }
    if ((Get-MsiProperty 'FeatureState' @($productCode, 'DesktopShortcutFeature')) -eq 3) {
        $tasks += 'desktopicon'
    }
    if (Test-Path -LiteralPath $newExecutable) {
        if ([version](Get-Item -LiteralPath $newExecutable).VersionInfo.ProductVersion -gt [version]$env:EDOLVIEW_UPDATE_VERSION) {
            throw 'A newer per-user version is already installed.'
        }
    }
    [IO.File]::WriteAllText((Join-Path $taskDirectory 'ready'), 'ready')
    while (-not [IO.File]::Exists((Join-Path $taskDirectory 'go'))) {
        if ($parent.HasExited) {
            if ([IO.File]::Exists((Join-Path $taskDirectory 'go'))) { break }
            return
        }
        Start-Sleep -Milliseconds 50
    }
    $committed = $true
    $parent.WaitForExit()
    if (Get-Process edolview -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $executable }) {
        throw 'Close other legacy Edolview windows before migrating.'
    }
    # Confirm the new viewer is usable before removing the old installation.
    # Delay shortcuts until after MSI removal, which may remove the old links.
    Install-NewViewer ''
    $removeArguments = @('/x', $productCode, '/passive', '/norestart',
        'REBOOT=ReallySuppress', 'MSIRESTARTMANAGERCONTROL=Disable', '/L*v',
        ('"' + (Join-Path $taskDirectory 'msi-remove.log') + '"'))
    $remove = Start-Process -FilePath (Join-Path $env:SystemRoot 'System32\msiexec.exe') -Verb RunAs -ArgumentList $removeArguments -Wait -PassThru
    if ($remove.ExitCode -notin @(0, 3010)) { throw "Legacy MSI removal failed (code $($remove.ExitCode))." }
    $restartExecutable = $newExecutable
    Install-NewViewer ($tasks -join ',')
    $succeeded = $true
    Write-Result ("OK`nMigrated to v" + $env:EDOLVIEW_UPDATE_VERSION + '.')
} catch {
    try { Write-Result ("ERROR`nMigration failed: " + $_.Exception.Message + ' Logs: ' + $taskDirectory) } catch { }
} finally {
    if ($parent) { $parent.Dispose() }
    if ($committed) {
        if (-not [IO.File]::Exists($restartExecutable) -and [IO.File]::Exists($newExecutable)) {
            $restartExecutable = $newExecutable
        }
        try {
            Start-Process -FilePath $restartExecutable -WorkingDirectory ([IO.Path]::GetDirectoryName($restartExecutable))
        } catch {
            try { Write-Result ("ERROR`nCould not reopen Edolview: " + $_.Exception.Message) } catch { }
        }
    }
    foreach ($name in @('setup.exe', 'update.ps1', 'ready', 'go.pending', 'go')) {
        Remove-Item -LiteralPath (Join-Path $taskDirectory $name) -Force -ErrorAction SilentlyContinue
    }
    if ($succeeded) {
        foreach ($name in @('install.log', 'msi-remove.log')) {
            Remove-Item -LiteralPath (Join-Path $taskDirectory $name) -Force -ErrorAction SilentlyContinue
        }
    }
    if ([IO.Directory]::Exists($taskDirectory) -and
        [IO.Directory]::GetFileSystemEntries($taskDirectory).Count -eq 0) {
        [IO.Directory]::Delete($taskDirectory)
    }
}
