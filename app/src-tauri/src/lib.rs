use std::sync::{Arc, Mutex};

use openf75::{
    audio::AudioViz,
    device::{self, Colour, Device, Kind},
    layout::{EFFECTS, Effect, KEYS, Key, SLEEP_TABLE},
    macros::Macro,
    proto::{KeyAction, KeyCode, PerKey},
    storage::{self, Profile},
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command(async)]
fn effects() -> Vec<Effect> {
    EFFECTS.to_vec()
}

pub struct AppState {
    dev: Arc<Mutex<Option<Device>>>,
    viz: Mutex<Option<AudioViz>>,
}

fn err<E: std::fmt::Display>(e: E) -> String {
    format!("{e:#}")
}

impl AppState {
    // shortcut: any error drops the handle so the next call reconnects; costs one open per failure.
    fn with_dev<T>(&self, f: impl FnOnce(&Device) -> anyhow::Result<T>) -> Result<T, String> {
        let mut guard = self.dev.lock().map_err(err)?;
        if guard.is_none() {
            let dev = device::open().map_err(err)?;
            storage::ensure_first_backup(&dev).map_err(err)?;
            *guard = Some(dev);
        }
        match f(guard.as_ref().expect("device set above")) {
            Ok(v) => Ok(v),
            Err(e) => {
                *guard = None;
                Err(err(e))
            }
        }
    }
}

#[derive(Serialize)]
pub struct Status {
    connected: bool,
    kind: Option<&'static str>,
    fw: Option<u16>,
    psd: Option<String>,
    online: Option<bool>,
    oem_running: bool,
    writable_error: Option<String>,
}

#[derive(Serialize)]
pub struct Lighting {
    row: Option<u8>,
    /// (brightness, speed, palette nibble) per effect row, 0-based.
    slots: Vec<(u8, u8, u8)>,
    /// 7 colours per effect row, 0-based.
    palette: Vec<[[u8; 3]; 7]>,
    /// Indexed by matrix (126 entries).
    perkey: Vec<[u8; 3]>,
}

#[derive(Deserialize)]
pub struct MacroEntry {
    name: String,
    events: Vec<openf75::macros::MacroEvent>,
}

#[tauri::command(async)]
fn layout() -> Vec<Key> {
    KEYS.to_vec()
}

#[tauri::command(async)]
fn status(state: State<'_, AppState>) -> Result<Status, String> {
    let oem_running = device::oem_running();
    let disconnected = |e: String| Status {
        connected: false,
        kind: None,
        fw: None,
        psd: None,
        online: None,
        oem_running,
        writable_error: Some(e),
    };
    let mut guard = state.dev.lock().map_err(err)?;
    if guard.is_none() {
        match device::open() {
            Ok(d) => {
                storage::ensure_first_backup(&d).map_err(err)?;
                *guard = Some(d);
            }
            Err(e) => return Ok(disconnected(err(e))),
        }
    }
    let dev = guard.as_ref().expect("device set above");
    let online = match dev.online() {
        Ok(v) => v,
        Err(e) => {
            *guard = None;
            return Ok(disconnected(err(e)));
        }
    };
    let dev = guard.as_ref().expect("device still set");
    Ok(Status {
        connected: true,
        kind: Some(match dev.kind {
            Kind::Cable => "cable",
            Kind::Dongle => "dongle",
        }),
        fw: Some(dev.fw),
        psd: dev.psd.map(|p| {
            p.iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        }),
        online: Some(online),
        oem_running,
        writable_error: dev.writable_with(oem_running, online).err().map(err),
    })
}

#[tauri::command(async)]
fn read_lighting(state: State<'_, AppState>) -> Result<Lighting, String> {
    state.with_dev(|dev| {
        let led = dev.led()?;
        let table = dev.table()?;
        let pk = dev.perkey()?;
        Ok(Lighting {
            row: led.effect_row().map(|r| r as u8),
            slots: (0..20).map(|r| led.slot(r).unwrap_or((0, 0, 0))).collect(),
            palette: (0..20)
                .map(|r| std::array::from_fn(|s| table.get(r, s).unwrap_or([0; 3])))
                .collect(),
            perkey: (0..126).map(|m| pk.get(m as u8).unwrap_or([0; 3])).collect(),
        })
    })
}

#[tauri::command(async)]
fn apply_effect(
    state: State<'_, AppState>,
    row: usize,
    brightness: u8,
    speed: u8,
    colour: Colour,
) -> Result<(), String> {
    if row >= 20 || brightness > 9 || speed > 4 {
        return Err("efeito fora da faixa (linha 0-19, brilho 0-9, velocidade 0-4)".into());
    }
    state.with_dev(|dev| dev.apply_effect(row, brightness, speed, colour))
}

#[tauri::command(async)]
fn apply_perkey(state: State<'_, AppState>, colours: Vec<(u8, [u8; 3])>) -> Result<(), String> {
    if colours.iter().any(|(m, _)| *m >= 126) {
        return Err("matriz de tecla inválida (0-125)".into());
    }
    state.with_dev(|dev| {
        let mut pk = dev.perkey()?;
        for (m, rgb) in &colours {
            pk.set(*m, *rgb);
        }
        dev.apply_perkey(&pk)
    })
}

fn decode_layer(raw: &[u8]) -> Vec<KeyAction> {
    raw.chunks_exact(4)
        .map(|c| KeyCode([c[0], c[1], c[2], c[3]]).decode())
        .collect()
}

#[tauri::command(async)]
fn read_layer(state: State<'_, AppState>, layer: u8) -> Result<Vec<KeyAction>, String> {
    if layer > 1 {
        return Err("camada inválida (0 = base, 1 = Fn)".into());
    }
    state.with_dev(|dev| Ok(decode_layer(&dev.layer(layer)?)))
}

