<#
.SYNOPSIS
  Read-only snapshot of system state that game boosters commonly change.
  Writes one JSON file. Changes nothing. Windows PowerShell 5.1 compatible.
.EXAMPLE
  .\snapshot.ps1 -Out before.json
#>
param([Parameter(Mandatory = $true)][string]$Out)

$ErrorActionPreference = 'Continue'
$snap = [ordered]@{}
$snap.timestamp = (Get-Date).ToString('o')
$snap.computer  = $env:COMPUTERNAME
$snap.user      = "$env:USERDOMAIN\$env:USERNAME"
$snap.isAdmin   = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$snap.notes     = New-Object System.Collections.ArrayList

# ---------------- power ----------------
Write-Host 'power...'
$guidRe = '[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}'
$active = (& powercfg /getactivescheme) -join ' '
$activeGuid = if ($active -match $guidRe) { $Matches[0] } else { $null }
$schemes = [ordered]@{}
foreach ($line in (& powercfg /list)) {
    if ($line -match "($guidRe)\s*(.*)$") { $schemes[$Matches[1]] = $Matches[2].Trim() }
}
# /qh = /query including hidden settings (core parking etc.). Parsed by indentation and
# GUIDs, not by English labels, so it works on any Windows language.
$settings = [ordered]@{}
if ($activeGuid) {
    $q = & powercfg /qh $activeGuid
    if (-not $q) { $q = & powercfg /query $activeGuid }
    $sub = ''; $cur = $null
    $flush = {
        if ($cur -and $cur.idx.Count -ge 2) {
            $key = "$($cur.sub) / $($cur.name)"
            if ($settings.Contains($key)) { $key = "$key [$($cur.guid)]" }
            $settings[$key] = "AC=$($cur.idx[-2]) DC=$($cur.idx[-1])"
        }
    }
    foreach ($line in $q) {
        if ($line -match "^(\s*)\S[^:]*:\s*($guidRe)\s*(?:\((.*)\))?") {
            $indent = $Matches[1].Length; $g = $Matches[2]; $n = if ($Matches[3]) { $Matches[3] } else { $g }
            if ($indent -ge 2 -and $indent -lt 4) { & $flush; $cur = $null; $sub = $n }
            elseif ($indent -ge 4) { & $flush; $cur = @{ sub = $sub; name = $n; guid = $g; idx = @() } }
        }
        elseif ($cur -and $line -match ':\s*(0x[0-9a-fA-F]+)\s*$') { $cur.idx += $Matches[1] }
    }
    & $flush
} else { [void]$snap.notes.Add('could not read active power scheme') }
$activeName = if ($activeGuid) { $schemes[$activeGuid] } else { '' }
$snap.power = [ordered]@{ activeScheme = "$activeGuid $activeName"; schemes = $schemes; settings = $settings }

# ---------------- services ----------------
Write-Host 'services...'
$svc = [ordered]@{}
Get-CimInstance Win32_Service | Sort-Object Name | ForEach-Object { $svc[$_.Name] = "$($_.State) / $($_.StartMode)$(if ($_.DelayedAutoStart) { ' (delayed)' })" }
$snap.services = $svc

# ---------------- processes ----------------
Write-Host 'processes...'
$procs = New-Object System.Collections.ArrayList
$denied = New-Object System.Collections.ArrayList
foreach ($p in (Get-Process | Sort-Object Name, Id)) {
    $prio = $null; $aff = $null; $path = $null
    try { $prio = $p.PriorityClass.ToString() } catch {}
    try { $aff = '0x{0:X}' -f $p.ProcessorAffinity.ToInt64() } catch {}
    try { $path = $p.Path } catch {}
    if (-not $prio) { [void]$denied.Add("$($p.Name) ($($p.Id))") }
    [void]$procs.Add([ordered]@{ name = $p.Name; pid = $p.Id; path = $path; priority = $prio; affinity = $aff })
}
$snap.processes = $procs
$snap.processesAccessDenied = $denied

# ---------------- registry ----------------
Write-Host 'registry...'
$reg = [ordered]@{}
function Format-RegValue($v, $kind) {
    switch ($kind) {
        'DWord'       { return ('{0} (0x{0:X8})' -f $v) }
        'QWord'       { return ('{0} (0x{0:X16})' -f $v) }
        'Binary'      { return 'hex:' + (($v | ForEach-Object { $_.ToString('x2') }) -join '') }
        'MultiString' { return ($v -join ' | ') }
        default       { return "$v" }
    }
}
function Read-Key([string]$path, [string[]]$only) {
    try { $k = Get-Item -LiteralPath "Registry::$path" -ErrorAction Stop }
    catch {
        if ($_.FullyQualifiedErrorId -like 'PathNotFound*') { $script:reg[$path] = '(key missing)' }
        else { $script:reg[$path] = "(error: $($_.Exception.Message))" }
        return
    }
    $names = if ($only) { $only } else { $k.GetValueNames() }
    if (-not $names) { $script:reg[$path] = '(key exists, no values)'; return }
    $present = $k.GetValueNames()
    foreach ($n in $names) {
        $label = if ($n) { $n } else { '(Default)' }
        if ($present -notcontains $n) { $script:reg["$path :: $label"] = '(value missing)'; continue }
        $script:reg["$path :: $label"] = Format-RegValue ($k.GetValue($n, $null, 'DoNotExpandEnvironmentNames')) ($k.GetValueKind($n))
    }
}
function Short([string]$name) { $name -replace '^HKEY_LOCAL_MACHINE', 'HKLM' -replace '^HKEY_CURRENT_USER', 'HKCU' }

