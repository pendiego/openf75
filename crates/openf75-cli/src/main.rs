use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};

use openf75::audio::AudioViz;
use openf75::device::{self, Colour, Device, Kind, Snapshot};
use openf75::layout::{self, EFFECTS, KEYS, SLEEP_TABLE};
use openf75::macros::{text_to_events, Macro};
use openf75::proto::{KeyAction, KeyCode, PerKey};
use openf75::storage;

#[derive(Parser)]
#[command(
    name = "openf75",
    version,
    about = "Configurador aberto do teclado AULA F75"
)]
struct Cli {
    #[command(subcommand)]
    command: Command_,
}

#[derive(Subcommand)]
#[allow(non_camel_case_types)]
enum Command_ {
    /// Mostra conexão, firmware, modelo, efeito, sleep e debounce
    Status,
    /// Exporta o estado completo do teclado (JSON)
    Dump {
        /// Arquivo de saída; sem ele, imprime no stdout
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Restaura um estado exportado por `dump`
    Restore { file: PathBuf },
    /// Define efeito de iluminação
    Effect {
        /// Número da linha (1-20) ou nome do efeito (ex.: Respire)
        effect: String,
        /// Brilho 0-9
        #[arg(long)]
        brightness: Option<u8>,
        /// Velocidade 0-4
        #[arg(long)]
        speed: Option<u8>,
        /// Slot da paleta 0-6
        #[arg(long)]
        slot: Option<u8>,
        /// Cor RRGGBB gravada no slot selecionado
        #[arg(long)]
        rgb: Option<String>,
        /// Cor aleatória
        #[arg(long)]
        random: bool,
    },
    /// Define cores por tecla (efeito Self-define)
    Perkey {
        /// Cor RRGGBB para todas as teclas
        #[arg(long)]
        all: Option<String>,
        /// Pares TECLA=RRGGBB
        #[arg(value_name = "KEY=RRGGBB")]
        pairs: Vec<String>,
    },
    /// Remapeia uma tecla na camada base ou Fn
    Remap {
        /// base | fn
        layer: String,
        /// Nome da tecla (ex.: CapsLock)
        key: String,
        /// key:<usage hex>[+ctrl|shift|alt|win], media:<usage hex>, macro:<n>[:times:N|:hold|:any], off, raw:XXXXXXXX
        target: String,
    },
    /// Grava texto como macro e atribui à tecla (camada base)
    MacroText { key: String, text: String },
    /// Tempo de sleep em segundos (60..1200 da tabela) ou never
    Sleep { value: String },
    /// Debounce 0-4
    Debounce { value: u8 },
    /// Restaura o padrão de fábrica
    Reset {
        /// Confirma a operação
        #[arg(long)]
        yes: bool,
    },
    /// Iluminação reativa ao áudio do sistema
    Audio {
        /// Duração em segundos
        #[arg(long)]
        seconds: u64,
        /// Ganho 0.5-4.0
        #[arg(long, default_value_t = 1.5)]
        gain: f32,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli.command) {
        eprintln!("erro: {e:#}");
        std::process::exit(1);
    }
}

fn run(cmd: Command_) -> Result<()> {
    match cmd {
        Command_::Audio { seconds, gain } => audio(seconds, gain),
        other => {
            let dev = open_dev()?;
            match other {
                Command_::Status => status(&dev),
                Command_::Dump { out } => dump(&dev, out),
                Command_::Restore { file } => restore(&dev, &file),
                Command_::Effect {
                    effect,
                    brightness,
                    speed,
                    slot,
                    rgb,
                    random,
                } => effect_cmd(&dev, &effect, brightness, speed, slot, rgb, random),
                Command_::Perkey { all, pairs } => perkey(&dev, all, &pairs),
                Command_::Remap { layer, key, target } => remap(&dev, &layer, &key, &target),
                Command_::MacroText { key, text } => macro_text(&dev, &key, &text),
                Command_::Sleep { value } => sleep(&dev, &value),
                Command_::Debounce { value } => debounce(&dev, value),
                Command_::Reset { yes } => reset(&dev, yes),
                Command_::Audio { .. } => unreachable!(),
            }
        }
    }
}

fn open_dev() -> Result<Device> {
    let dev = device::open()?;
    storage::ensure_first_backup(&dev)?;
    Ok(dev)
}

fn status(dev: &Device) -> Result<()> {
    let conn = match dev.kind {
        Kind::Cable => "Cabo",
        Kind::Dongle => "Receptor 2.4G",
    };
    println!("conexão: {conn}");
    println!("firmware: {}", dev.fw);
    match dev.psd {
        Some(p) => println!("psd {}", hex(&p)),
        None => println!("psd: desconhecido"),
    }
    println!("online: {}", sim_nao(dev.online()?));
    println!("OEM aberto: {}", sim_nao(oem_running()));

    let led = dev.led()?;
    match led.effect_row() {
        Some(row) => {
            let e = &EFFECTS[row];
            let (brightness, speed, nibble) = led.slot(row).unwrap_or((0, 0, 0));
            let colour = if nibble == 7 {
                "aleatória".to_string()
            } else {
                let [r, g, b] = dev.table()?.get(row, nibble.into()).unwrap_or([0; 3]);
                format!("slot {nibble} ({r:02X}{g:02X}{b:02X})")
            };
            println!(
                "efeito: {}, brilho {brightness}, velocidade {speed}, cor {colour}",
                e.name
            );
        }
        None => println!("efeito: desconhecido (hw {})", led.0[10]),
    }
    let sleep_hw = led.0[0x18];
    match SLEEP_TABLE.iter().find(|(_, hw)| *hw == sleep_hw) {
        _ if sleep_hw == 0 => println!("sleep: nunca"),
        Some((secs, _)) => println!("sleep: {secs} s"),
        None => println!("sleep: desconhecido (hw {sleep_hw})"),
    }
    println!("debounce: {}", led.0[3]);
    Ok(())
}

fn dump(dev: &Device, out: Option<PathBuf>) -> Result<()> {
    let snap = dev.snapshot()?;
    let json = serde_json::to_string_pretty(&snap)?;
    match out {
        Some(path) => {
            fs::write(&path, json).with_context(|| format!("gravar {}", path.display()))?;
            println!("estado salvo em {}", path.display());
        }
        None => println!("{json}"),
    }
    Ok(())
}

fn restore(dev: &Device, file: &PathBuf) -> Result<()> {
    let text = fs::read_to_string(file).with_context(|| format!("ler {}", file.display()))?;
    let snap: Snapshot = serde_json::from_str(&text).context("arquivo de backup inválido")?;
    dev.restore(&snap)?;
    println!("estado restaurado de {}", file.display());
    Ok(())
}

fn effect_cmd(
    dev: &Device,
    name: &str,
    brightness: Option<u8>,
    speed: Option<u8>,
    slot: Option<u8>,
    rgb: Option<String>,
    random: bool,
) -> Result<()> {
    let row = find_effect(name)?;
    let e = &EFFECTS[row];
    if brightness.is_some_and(|b| b > 9) {
        bail!("brilho deve ser 0-9");
    }
    if speed.is_some_and(|s| s > 4) {
        bail!("velocidade deve ser 0-4");
    }
    if slot.is_some_and(|s| s > 6) {
        bail!("slot deve ser 0-6");
    }
    if brightness.is_some() && !e.brightness {
        bail!("efeito {} não tem brilho", e.name);
    }
    if speed.is_some() && !e.speed {
        bail!("efeito {} não tem velocidade", e.name);
    }
    let colour_flags = [slot.is_some(), rgb.is_some(), random]
        .iter()
        .filter(|f| **f)
        .count();
    if colour_flags > 1 {
        bail!("use apenas um de --slot, --rgb ou --random");
    }
    if colour_flags > 0 && !e.colour {
        bail!("efeito {} não tem cor", e.name);
    }

    let led = dev.led()?;
    let (cur_brightness, cur_speed, cur_nibble) =
        led.slot(row.try_into()?).unwrap_or((0, 0, 0));
    let colour = if let Some(s) = slot {
        Colour::Slot(s)
    } else if let Some(hex) = rgb {
        Colour::Custom(parse_rgb(&hex)?)
    } else if random {
        Colour::Random
    } else if cur_nibble == 7 {
        Colour::Random
    } else {
        Colour::Slot(cur_nibble)
    };
    dev.apply_effect(
        row.try_into()?,
        brightness.unwrap_or(cur_brightness).into(),
        speed.unwrap_or(cur_speed).into(),
        colour,
    )?;
    println!("efeito aplicado: {}", e.name);
    Ok(())
}

fn perkey(dev: &Device, all: Option<String>, pairs: &[String]) -> Result<()> {
    let mut pk: PerKey = dev.perkey()?;
    if let Some(hex) = all {
        let rgb = parse_rgb(&hex)?;
        for key in KEYS.iter() {
            pk.set(key.matrix, rgb);
        }
    }
    for pair in pairs {
        let (name, hex) = pair
            .split_once('=')
            .ok_or_else(|| anyhow!("use TECLA=RRGGBB, recebido: {pair}"))?;
        let key = layout::by_name(name).ok_or_else(|| anyhow!("tecla desconhecida: {name}"))?;
        pk.set(key.matrix, parse_rgb(hex)?);
    }
    dev.apply_perkey(&pk)?;
    println!("cores por tecla aplicadas");
    Ok(())
}

fn remap(dev: &Device, layer: &str, key: &str, target: &str) -> Result<()> {
    let n: u8 = match layer {
        "base" => 0,
        "fn" => 1,
        _ => bail!("camada deve ser base ou fn"),
    };
    let key = layout::by_name(key).ok_or_else(|| anyhow!("tecla desconhecida: {key}"))?;
    let code = KeyCode::encode(&parse_target(target)?);
    let mut buf = dev.layer(n.into())?;
    let o = key.matrix as usize * 4;
    buf[o..o + 4].copy_from_slice(&code.0);
    dev.set_layer(n.into(), &buf)?;
    println!("{} ({}) remapeada na camada {layer}", key.name, key.matrix);
    Ok(())
}

fn macro_text(dev: &Device, key: &str, text: &str) -> Result<()> {
    let key = layout::by_name(key).ok_or_else(|| anyhow!("tecla desconhecida: {key}"))?;
    let mut macros = storage::load_macros()?;
    let index: u8 = macros
        .len()
        .try_into()
        .map_err(|_| anyhow!("macros demais (máximo 256)"))?;
    macros.push(Macro {
        name: text.chars().take(30).collect(),
        events: text_to_events(text),
    });
    storage::save_macros(dev, &macros)?;

    let code = KeyCode::encode(&KeyAction::Macro {
        mode: 1,
        count: 1,
        index,
    });
    let mut buf = dev.layer(0)?;
    let o = key.matrix as usize * 4;
    buf[o..o + 4].copy_from_slice(&code.0);
    dev.set_layer(0, &buf)?;
    println!("macro {index} gravada em {} (texto: {text})", key.name);
    Ok(())
}

fn sleep(dev: &Device, value: &str) -> Result<()> {
    let hw = if value.eq_ignore_ascii_case("never") {
        0
    } else {
        let secs: u32 = value
            .parse()
            .map_err(|_| anyhow!("sleep inválido: {value} (use segundos ou never)"))?;
        SLEEP_TABLE
            .iter()
            .find(|(s, _)| *s == secs)
            .map(|(_, hw)| *hw)
            .ok_or_else(|| anyhow!("sleep inválido: {secs} (use 60, 90, 120, 180, 240, 300, 600, 900, 1200 ou never)"))?
    };
    let mut led = dev.led()?;
    led.set_sleep_hw(hw);
    dev.set_led(&led)?;
    println!("sleep definido: {value}");
    Ok(())
}

fn debounce(dev: &Device, value: u8) -> Result<()> {
    let mut led = dev.led()?;
    led.set_debounce(value)?;
    dev.set_led(&led)?;
    println!("debounce definido: {value}");
    Ok(())
}

fn reset(dev: &Device, yes: bool) -> Result<()> {
    if !yes {
        bail!("reset apaga as configurações do teclado; repita com --yes");
    }
    dev.factory_reset()?;
    println!("padrão de fábrica restaurado");
    Ok(())
}

fn audio(seconds: u64, gain: f32) -> Result<()> {
    let dev = open_dev()?;
    let shared = Arc::new(Mutex::new(Some(dev)));
    let viz = AudioViz::start(shared.clone(), gain)?;
    thread::sleep(Duration::from_secs(seconds));
    viz.stop();
    println!("áudio finalizado após {seconds} s");
    Ok(())
}

/// Parses a remap target: `key:<hex>[+mod…]`, `media:<hex>`, `macro:<n>[:times:N|:hold|:any]`, `off`, `raw:<8 hex>`.
fn parse_target(s: &str) -> Result<KeyAction> {
    if s == "off" {
        return Ok(KeyAction::Disabled);
    }
    let (kind, rest) = s
        .split_once(':')
        .ok_or_else(|| anyhow!("alvo inválido: {s}"))?;
    match kind {
        "key" => {
            let mut parts = rest.split('+');
            let usage = u8::try_from(hex_value(parts.next().unwrap_or(""))?)
                .map_err(|_| anyhow!("usage fora de 0-FF: {rest}"))?;
            let mut mods = 0u8;
            for m in parts {
                mods |= modifier_bit(m)?;
            }
            Ok(KeyAction::Key { mods, usage })
        }
        "media" => {
            let usage = u16::try_from(hex_value(rest)?)
                .map_err(|_| anyhow!("usage de mídia fora de 0-FFFF: {rest}"))?;
            Ok(KeyAction::Consumer(usage))
        }
        "macro" => {
            let mut p = rest.split(':');
            let index: u8 = p
                .next()
                .unwrap_or("")
                .parse()
                .map_err(|_| anyhow!("índice de macro inválido: {rest}"))?;
            let (mode, count) = match (p.next(), p.next()) {
                (None, _) => (1, 1),
                (Some("times"), Some(n)) => {
                    let n: u8 = n.parse().map_err(|_| anyhow!("vezes inválido: {n}"))?;
                    if n == 0 {
                        bail!("vezes deve ser 1-255");
                    }
                    (1, n)
                }
                (Some("hold"), None) => (2, 1),
                (Some("any"), None) => (4, 1),
                _ => bail!("modo de macro inválido: {rest}"),
            };
            Ok(KeyAction::Macro { mode, count, index })
        }
        "raw" => {
            let digits: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
            if digits.len() != 8 {
                bail!("raw espera 8 dígitos hex: {rest}");
            }
            let mut b = [0u8; 4];
            for (i, byte) in b.iter_mut().enumerate() {
                *byte = u8::from_str_radix(&digits[i * 2..i * 2 + 2], 16)
                    .map_err(|_| anyhow!("raw com hex inválido: {rest}"))?;
            }
            Ok(KeyAction::Raw(b))
        }
        _ => bail!("alvo inválido: {s}"),
    }
}

fn modifier_bit(name: &str) -> Result<u8> {
    Ok(match name.to_ascii_lowercase().as_str() {
        "ctrl" => 0x01,
        "shift" => 0x02,
        "alt" => 0x04,
        "win" => 0x08,
        "rctrl" => 0x10,
        "rshift" => 0x20,
        "ralt" => 0x40,
        "rwin" => 0x80,
        other => bail!("modificador desconhecido: {other}"),
    })
}

fn hex_value(s: &str) -> Result<u32> {
    let t = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    u32::from_str_radix(t, 16).map_err(|_| anyhow!("hex inválido: {s}"))
}

fn parse_rgb(s: &str) -> Result<[u8; 3]> {
    if s.len() != 6 || !s.is_ascii() {
        bail!("cor deve ser RRGGBB: {s}");
    }
    let mut c = [0u8; 3];
    for (i, byte) in c.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|_| anyhow!("cor inválida: {s}"))?;
    }
    Ok(c)
}

