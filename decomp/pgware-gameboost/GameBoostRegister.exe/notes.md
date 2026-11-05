# GameBoostRegister.exe (and the licence check in GameBoost.exe)

Delphi VCL + Indy, PE32, 3,747,328 bytes, sha256 `6aa9423e08af86e12a3ef512e0eaaaa5972abc3dd43554f17c2d1493d769049c`,
unsigned. Status: **Strings** + imports, plus the `/online` routine in `GameBoost.exe` (0x6c63c4).
High level only, on purpose: no serial algorithm, no bypass details.

## Model

- **Trial**: timed ("will stop working in [days] days", then "has expired and has stopped working").
  In the trial the "Maximum Boost" setting is locked (its only extra is `MaxConnectionsPerServer` 8
  instead of 4, see [GameBoost.exe notes](../GameBoost.exe/notes.md)).
- **Paid**: name + serial, bought on pgware.com (card or PayPal:
  `…/store/?product=gameboost&type=paypal`), or a "FREE serial" through **TrialPay** offers
  (`…/store/?product=gameboost&type=trialpay&campaign=trial`). Serials have a per-serial machine
  limit ("activation limit for this serial number").
- Stored in `HKLM\Software\PGWARE\GameBoost\Name` / `Serial`.

## Online check (`GameBoost.exe /online`, started on every launch)

- GET `http://www.pgware.com/register/?product=gameboost&serial=[serial]&machine=[machine]&version=3.10.21.2024`
  over **plain HTTP**. `[machine]` is a machine ID; `GetVolumeInformationW` (volume serial) is the
  only hardware-ID API imported.
- Server answer `PGWARE|0` → the app deletes `Name` and `Serial` (remote revocation).
- Another answer path opens the browser on `http://www.pgware.com/piracy/report/?product=gameboost&[data]`,
  and the activation form's `MessageBoxPiracy` says "The name and serial number you have entered is
  illegal, you have been reported!".

## Telemetry

Nothing beyond the above: no analytics, no crash reporter, no hardware inventory upload. Per launch:
the licence GET (serial, empty when unregistered unless the code skips it, which was not traced, plus machine ID) and at most every 45 days the version GET.
