# Accessibility shortcuts

**What it is.** Turns off the keyboard shortcuts that switch on three Windows accessibility
features, for the length of the boost:

- **Sticky Keys**: press Shift five times.
- **Filter Keys**: hold Right Shift for 8 seconds.
- **Toggle Keys**: hold Num Lock for 5 seconds.

gamebooster reads each feature's flags with `SystemParametersInfoW` (`SPI_GETSTICKYKEYS`,
`SPI_GETFILTERKEYS`, `SPI_GETTOGGLEKEYS`), saves the original `dwFlags` in the state file, and
clears only the "hotkey active" and "confirm hotkey" bits (`SKF_`/`FKF_`/`TKF_HOTKEYACTIVE` and
`..._CONFIRMHOTKEY`). If a feature itself is **on** (you use Sticky Keys, say), that feature is left
alone entirely; so is one whose shortcut is already off. Restore writes the saved `dwFlags` back
exactly, only for the features it changed.

**Session only.** The change is sent with `SPIF_SENDCHANGE` and without `SPIF_UPDATEINIFILE`, so
nothing is written to your profile (`HKCU\Control Panel\Accessibility`). If gamebooster or Windows
crashes before `restore`, signing out or rebooting brings your shortcuts back by itself.

**Windows default.** All three shortcuts are on (with a confirmation dialog); the features are off.

**Why it helps.** Not FPS: comfort. Mashing or holding Shift in a game (sprint, crouch) pops up the
Sticky Keys dialog and drops you to the desktop. The idea comes from Razer Cortex
([razer-cortex](../razer-cortex/README.md)).

**Exact setting.** `accessibility_shortcuts_off = true` in the config (the default; harmless and
reversible). Module: `src/accessibility.rs`.

## Try and undo by hand

Settings > Accessibility > Keyboard: switch off "Keyboard shortcut for Sticky keys", and the same
under Filter keys and Toggle keys. (That way is persistent; switch them back on to undo.)

## Measure

- `gamebooster boost --dry-run` and `gamebooster status` print, per feature,
  `accessibility: Sticky Keys shortcut off (flags 0x1fe restored)` or `... left alone`.
- While boosted, press Shift five times: no dialog. After `restore`, the dialog comes back.