$mm = 'HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile'
Read-Key $mm
Get-ChildItem -LiteralPath "Registry::$mm" -Recurse -ErrorAction SilentlyContinue | ForEach-Object { Read-Key (Short $_.Name) }
Read-Key "$mm\Tasks\Games"   # marks it missing if the recursive walk didn't find it
foreach ($k in @(
    'HKLM\SYSTEM\CurrentControlSet\Control\PriorityControl',
    'HKLM\SYSTEM\CurrentControlSet\Control\Power',
    'HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling',
    'HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers',
    'HKCU\System\GameConfigStore',
    'HKCU\Software\Microsoft\GameBar',
    'HKLM\SOFTWARE\Policies\Microsoft\Windows\GameDVR',
    'HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management',
    'HKCU\Control Panel\Desktop',
    'HKCU\Control Panel\Desktop\WindowMetrics',
    'HKCU\Control Panel\Mouse',
    'HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects',
    'HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update',
    'HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU'
)) { Read-Key $k }
$tcp = 'HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces'
# Only the tuning values: the interface keys also hold DHCP lease timestamps that change on their own.
Get-ChildItem -LiteralPath "Registry::$tcp" -ErrorAction SilentlyContinue | ForEach-Object {
    Read-Key (Short $_.Name) @('TcpAckFrequency', 'TCPNoDelay', 'TcpDelAckTicks')
}
$ifeo = 'HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options'
Get-ChildItem -LiteralPath "Registry::$ifeo" -ErrorAction SilentlyContinue | ForEach-Object {
    $po = (Short $_.Name) + '\PerfOptions'
    if (Test-Path -LiteralPath "Registry::$po") { Read-Key $po }
}
$snap.registry = $reg

# ---------------- windows update (the Automatic Updates COM object SGB uses) ----------------
Write-Host 'windows update...'
$wu = [ordered]@{}
try {
    $au = New-Object -ComObject Microsoft.Update.AutoUpdate
    $wu.serviceEnabled = $au.ServiceEnabled
    # NotificationLevel: 0 not configured, 1 disabled, 2 notify before download, 3 before install, 4 scheduled
    $wu.notificationLevel = $au.Settings.NotificationLevel
    $wu.readOnly = $au.Settings.ReadOnly
    $wu.required = $au.Settings.Required
} catch { [void]$snap.notes.Add("windows update query failed: $($_.Exception.Message)") }
$snap.windowsUpdate = $wu

# ---------------- bcd ----------------
Write-Host 'bcdedit...'
$bcd = [ordered]@{}
$bout = & bcdedit /enum '{current}' 2>&1
if ($LASTEXITCODE -ne 0) {
    [void]$snap.notes.Add("bcdedit failed (needs admin): $($bout -join ' ')")
} else {
    foreach ($line in $bout) {
        if ("$line" -match '^(\S+)\s+(.+)$' -and "$line" -notmatch '^-+$') { $bcd[$Matches[1]] = $Matches[2].Trim() }
    }
}
$snap.bcd = $bcd

# ---------------- timer resolution ----------------
Write-Host 'timer...'
try {
    if (-not ('GbTimerRes' -as [type])) {
        Add-Type -TypeDefinition @'
using System.Runtime.InteropServices;
public static class GbTimerRes {
    [DllImport("ntdll.dll")]
    public static extern int NtQueryTimerResolution(out uint minimumResolution, out uint maximumResolution, out uint currentResolution);
}
'@
    }
    [uint32]$a = 0; [uint32]$b = 0; [uint32]$c = 0
    [void][GbTimerRes]::NtQueryTimerResolution([ref]$a, [ref]$b, [ref]$c)
    # units are 100 ns; "coarsest" is usually 15.625 ms, "finest" 0.5 ms
    $snap.timer = [ordered]@{
        current_ms  = [math]::Round($c / 10000.0, 4)
        coarsest_ms = [math]::Round([math]::Max($a, $b) / 10000.0, 4)
        finest_ms   = [math]::Round([math]::Min($a, $b) / 10000.0, 4)
    }
} catch { [void]$snap.notes.Add("timer query failed: $($_.Exception.Message)"); $snap.timer = [ordered]@{} }

# ---------------- memory (shows RAM "cleaning") ----------------
$os = Get-CimInstance Win32_OperatingSystem
$snap.memory = [ordered]@{ freeMB = [math]::Round($os.FreePhysicalMemory / 1024); totalMB = [math]::Round($os.TotalVisibleMemorySize / 1024) }

if (-not $snap.isAdmin) { [void]$snap.notes.Add('not running as administrator: bcdedit and some processes are unreadable') }
$snap | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $Out -Encoding UTF8
Write-Host "wrote $Out ($($procs.Count) processes, $($svc.Count) services, $($reg.Count) registry entries, $($denied.Count) processes access-denied)"
foreach ($n in $snap.notes) { Write-Host "note: $n" }
