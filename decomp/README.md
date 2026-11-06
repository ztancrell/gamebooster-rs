# decomp/

## Policy

Vendor code is decompiled only locally, under `/home/zpt/re/`, and only to research. The findings
are written here as prose (`notes.md`, `README.md`, `features.md`), with function addresses as
evidence; the decompiled or disassembled output and decoded vendor data are deleted afterwards and
never committed. Earlier commits that held such output were purged from the history on 2026-11-05.

## Products

Static analysis notes, one folder per company and product. Plain-language pages are in `docs/`.

| Company | Product | Decomp folder | Docs folder |
|---|---|---|---|
| IObit / iTop | Smart Game Booster 6.0 | [iobit/smart-game-booster/](iobit/smart-game-booster/README.md) | [docs/sgb/](../docs/sgb/README.md) |
| IObit | Advanced SystemCare Free 20 | [iobit/advanced-systemcare/](iobit/advanced-systemcare/README.md) | [docs/advanced-systemcare/](../docs/advanced-systemcare/README.md) |
| Skowsand Tecnologia | ExitLag | [skowsand/exitlag/](skowsand/exitlag/README.md) | [docs/exitlag/](../docs/exitlag/README.md) |
| AuraSide | Hone | [auraside/hone/](auraside/hone/README.md) | [docs/hone/](../docs/hone/README.md) |
| Wagnardsoft | Intelligent Standby List Cleaner (ISLC) | [wagnardsoft/islc/](wagnardsoft/islc/README.md) | [docs/islc/](../docs/islc/README.md) |
| PGWARE | GameBoost | [pgware/gameboost/](pgware/gameboost/README.md) | [docs/pgware/](../docs/pgware/README.md) |
| Bitsum Technologies | Process Lasso | [bitsum/process-lasso/](bitsum/process-lasso/README.md) | [docs/process-lasso/](../docs/process-lasso/README.md) |
| Razer | Cortex | [razer/cortex/](razer/cortex/README.md) | [docs/razer-cortex/](../docs/razer-cortex/README.md) |
| Smart PC Utilities | PC Game Booster | [smart-pc-utilities/game-booster/](smart-pc-utilities/game-booster/README.md) | [docs/smartpcutilities-gb/](../docs/smartpcutilities-gb/README.md) |

Generic helper scripts (`gdis.py`, `inno_carve.py`, `dfm_timers.py`) are in [tools/](tools/); see the
SGB README for what each does.
