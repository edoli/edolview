param(
    [string]$Executable = "$PSScriptRoot/../target/release/edolview.exe",
    [int]$Runs = 5,
    [string]$ImagePath,
    [string]$ConfigRoot = "$PSScriptRoot/../target/startup-benchmark-config",
    [string]$OutputDirectory = "$PSScriptRoot/../target/startup-traces"
)
$ErrorActionPreference = 'Stop'
$executablePath = (Resolve-Path -LiteralPath $Executable).Path
$null = New-Item -ItemType Directory -Force -Path $OutputDirectory
$outputPath = (Resolve-Path -LiteralPath $OutputDirectory).Path
$previousTrace = $env:EDOLVIEW_STARTUP_TRACE
$previousLocalAppData = $env:LOCALAPPDATA
# Isolate the active-window registry and settings writes from the user's session.
$configDirectory = Join-Path $ConfigRoot 'Edolview'
$null = New-Item -ItemType Directory -Force -Path $configDirectory
$configRootPath = (Resolve-Path -LiteralPath $ConfigRoot).Path
$settingsPath = Join-Path $previousLocalAppData 'Edolview/settings.json'
$settings = if (Test-Path -LiteralPath $settingsPath) { Get-Content -Raw -LiteralPath $settingsPath | ConvertFrom-Json } else { [pscustomobject]@{} }
$settings | Add-Member -NotePropertyName external_open_mode -NotePropertyValue 'new_window' -Force
[System.IO.File]::WriteAllText((Join-Path $configDirectory 'settings.json'), ($settings | ConvertTo-Json -Depth 32), [System.Text.UTF8Encoding]::new($false))
$env:LOCALAPPDATA = $configRootPath
try {
    for ($run = 1; $run -le $Runs; $run++) {
        $tracePath = Join-Path $outputPath "run-$run.jsonl"
        if (Test-Path -LiteralPath $tracePath) { Remove-Item -LiteralPath $tracePath }
        $env:EDOLVIEW_STARTUP_TRACE = $tracePath
        $launch = @{
            FilePath = $executablePath
            WorkingDirectory = (Resolve-Path "$PSScriptRoot/..").Path
            WindowStyle = 'Hidden'
            PassThru = $true
            RedirectStandardError = (Join-Path $outputPath "run-$run.stderr.log")
        }
        if ($ImagePath) { $launch.ArgumentList = '"' + (Resolve-Path -LiteralPath $ImagePath).Path + '"' }
        $watch = [System.Diagnostics.Stopwatch]::StartNew()
        $process = Start-Process @launch
        try {
            $expected = if ($ImagePath) { 'first_image_paint' } else { 'first_ui_build' }
            do {
                Start-Sleep -Milliseconds 20
                if ($process.HasExited) { throw "Process exited before $expected (code $($process.ExitCode))" }
                $events = if (Test-Path -LiteralPath $tracePath) {
                    @(Get-Content -LiteralPath $tracePath | ForEach-Object {
                        # The writer may still be appending the last JSON record.
                        try { $_ | ConvertFrom-Json -ErrorAction Stop } catch { }
                    })
                } else { @() }
                $ready = @($events | Where-Object event -eq $expected)
                if ($watch.Elapsed.TotalSeconds -gt 30) { throw "Startup timed out waiting for $expected" }
            } while ($ready.Count -eq 0)
            [pscustomobject]@{ Run = $run; Milestone = $expected; MainToMilestoneMs = $ready[0].elapsed_ms; ObservedFromLaunchMs = $watch.Elapsed.TotalMilliseconds }
            # Allow the frame to be submitted and background startup work to settle.
            Start-Sleep -Milliseconds 1000
        } finally {
            if (-not $process.HasExited) {
                $closed = $process.CloseMainWindow()
                if (-not $closed -or -not $process.WaitForExit(3000)) { $process.Kill(); $process.WaitForExit() }
            }
            $process.Dispose()
        }
    }
} finally {
    $env:EDOLVIEW_STARTUP_TRACE = $previousTrace
    $env:LOCALAPPDATA = $previousLocalAppData
}
