# iTopClientExt.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_0d3ea4aa_2049080.bin`. Delphi, "Smart Game Booster Client Module".

Exports `HttpPostTntCtEx`, `FreeData`. Carries a list of BitTorrent/P2P clients
(`main_launcher=uTorrent.exe|BitTorrent.exe`, `qbittorrent`, `deluge.exe`, `emule.exe`, `fdm.exe`, …),
`feedback.php`, `install.php`, `api.gameboosta.com` and the AntiBan lists. Shared iTop component:
an HTTP-post helper plus detection of installed P2P clients (iTop VPN heritage). SgbMain loads it
(`LoadiTopClientExt error:`). Exact use in SGB not traced.