#[tauri::command(async)]
fn set_key(
    state: State<'_, AppState>,
    layer: u8,
    matrix: u8,
    action: KeyAction,
) -> Result<(), String> {
    if layer > 1 || matrix >= 128 {
        return Err("tecla ou camada inválida".into());
    }
    state.with_dev(|dev| {
        let mut data = dev.layer(layer)?;
        let i = matrix as usize * 4;
        data[i..i + 4].copy_from_slice(&KeyCode::encode(&action).0);
        dev.set_layer(layer, &data)
    })
}

#[tauri::command(async)]
fn list_macros() -> Result<Vec<Macro>, String> {
    storage::load_macros().map_err(err)
}

#[tauri::command(async)]
fn save_macros(state: State<'_, AppState>, macros: Vec<Macro>) -> Result<(), String> {
    state.with_dev(|dev| storage::save_macros(dev, &macros))
}

#[tauri::command(async)]
fn delete_macro(state: State<'_, AppState>, index: usize) -> Result<(), String> {
    state.with_dev(|dev| storage::delete_macro(dev, index))
}

#[tauri::command(async)]
fn set_sleep(state: State<'_, AppState>, secs: Option<u32>) -> Result<(), String> {
    let hw = match secs {
        None => 0,
        Some(s) => SLEEP_TABLE
            .iter()
            .find(|(v, _)| *v == s)
            .map(|(_, hw)| *hw)
            .ok_or("tempo de suspensão não suportado")?,
    };
    state.with_dev(|dev| {
        let mut led = dev.led()?;
        led.set_sleep_hw(hw);
        dev.set_led(&led)
    })
}

#[tauri::command(async)]
fn set_debounce(state: State<'_, AppState>, value: u8) -> Result<(), String> {
    if value > 4 {
        return Err("debounce deve ser 0-4".into());
    }
    state.with_dev(|dev| {
        let mut led = dev.led()?;
        led.set_debounce(value)?;
        dev.set_led(&led)
    })
}

#[tauri::command(async)]
fn get_power(state: State<'_, AppState>) -> Result<(Option<u32>, u8), String> {
    state.with_dev(|dev| {
        let led = dev.led()?;
        let hw = led.0[0x18];
        let secs = SLEEP_TABLE.iter().find(|(_, h)| *h == hw).map(|(s, _)| *s);
        Ok((secs, led.0[3]))
    })
}

#[tauri::command(async)]
fn factory_reset(state: State<'_, AppState>) -> Result<(), String> {
    state.with_dev(|dev| dev.factory_reset())
}

#[tauri::command(async)]
fn backup_now(state: State<'_, AppState>) -> Result<String, String> {
    state.with_dev(|dev| {
        let mut snap = dev.snapshot()?;
        snap.macros = storage::load_macros()?;
        Ok(storage::write_backup(&snap)?.display().to_string())
    })
}

#[tauri::command(async)]
fn restore_backup(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let snap = storage::load_snapshot(std::path::Path::new(&path)).map_err(err)?;
    state.with_dev(|dev| dev.restore(&snap))
}

#[tauri::command(async)]
fn list_profiles() -> Result<Vec<String>, String> {
    storage::list_profiles().map_err(err)
}

#[tauri::command(async)]
fn save_profile(state: State<'_, AppState>, name: String) -> Result<(), String> {
    state.with_dev(|dev| {
        let mut snapshot = dev.snapshot()?;
        snapshot.macros = storage::load_macros()?;
        storage::save_profile(&name, &snapshot)
    })
}

#[tauri::command(async)]
fn apply_profile(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let profile = storage::load_profile(&name).map_err(err)?;
    state.with_dev(|dev| {
        dev.restore(&profile.snapshot)?;
        storage::write_macros_file(&profile.snapshot.macros)
    })
}

#[tauri::command(async)]
fn delete_profile(name: String) -> Result<(), String> {
    storage::delete_profile(&name).map_err(err)
}

#[tauri::command(async)]
fn export_profile(name: String, path: String) -> Result<(), String> {
    let profile = storage::load_profile(&name).map_err(err)?;
    let json = serde_json::to_string_pretty(&profile).map_err(err)?;
    std::fs::write(&path, json).map_err(err)
}

#[tauri::command(async)]
fn import_profile(path: String) -> Result<String, String> {
    let text = std::fs::read_to_string(&path).map_err(err)?;
    let profile: Profile = serde_json::from_str(&text).map_err(err)?;
    let name = profile.name.clone();
    storage::save_profile(&profile.name, &profile.snapshot).map_err(err)?;
    Ok(name)
}

#[tauri::command(async)]
fn audio_start(state: State<'_, AppState>, gain: f32) -> Result<(), String> {
    let mut viz = state.viz.lock().map_err(err)?;
    if viz.is_some() {
        return Err("áudio já está ativo".into());
    }
    state.with_dev(|_| Ok(()))?;
    *viz = Some(AudioViz::start(Arc::clone(&state.dev), gain).map_err(err)?);
    Ok(())
}

#[tauri::command(async)]
fn audio_stop(state: State<'_, AppState>) -> Result<(), String> {
    let taken = state.viz.lock().map_err(err)?.take();
    if let Some(v) = taken {
        v.stop().map_err(err)?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            dev: Arc::new(Mutex::new(None)),
            viz: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            layout,
            effects,
            status,
            read_lighting,
            apply_effect,
            apply_perkey,
            read_layer,
            set_key,
            list_macros,
            save_macros,
            delete_macro,
            set_sleep,
            set_debounce,
            get_power,
            factory_reset,
            backup_now,
            restore_backup,
            list_profiles,
            save_profile,
            apply_profile,
            delete_profile,
            export_profile,
            import_profile,
            audio_start,
            audio_stop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
