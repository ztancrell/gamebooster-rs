# Sngulc.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_0c8e65c7_1665592.bin`. Delphi, "Smart Game Booster Sngulc".

Exports `InitSngulc`, `VerifyGameNode`, `FreeSngulc`. Strings: `Ngulocal.json`, `game_id`,
`GameType`, `GamePathAndFileName`, `GameIDs=`, `com.epicgames.launcher://apps/`, `SmartGameBooster.ini`,
`api.gameboosta.com`, AntiBan lists. Verifies/normalises game-library entries (launcher URIs, IDs)
against a list fetched from the vendor API and cached in `Ngulocal.json`. SgbMain loads it. Medium-low
confidence on purpose (strings only).
