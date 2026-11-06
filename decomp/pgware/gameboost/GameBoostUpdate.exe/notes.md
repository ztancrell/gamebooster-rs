# GameBoostUpdate.exe

Delphi VCL + Indy, PE32, 3,907,584 bytes, sha256 `d82f4e4a8df0f26c6f830f27ff08525693419748259ccfc834edffb0b181327b`,
unsigned. Status: **Targeted** (strings, imports, flow; not every function read).

## Flow

1. Admin check (`MessageBoxAdmin1/2`: "Please login … as an Administrator").
2. Version check: `http://www.pgware.com/downloads/update/?product=gameboost&version=3.10.21.2024`,
   User-Agent `Mozilla/4.0 (compatible; PGWARE)`, answer `PGWARE|0/1/2` (same as `GameBoost.exe /checkupdate`).
3. Download: `http://www.pgware.com/downloads/gameboost.exe` (the full Inno installer) to a temp
   file (`update*.*` cleanup pattern).
4. Run it with `ShellExecuteExW`, arguments `/NORESTART /RESTARTEXITCODE=99`; exit code 99 →
   "restart Windows now?" (`ExitWindowsEx`).
5. On failure: "manually download" opens `http://www.pgware.com/downloads/`.

## Security

- **Plain HTTP** for both the version check and the installer download (Indy's OpenSSL support is
  linked in but the URLs are `http://`).
- **No integrity check**: no `WinVerifyTrust`, no Crypt* imports, no hash compare; the installer is
  not Authenticode-signed anyway.
- Runs elevated (admin) and executes what it downloaded.

Result: anyone who can tamper with the connection (public Wi-Fi, rogue DNS, compromised router) can
make it run any program as administrator. This is the most serious finding in the product.
Compare SGB's `SgbUpdater.exe`: HTTPS manifest plus hash and signature checks.
