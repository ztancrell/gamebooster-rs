# decomp/process-lasso/

Static analysis of **Process Lasso** (Bitsum LLC, https://bitsum.com/). Done to compare it with the
other boosters in this folder, and mainly to learn how **ProBalance** works: it lowers background
processes for a short time, only while they hog the CPU. Our `lower_others` lowers them once, for the
whole boost. Nothing was executed (no Wine). Plain-language overview and adoption proposal:
[docs/process-lasso/README.md](../../docs/process-lasso/README.md).

## Policy

This folder holds **findings in prose only**: no decompiled code, excerpts, scripts, keys or Bitsum
assets. Decompiled and extracted output lived only in the local work directory
`/home/zpt/re/process-lasso/` and was deleted after these notes were written. Only the downloaded
installer remains there (`/home/zpt/re/process-lasso/dl/`). INI key names and Windows power-setting
GUIDs are named because they are public (Bitsum's INI reference, Microsoft's power settings), not
copied code.

## Source

| What | Value |
|---|---|
| Downloaded | 2026-11-05, from the "Stable" link on `https://bitsum.com/download-process-lasso/`: `https://dl.bitsum.com/files/processlassosetup64.exe` (direct, HTTP 200, no redirect) |
| Installer | `processlassosetup64.exe`, 3,787,792 bytes, sha256 `92cc0aaaaaadca6d54cba2ce3280f9b1ed0f9849a9869de13f56da21c2e2804b`. NSIS, 32-bit stub, file/product version **19.0.0.94** (released 2026-11-02 per the download page) |
| Signing | Authenticode **Bitsum Technologies (Bitsum LLC)**, Morristown, Tennessee, US; DigiCert Trusted G4 Code Signing RSA4096 SHA384 2021 CA1; valid 2025-01-12 to 2028-03-28; serial `08c1a22680abe5ed39c3d67663fe2715`; signature valid per binary-refinery `pemeta`. The governor, GUI and IRQControl carry the same signature |
| Tools | binary-refinery (`emit`, `pemeta`, `xtnsis`), 7-Zip (NSIS payload), `strings`, `objdump`, `decomp/gdis.py`, Ghidra 11 headless (two runs on `ProcessGovernor.exe`, one throwaway post-script that decompiled the functions referencing chosen strings or addresses) |
| Public docs read | bitsum.com: download page, ProBalance algorithm page, "How ProBalance works", INI configuration reference, setup guide, FAQ, Performance Mode / BHP page |

Key component hashes (sha256), all 64-bit native C++ (no .NET, no packer):

| File | Version | Size | sha256 |
|---|---|---|---|
| `ProcessGovernor.exe` (core engine) | 19.0.0.94 | 1,610,232 | `96d914a2d702cbdbcfd391c6869d20d489d5ea113b20eff2c9453d51b619003d` |
| `ProcessLasso.exe` (GUI) | 19.0.0.94 | 3,225,584 | `ac63864daf0fae6f82477336349350a37e283f31696a589e6c7dc1e22b7db7f2` |
| `InstallHelper.exe` | 19.0.0.94 | 1,113,080 | `2adb8fc2c499e09eafae66208b9162d1f5bcada96efadee5befda704f632754b` |
| `srvstub.exe` (service wrapper) | 19.0.0.94 | 135,160 | `03b165b76a831de8fb3d406e4a85366d3a57b4b02ef661489d60f85fa8780619` |
| `bitsumsessionagent.exe` | 19.0.0.94 | 184,824 | `82cf16e36e18a6b8d7ae9cbfb6c7c50264aca00f2202d07ba2913e297698f9f4` |
| `ProcessLassoLauncher.exe` | | 420,344 | `dcf3b608c60370751f37dae437b52c7b4d9893a7cb3e59cc99cdc2bc79d0c8dc` |
| `QuickUpgrade.exe` (updater) | | 541,168 | `febc78cb439bb3de3c171571662eddfdf3b635776357ca3ae7dc49dc49e2930d` |
| `plActivate.exe` (licence) | | 260,088 | `2eaead47faa743f636cbd2c4b72611c7fd06a62dcfcaad5315759f790f651d78` |
| `TweakScheduler.exe` | | 275,440 | `d3b86ee492e22d32fe5f0dff9bced79b196253a8e1378b85da6271e890ba26b7` |
| `IRQControl.exe` | 1.0.0.26 | 469,488 | `b97526070b582fa0f371791a9d7c923b3e66cfe85e3619873118faa473dba52e` |
| `Insights.exe` | | 1,026,544 | `62f3fb48d8a47a8c51ca968c78c361e04b6ca87b497803dc43f106096526a452` |

Also in the package: CPUEater (ProBalance demo load), ThreadRacer, LatencyTest, LogViewer,
testlasso, `vistammsc.exe`, 19 language DLLs, `start-governor.bat` / `stop-governor.bat` /
`pl.cmd`, licences, uninstaller. **No `.sys` file: Process Lasso ships no kernel driver.**

## Component index

| Component | What | Status |
|---|---|---|
| [Core engine (ProcessGovernor)](governor/notes.md) | ProBalance (exact defaults and restraint/restore logic), Performance Mode and the "Bitsum Highest Performance" plan, IdleSaver, SmartTrim, CPU Limiter, persistent rules, foreground boost, Steam game detection. | **Targeted**: config loader/saver and BHP creator decompiled; the main loop (too large for Ghidra's decompiler) read as disassembly around the ProBalance checks; strings and English resource text for the rest. Deleted. |
| [Installer, service, licence, updates](installer/notes.md) | NSIS script, what gets installed and started, service vs logon task, session agent, Free vs Pro and the nag, update and licence endpoints, telemetry claims. | **Targeted**: NSIS script extracted and read for actions; InstallHelper, QuickUpgrade and GUI strings/imports. Deleted. |
