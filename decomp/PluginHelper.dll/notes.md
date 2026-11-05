# PluginHelper.dll

Static strings, imports and exports only (objdump/strings); nothing executed. Carved `pe_0950a05e_137728.bin`. MSVC. PDB `C:\Users\r\Desktop\ChromeHMacGenerate\Release\ChromeHMacGenerate.pdb`.

Exports `GenerateHMac`, `GenerateHMacEx`, `GetMachineIDStr`. It computes the HMAC that Chrome uses to
protect `Secure Preferences` (keyed with the machine ID), so a third party can edit Chrome settings
without Chrome flagging them as tampered. SgbMain imports `GenerateHMac` and contains the string
`Secure Preferences`. What SgbMain changes in Chrome was not traced (likely the browser-cleaning or
add-on management feature). Medium confidence on purpose; the name and PDB are unambiguous about the
mechanism.