fn find_effect(s: &str) -> Result<usize> {
    if let Ok(n) = s.parse::<usize>() {
        if (1..=EFFECTS.len()).contains(&n) {
            return Ok(n - 1);
        }
        bail!("linha de efeito deve ser 1-{}", EFFECTS.len());
    }
    EFFECTS
        .iter()
        .position(|e| e.name.eq_ignore_ascii_case(s))
        .ok_or_else(|| anyhow!("efeito desconhecido: {s}"))
}

fn oem_running() -> bool {
    Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq OemDrv.exe", "/NH"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("OemDrv.exe"))
        .unwrap_or(false)
}

fn sim_nao(b: bool) -> &'static str {
    if b {
        "sim"
    } else {
        "não"
    }
}

fn hex(b: &[u8]) -> String {
    b.iter()
        .map(|x| format!("{x:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_remap_targets() {
        assert!(matches!(
            parse_target("key:0x29+ctrl").unwrap(),
            KeyAction::Key {
                mods: 0x01,
                usage: 0x29
            }
        ));
        assert!(matches!(
            parse_target("media:0xCD").unwrap(),
            KeyAction::Consumer(0xCD)
        ));
        assert!(matches!(
            parse_target("macro:2:times:3").unwrap(),
            KeyAction::Macro {
                mode: 1,
                count: 3,
                index: 2
            }
        ));
        assert!(matches!(
            parse_target("macro:1:hold").unwrap(),
            KeyAction::Macro {
                mode: 2,
                count: 1,
                index: 1
            }
        ));
        assert!(matches!(parse_target("off").unwrap(), KeyAction::Disabled));
        assert!(matches!(
            parse_target("raw:07000 01D").unwrap(),
            KeyAction::Raw(b) if b == [0x07, 0x00, 0x00, 0x1D]
        ));
        assert!(parse_target("key:0x29+bogus").is_err());
        assert!(parse_target("macro:1:times:0").is_err());
    }
}
