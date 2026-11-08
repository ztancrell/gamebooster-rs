//! The Mac power fix's GPU part: a soft PowerPlay table for the Radeon Pro 5500M with only the
//! power limit changed to [`GPU_WATTS`], built at run time from the card's **stock** table (no
//! undervolt, no other edits; the exe embeds no vendor table bytes). See
//! docs/superpowers/specs/2026-11-08-mac-power-fix-design.md.
//!
//! Not built yet: where the stock table is read from and which offsets to patch are being
//! researched separately (findings go in docs/tweaks/gpu-power-table.md). [`build`] is the seam: everything around it (the
//! Mac gate, the backup of the old value in the install record, the registry write, the GPU
//! restart, `uninstall`) lives in `crate::mac` and is ready.

/// GPU power limit (PPT) the table sets. Fallback 30 W if the confirmation run crosses the SMC budget.
pub const GPU_WATTS: u32 = 32;

/// The registry value (REG_BINARY) under the GPU's display class key that the AMD driver reads at start.
pub const VALUE: &str = "PP_PhmSoftPowerPlayTable";

/// The patched table to write as [`VALUE`] under `class_key` (`SYSTEM\...\Class\{4d36e968-...}\NNNN`).
pub fn build(_class_key: &str) -> Result<Vec<u8>, String> {
    Err("GPU power table not built yet".into())
}
