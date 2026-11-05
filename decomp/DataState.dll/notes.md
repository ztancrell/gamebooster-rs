# DataState.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Plus disassembly of the three exports (`gdis.py`). Carved `pe_07cdf3ee_76288.bin` (same size).
MSVC, 32-bit, built 2014-07-09, version info "Smart Game Booster / DataState".

**What it is**: a tiny named-pipe IPC helper. Imports only kernel32 (pipe/file/thread calls) and
advapi32 (`ConvertStringSecurityDescriptorToSecurityDescriptorA`, `GetSecurityDescriptorSacl`,
`SetSecurityInfo`).

| Export | RVA | What it does |
|---|---|---|
| `Server(name)` | `0x2182` | Builds `\\.\pipe\datastate_<name>`; if a client can already open it (`CreateFileW` ok) returns false (another server owns it). Otherwise `CreateNamedPipeW` (duplex, message mode) and, on Vista+, applies SACL `S:(ML;;NW;;;LW)` (low integrity label, so low-IL processes may connect); starts a listener thread (`0x10001065`). |
| `Func(name, …)` | `0x1173` | Client side: `CreateFileW` on the pipe, writes a request, `ReadFile` the reply, closes. |
| `Clear` | `0x1c7d` | Tears down the server state under its critical section. |

**Who loads it**: SgbMain (dynamic, `datastate.dll` + `Clear`/`Func`/`Server`, strings near file
offset `0x65dc84`/`0x65e0c0`) and SgbMini (`Func`). SgbMain, SgbMini, SgbTray and SgbInit also contain
a Delphi unit `WDataState` that speaks the same pipes natively (`\\.\pipe\datastate_`,
`\\.\pipe\datastateL_`, e.g. SgbTray file offsets `0x18eea4`/`0x18eedc`).

**Purpose** (medium confidence): inter-process "state" channel between the SGB processes (main
window, tray, mini window, init), e.g. passing commands/status between instances. Message format not
decoded. Local only; no network.
