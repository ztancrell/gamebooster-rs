<#
.SYNOPSIS
  Prints what changed between two snapshot.ps1 JSON files, grouped by category.
.EXAMPLE
  .\compare.ps1 -Before before.json -After after.json > diff.txt
#>
param(
    [Parameter(Mandatory = $true)][string]$Before,
    [Parameter(Mandatory = $true)][string]$After
)

$A = Get-Content -LiteralPath $Before -Raw | ConvertFrom-Json
$B = Get-Content -LiteralPath $After -Raw | ConvertFrom-Json

function ToMap($o) {
    $h = @{}
    if ($o) { foreach ($p in $o.PSObject.Properties) { $h[$p.Name] = "$($p.Value)" } }
    return $h
}

# Prints added (+), removed (-) and changed (~) keys of two flat objects.
function Diff-Map($title, $a, $b) {
    "== $title =="
    $ma = ToMap $a; $mb = ToMap $b; $n = 0
    foreach ($k in (@($ma.Keys) + @($mb.Keys) | Sort-Object -Unique)) {
        if (-not $ma.ContainsKey($k))     { "  + $k = $($mb[$k])"; $n++ }
        elseif (-not $mb.ContainsKey($k)) { "  - $k (was $($ma[$k]))"; $n++ }
        elseif ($ma[$k] -ne $mb[$k])      { "  ~ ${k}: $($ma[$k]) -> $($mb[$k])"; $n++ }
    }
    if ($n -eq 0) { '  (no changes)' }
    ''
}

"Before: $($A.timestamp)  admin=$($A.isAdmin)"
"After:  $($B.timestamp)  admin=$($B.isAdmin)"
foreach ($s in @($A, $B)) { foreach ($note in $s.notes) { "note ($($s.timestamp)): $note" } }
''

'== Power =='
if ($A.power.activeScheme -ne $B.power.activeScheme) { "  ~ active scheme: $($A.power.activeScheme) -> $($B.power.activeScheme)" }
else { "  active scheme unchanged: $($A.power.activeScheme)" }
''
Diff-Map 'Power schemes (powercfg /list)' $A.power.schemes $B.power.schemes
Diff-Map 'Power settings of active scheme' $A.power.settings $B.power.settings
Diff-Map 'Services (state / start mode)' $A.services $B.services

'== Processes =='
$pa = @{}; foreach ($p in $A.processes) { $pa["$($p.name)|$($p.pid)"] = $p }
$pb = @{}; foreach ($p in $B.processes) { $pb["$($p.name)|$($p.pid)"] = $p }
$n = 0
foreach ($k in ($pa.Keys | Sort-Object)) {
    if (-not $pb.ContainsKey($k)) { $p = $pa[$k]; "  stopped: $($p.name) (pid $($p.pid)) $($p.path)"; $n++ }
}
foreach ($k in ($pb.Keys | Sort-Object)) {
    $p = $pb[$k]
    if (-not $pa.ContainsKey($k)) { "  started: $($p.name) (pid $($p.pid)) priority=$($p.priority) affinity=$($p.affinity) $($p.path)"; $n++; continue }
    $o = $pa[$k]
    if ("$($o.priority)" -ne "$($p.priority)") { "  ~ $($p.name) (pid $($p.pid)) priority: $($o.priority) -> $($p.priority)"; $n++ }
    if ("$($o.affinity)" -ne "$($p.affinity)") { "  ~ $($p.name) (pid $($p.pid)) affinity: $($o.affinity) -> $($p.affinity)"; $n++ }
}
if ($n -eq 0) { '  (no changes)' }
"  (unreadable processes: before $(@($A.processesAccessDenied).Count), after $(@($B.processesAccessDenied).Count))"
''

Diff-Map 'Registry' $A.registry $B.registry
Diff-Map 'Windows Update (Automatic Updates settings)' $A.windowsUpdate $B.windowsUpdate
Diff-Map 'BCD (bcdedit /enum {current})' $A.bcd $B.bcd
Diff-Map 'Timer resolution' $A.timer $B.timer
Diff-Map 'Memory' $A.memory $B.memory
