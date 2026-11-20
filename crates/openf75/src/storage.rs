//! Local files under `%APPDATA%\openf75\`: macro library, profiles, backups.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use crate::device::{Device, Snapshot};
use crate::layout;
use crate::macros::{encode_buffer, Macro};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub name: String,
    pub snapshot: Snapshot,
}

fn base_dir() -> Result<PathBuf> {
    let appdata = std::env::var("APPDATA").context("variável APPDATA ausente")?;
    let dir = PathBuf::from(appdata).join("openf75");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn sub_dir(name: &str) -> Result<PathBuf> {
    let dir = base_dir()?.join(name);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn backup_dir() -> Result<PathBuf> {
    sub_dir("backups")
}

fn profiles_dir() -> Result<PathBuf> {
    sub_dir("profiles")
}

fn macros_path() -> Result<PathBuf> {
    Ok(base_dir()?.join("macros.json"))
}

/// Macro library saved on disk; empty when the file does not exist yet.
pub fn load_macros() -> Result<Vec<Macro>> {
    let p = macros_path()?;
    if !p.exists() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&fs::read_to_string(&p)?)?)
}

/// Writes the macro library file only; does not touch the keyboard.
pub fn write_macros_file(macros: &[Macro]) -> Result<()> {
    fs::write(macros_path()?, serde_json::to_string_pretty(macros)?)?;
    Ok(())
}

/// Encodes the library, writes it to the keyboard, then persists the file.
pub fn save_macros(dev: &Device, macros: &[Macro]) -> Result<()> {
    let buf = encode_buffer(macros)?;
    dev.set_macros(&buf)?;
    write_macros_file(macros)
}

/// Removes macro `i`, refusing if a key still uses it; shifts later indices down.
pub fn delete_macro(dev: &Device, i: usize) -> Result<()> {
    dev.writable()?;
    let mut macros = load_macros()?;
    ensure!(i < macros.len(), "macro inexistente: {i}");

    let mut in_use = Vec::new();
    let mut layers = [[0u8; 512]; 2];
    for n in 0..2u8 {
        layers[n as usize] = dev.layer(n)?;
        for (pos, c) in layers[n as usize].chunks_exact(4).enumerate() {
            if c[0] == 0x03 && c[3] as usize == i {
                let name = layout::KEYS
                    .iter()
                    .find(|k| k.matrix as usize == pos)
                    .map(|k| k.name.to_string())
                    .unwrap_or_else(|| format!("slot {pos}"));
                in_use.push(if n == 1 { format!("Fn+{name}") } else { name });
            }
        }
    }
    if !in_use.is_empty() {
        bail!("macro em uso nas teclas: {}", in_use.join(", "));
    }

    for n in 0..2u8 {
        let mut d = layers[n as usize];
        for c in d.chunks_exact_mut(4) {
            if c[0] == 0x03 && c[3] as usize > i {
                c[3] -= 1;
            }
        }
        dev.set_layer(n, &d)?;
    }
    macros.remove(i);
    save_macros(dev, &macros)
}

/// Saves the current device state once, if no first backup exists yet.
pub fn ensure_first_backup(dev: &Device) -> Result<()> {
    let p = backup_dir()?.join("first.json");
    if p.exists() {
        return Ok(());
    }
    let snap = dev.snapshot()?;
    fs::write(&p, serde_json::to_string_pretty(&snap)?)?;
    Ok(())
}

/// Writes a timestamped backup file and returns its path.
pub fn write_backup(snap: &Snapshot) -> Result<PathBuf> {
    let p = backup_dir()?.join(format!("{}.json", timestamp()));
    fs::write(&p, serde_json::to_string_pretty(snap)?)?;
    Ok(p)
}

pub fn load_snapshot(path: &Path) -> Result<Snapshot> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn check_name(name: &str) -> Result<()> {
    let bad = name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\', ':']);
    ensure!(!bad, "nome de perfil inválido: {name:?}");
    Ok(())
}

fn profile_path(name: &str) -> Result<PathBuf> {
    check_name(name)?;
    Ok(profiles_dir()?.join(format!("{name}.json")))
}

pub fn list_profiles() -> Result<Vec<String>> {
    let mut names = Vec::new();
    for e in fs::read_dir(profiles_dir()?)? {
        let p = e?.path();
        if p.extension().is_some_and(|x| x == "json") {
            if let Some(s) = p.file_stem() {
                names.push(s.to_string_lossy().into_owned());
            }
        }
    }
    names.sort();
    Ok(names)
}

pub fn save_profile(name: &str, snap: &Snapshot) -> Result<()> {
    let p = profile_path(name)?;
    let prof = Profile {
        name: name.to_string(),
        snapshot: snap.clone(),
    };
    fs::write(p, serde_json::to_string_pretty(&prof)?)?;
    Ok(())
}

pub fn load_profile(name: &str) -> Result<Profile> {
    let p = profile_path(name)?;
    ensure!(p.exists(), "perfil inexistente: {name}");
    Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
}

pub fn delete_profile(name: &str) -> Result<()> {
    let p = profile_path(name)?;
    fs::remove_file(&p).map_err(|e| anyhow!("apagar perfil {name}: {e}"))
}

/// `YYYYMMDD-HHMMSS` in UTC, without a date crate (Hinnant civil-from-days).
fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}{m:02}{d:02}-{:02}{:02}{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}
