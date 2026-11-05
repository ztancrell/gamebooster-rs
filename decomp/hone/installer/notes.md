# Hone installer: notes

Our own words, from static reading; nothing was run. See [../README.md](../README.md) for hashes.

## Stage 1: Overwolf web installer (`Hone - Installer.exe`, 2.316.0.1)

- An NSIS stub signed by Overwolf Ltd. It unpacks a .NET installer (`OWInstaller.exe` with WebView2,
  DotNetZip, TaskScheduler, Sentry (SharpRaven), log4net and websocket-sharp) and a small HTML UI. The
  UI includes a consent page (`cmp.html`), Overwolf promo images and a game-detection list.
- It asks for elevation through the NSIS UAC plug-in and then runs `OWInstaller.exe` with Hone's
  branding: silent setup, the Hone EULA and privacy links, no choice of install location, no Overwolf
  shortcut UI, an app shortcut, and `--owelectronUrl` pointing at Overwolf's per-app setup redirect
  (see the parent README).
- **Install telemetry:** the NSIS script sends counters (run failure, the user's UAC choice) to
  `analyticsnew.overwolf.com` over **plain HTTP**, with the installer version. The .NET installer has
  more events (download progress, bandwidth audit, an "electron muid" machine id) and Sentry error
  reports.
- **Promotion:** the arguments enable custom promo pages and a partner customisation level. The
  installer's strings name a "CRI" flow (an extra Overwolf-recommended app downloaded and installed
  alongside, as user or elevated). This is Overwolf's cross-promotion mechanism. Which app, if any, is
  offered depends on the server and was not observed.
- **Download check:** the setup is downloaded over HTTPS and compared with a server-supplied MD5 or
  SHA-1 checksum. The strings show no Authenticode check of the downloaded setup.
- The .NET installer could not be decompiled (ilspycmd found no readable managed metadata), so these
  points come from its strings and the NSIS script.

## Stage 2: Hone setup (`setup.exe`, electron-builder NSIS, 2.1.0)

- It installs the ow-electron host `Hone.exe` and `resources/app.asar`, plus
  `resources/assets/hone-resources/`. That folder holds the helper programs (see
  [../helpers/notes.md](../helpers/notes.md)), three power plan files (`HoneV5.pow` "Hone Ultimate
  Power Plan V4", plus Balanced and High performance copies), two NVIDIA Profile Inspector profiles
  with `nvidiaProfileInspector.zip`, 7-Zip, `TotalReg.exe` (a third-party registry editor) and
  `HWiNFO64.dll`. `resources/openmouse-host/` is an AGPL Node helper that talks to gaming mice over HID.
- The installer itself adds **no service, driver or Run key**. Those come later from the app: HoneSvc
  is created by HoneUtility when the device-affinity or MSI optimization is applied (see helpers). The
  app also turns its own "open at login" setting **off** at start.
- Uninstall runs `uninstall-event.bat`. It reads the Hone user id from `%APPDATA%\hone\user_id.txt`,
  posts a `hone_uninstalled` event to Mixpanel with curl, and deletes the id file. The event is sent
  even though the app is being removed.
- The NSIS script of stage 2 was not unpacked (7-Zip gives files only). Install scope (per-user under
  `%LOCALAPPDATA%\Programs` is the electron-builder default) is not verified. The elevation helper is
  also copied to `%APPDATA%\ow-electron\<app id>\` at run time.
