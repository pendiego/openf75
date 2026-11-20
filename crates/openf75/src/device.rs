//! HID transport and device operations (plan Step 3).
//!
//! Only the primitives `read`/`send` and the `cable_*`/`dongle_*` helpers touch HID.
//! Public write methods call `writable()` once; composites call it once and use the `put_*` helpers.

use std::os::windows::process::CommandExt;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{bail, ensure, Result};
use hidapi::{DeviceInfo, HidApi, HidDevice};
use serde::{Deserialize, Serialize};

use crate::layout::EFFECTS;
use crate::macros::Macro;
use crate::proto::{
    self, Cmd, LedBlock, PaletteTable, PerKey, CABLE_VIDPID, DONGLE_VIDPID, KNOWN_PSD,
};
use crate::storage;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Effect row of self-define (per-key) mode, 0-based.
pub const SELF_DEFINE_ROW: usize = 18;

fn pause(ms: u64) {
    sleep(Duration::from_millis(ms));
}

pub enum Transport {
    Cable(HidDevice),
    Dongle(HidDevice),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Cable,
    Dongle,
}

pub struct Device {
    t: Transport,
    pub kind: Kind,
    pub psd: Option<[u8; 6]>,
    pub fw: u16,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Colour {
    /// Palette slot 0..=6.
    Slot(u8),
    Custom([u8; 3]),
    Random,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub led: Vec<u8>,
    pub table: Vec<u8>,
    pub perkey: Vec<u8>,
    pub layers: [Vec<u8>; 2],
    pub macros: Vec<Macro>,
}

fn find<'a>(api: &'a HidApi, vidpid: (u16, u16), page: u16, usage: u16) -> Option<&'a DeviceInfo> {
    api.device_list().find(|i| {
        (i.vendor_id(), i.product_id()) == vidpid && i.usage_page() == page && i.usage() == usage
    })
}

pub fn open() -> Result<Device> {
    let api = HidApi::new()?;
    let (kind, t, fw) = if let Some(i) = find(&api, CABLE_VIDPID, 0xFF00, 1) {
        (
            Kind::Cable,
            Transport::Cable(i.open_device(&api)?),
            i.release_number(),
        )
    } else if let Some(i) = find(&api, DONGLE_VIDPID, 0xFF02, 2) {
        (
            Kind::Dongle,
            Transport::Dongle(i.open_device(&api)?),
            i.release_number(),
        )
    } else {
        bail!("Teclado AULA F75 não encontrado (cabo ou receptor 2.4G)");
    };
    let mut dev = Device {
        t,
        kind,
        psd: None,
        fw,
    };
    if let Transport::Cable(h) = &dev.t {
        dev.psd = cable_read(h, 0x82, 1, 6)
            .ok()
            .and_then(|v| v.try_into().ok());
    }
    Ok(dev)
}

fn hex(b: &[u8]) -> String {
    b.iter()
        .map(|x| format!("{x:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn oem_running() -> bool {
    Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq OemDrv.exe", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("OemDrv.exe"))
        .unwrap_or(false)
}

fn fixed<const N: usize>(v: &[u8], what: &str) -> Result<[u8; N]> {
    v.try_into()
        .map_err(|_| anyhow::anyhow!("snapshot inválido: {what}"))
}

impl Device {
    pub fn writable(&self) -> Result<()> {
        if self.kind == Kind::Cable {
            match self.psd {
                Some(p) if KNOWN_PSD.contains(&p) => {}
                other => bail!(
                    "modelo não suportado: psd={}",
                    other.map_or("desconhecido".to_string(), |p| hex(&p))
                ),
            }
        }
        if oem_running() {
            bail!("Feche o software da AULA (OemDrv.exe) antes de gravar");
        }
        if self.kind == Kind::Dongle && !self.online()? {
            bail!("Teclado dormindo ou desligado: pressione uma tecla");
        }
        Ok(())
    }

    pub fn online(&self) -> Result<bool> {
        match &self.t {
            Transport::Cable(_) => Ok(true),
            Transport::Dongle(h) => {
                Ok(dongle_read(h, 0x07, 0, 1)?.first().is_some_and(|&b| b != 0))
            }
        }
    }

    pub fn led(&self) -> Result<LedBlock> {
        for attempt in 0..5 {
            if attempt > 0 {
                pause(100);
            }
            if let Ok(v) = self.read(Cmd::Led, 0, 128) {
                if let Ok(arr) = <[u8; 128]>::try_from(v.as_slice()) {
                    let b = LedBlock(arr);
                    if b.validate().is_ok() {
                        return Ok(b);
                    }
                }
            }
        }
        bail!("bloco LED inválido (sem 5A A5)")
    }

    pub fn set_led(&self, b: &LedBlock) -> Result<()> {
        self.writable()?;
        self.put_led(b)
    }

    /// The device palette, or `None` when it cannot be read (the receiver never answers 0x49).
    pub fn table_read(&self) -> Option<PaletteTable> {
        if matches!(self.t, Transport::Dongle(_)) {
            return None; // the receiver never answers 0x49 (observed); don't spend 3 s per attempt
        }
        let v = self.read(Cmd::Table, 0, 512).ok()?;
        let a = <[u8; 512]>::try_from(v.as_slice()).ok()?;
        (a[506] == 0x5A && a[507] == 0xA5).then_some(PaletteTable(a))
    }

    /// Display only: falls back to defaults. Never feed this to a write or a backup.
    pub fn table(&self) -> Result<PaletteTable> {
        Ok(self.table_read().unwrap_or_else(PaletteTable::defaults))
    }

    pub fn set_table(&self, t: &PaletteTable) -> Result<()> {
        self.writable()?;
        self.put_table(t)
    }

    pub fn layer(&self, n: u8) -> Result<[u8; 512]> {
        ensure!(n <= 1, "camada inválida: {n}");
        let a: [u8; 512] = self
            .read(Cmd::Matrix, n, 512)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("camada {n}: tamanho inválido"))?;
        if a.iter().all(|&b| b == 0) || a.iter().all(|&b| b == 0xFF) {
            bail!("camada {n} vazia ou inválida");
        }
        Ok(a)
    }

    /// Writes a layer; slot 127 (wheel knob) keeps the value currently on the device.
    pub fn set_layer(&self, n: u8, data: &[u8; 512]) -> Result<()> {
        self.writable()?;
        let cur = self.layer(n)?;
        let mut d = *data;
        d[508..512].copy_from_slice(&cur[508..512]);
        self.put_layer(n, &d)
    }

    pub fn perkey(&self) -> Result<PerKey> {
        let v = self.read(Cmd::PerKey, 0, 384)?;
        Ok(PerKey(fixed(&v, "por tecla")?))
    }

    pub fn set_perkey(&self, p: &PerKey) -> Result<()> {
        self.writable()?;
        self.put_perkey(p)
    }

    /// Writes the raw macro buffer. Receiver: 512-byte pages with sub = page.
    pub fn set_macros(&self, buf: &[u8]) -> Result<()> {
        self.writable()?;
        self.put_macros(buf)
    }

    /// Shortcut: skips `writable()` because the audio stream sends ~30 frames/s and
    /// `writable()` spawns tasklist; `audio::start` checks once before streaming.
    pub fn realtime(&self, frame: &[u8; 378]) -> Result<()> {
        self.send(Cmd::Realtime, 0, frame)
    }

    pub fn factory_reset(&self) -> Result<()> {
        self.writable()?;
        self.send(Cmd::Reset, 0, &[0])
    }

    pub fn apply_effect(
        &self,
        row: usize,
        brightness: u8,
        speed: u8,
        colour: Colour,
    ) -> Result<()> {
        self.writable()?;
        ensure!(row < EFFECTS.len(), "efeito inválido: {row}");
        ensure!(brightness <= 9, "brilho deve ser 0..9");
        ensure!(speed <= 4, "velocidade deve ser 0..4");
        let mut led = self.led()?;
        let mut table = None;
        let nibble = match colour {
            Colour::Slot(s) => {
                ensure!(s <= 6, "cor deve ser 0..6");
                s
            }
            Colour::Random => 7,
            Colour::Custom(rgb) => {
                let mut t = self.table_read().ok_or_else(|| {
                    anyhow::anyhow!("cor personalizada exige ler a paleta, e este teclado não a devolve (use o cabo ou um slot 0-6)")
                })?;
                let cur = led.slot(row).map_or(7, |s| s.2);
                let slot = if cur == 7 { 0 } else { cur };
                t.set(row, slot as usize, rgb)?;
                table = Some(t);
                slot
            }
        };
        led.set_effect(row, brightness, speed, nibble)?;
        self.put_led(&led)?;
        if let (Some(t), true) = (&table, row != SELF_DEFINE_ROW) {
            self.put_table(t)?;
        }
        Ok(())
    }

    pub fn apply_perkey(&self, p: &PerKey) -> Result<()> {
        self.writable()?;
        let mut led = self.led()?;
        let (b, s, n) = led.slot(SELF_DEFINE_ROW).unwrap_or((0, 0, 0));
        led.set_effect(SELF_DEFINE_ROW, b, s, n)?;
        self.put_led(&led)?;
        self.put_perkey(p)
    }

    pub fn snapshot(&self) -> Result<Snapshot> {
        Ok(Snapshot {
            led: self.led()?.0.to_vec(),
            table: self.table_read().map(|t| t.0.to_vec()).unwrap_or_default(),
            perkey: self.perkey()?.0.to_vec(),
            layers: [self.layer(0)?.to_vec(), self.layer(1)?.to_vec()],
            macros: storage::load_macros()?,
        })
    }

    pub fn restore(&self, s: &Snapshot) -> Result<()> {
        self.writable()?;
        let l0: [u8; 512] = fixed(&s.layers[0], "camada 0")?;
        let l1: [u8; 512] = fixed(&s.layers[1], "camada 1")?;
        self.put_layer(0, &l0)?;
        self.put_layer(1, &l1)?;
        if !s.macros.is_empty() {
            self.put_macros(&crate::macros::encode_buffer(&s.macros)?)?;
            storage::write_macros_file(&s.macros)?;
        }
        let led = LedBlock(fixed(&s.led, "LED")?);
        // Observed on the receiver: per-key data is only kept when written while self-define is selected.
        let mut tmp = led.clone();
        let (b, sp, n) = tmp.slot(SELF_DEFINE_ROW).unwrap_or((0, 0, 0));
        tmp.set_effect(SELF_DEFINE_ROW, b, sp, n)?;
        self.put_led(&tmp)?;
        self.put_perkey(&PerKey(fixed(&s.perkey, "por tecla")?))?;
        self.put_led(&led)?;
        if !s.table.is_empty() && matches!(self.t, Transport::Cable(_)) {
            self.put_table(&PaletteTable(fixed(&s.table, "tabela")?))?;
        }
        Ok(())
    }

    // ---- unchecked writers: callers must have called writable() ----

    fn put_led(&self, b: &LedBlock) -> Result<()> {
        b.validate()?;
        pause(60);
        self.send(Cmd::Led, 0, &b.0)
    }

    fn put_table(&self, t: &PaletteTable) -> Result<()> {
        pause(130);
        self.send(Cmd::Table, 0, &t.0)
    }

    fn put_layer(&self, n: u8, d: &[u8; 512]) -> Result<()> {
        self.send(Cmd::Matrix, n, d)?;
        pause(50);
        Ok(())
    }

    fn put_perkey(&self, p: &PerKey) -> Result<()> {
        pause(100);
        self.send(Cmd::PerKey, 0, &p.0)
    }

    fn put_macros(&self, buf: &[u8]) -> Result<()> {
        match &self.t {
            Transport::Cable(_) => self.send(Cmd::Macro, 0, buf),
            Transport::Dongle(_) => {
                for (page, chunk) in buf.chunks(512).enumerate() {
                    if page > 0 {
                        pause(100);
                    }
                    self.send(Cmd::Macro, page as u8, chunk)?;
                }
                Ok(())
            }
        }
    }

    // ---- primitives ----

    fn read(&self, cmd: Cmd, sub: u8, len: usize) -> Result<Vec<u8>> {
        match &self.t {
            Transport::Cable(h) => cable_read(h, cmd.cable() | 0x80, sub, len),
            Transport::Dongle(h) => dongle_read(h, cmd.dongle() | 0x40, sub, len),
        }
    }

    fn send(&self, cmd: Cmd, sub: u8, data: &[u8]) -> Result<()> {
        let rt = matches!(cmd, Cmd::Realtime);
        match &self.t {
            Transport::Cable(h) => cable_write(h, cmd.cable(), sub, data, if rt { 7 } else { 20 }),
            Transport::Dongle(h) => {
                dongle_write(h, cmd.dongle(), sub, data, if rt { 10 } else { 15 })
            }
        }
    }
}

// ---- cable: 520-byte feature reports, report id 0x06 ----

fn cable_write(h: &HidDevice, cmd: u8, sub: u8, data: &[u8], delay: u64) -> Result<()> {
    for frame in proto::cable_frames(cmd, sub, data) {
        pause(delay);
        let mut attempt = 1;
        loop {
            match h.send_feature_report(&frame) {
                Ok(()) => break,
                Err(e) if attempt >= 3 => return Err(e.into()),
                Err(_) => {
                    attempt += 1;
                    pause(70);
                }
            }
        }
    }
    Ok(())
}

fn cable_read(h: &HidDevice, rcmd: u8, sub: u8, len: usize) -> Result<Vec<u8>> {
    let total = len.div_ceil(512);
    let mut out = Vec::with_capacity(len);
    for idx in 0..total {
        let clen = (len - idx * 512).min(512);
        h.send_feature_report(&proto::cable_read_header(
            rcmd,
            sub,
            idx as u8,
            total as u8,
            clen as u16,
        ))?;
        pause(20);
        let mut buf = [0u8; 520];
        buf[0] = 0x06;
        h.get_feature_report(&mut buf)?;
        out.extend_from_slice(&buf[8..8 + clen]);
    }
    Ok(out)
}

// ---- receiver: 20-byte output/input reports, report id 0x13 ----

enum Ack {
    Ok,
    Crc,
}

fn write_retry(h: &HidDevice, pkt: &[u8]) -> Result<()> {
    let mut attempt = 1;
    loop {
        match h.write(pkt) {
            Ok(_) => return Ok(()),
            Err(e) if attempt >= 4 => return Err(e.into()),
            Err(_) => {
                attempt += 1;
                pause(10);
            }
        }
    }
}

fn wait_ack(h: &HidDevice, cmd: u8, idx: u8) -> Result<Ack> {
    let mut buf = [0u8; 20];
    for _ in 0..20 {
        if h.read_timeout(&mut buf, 1000)? == 0 {
            continue;
        }
        if buf[0] == 0xFF {
            bail!("receptor recusou o comando {cmd:#04x}");
        }
        if buf[0] != 0x13 || buf[1] != cmd {
            continue;
        }
        if buf[2] & 0x80 != 0 {
            return Ok(Ack::Crc);
        }
        if buf[3] == idx {
            return Ok(Ack::Ok);
        }
    }
    bail!("receptor sem resposta ao comando {cmd:#04x}")
}

fn dongle_write(h: &HidDevice, cmd: u8, sub: u8, data: &[u8], delay: u64) -> Result<()> {
    let ack = (cmd & 0x80) == 0 && cmd != 0x06;
    for pkt in proto::dongle_packets(cmd, sub, data) {
        pause(delay);
        let mut crc_retries = 0;
        loop {
            write_retry(h, &pkt)?;
            if !ack {
                break;
            }
            match wait_ack(h, cmd, pkt[3])? {
                Ack::Ok => break,
                Ack::Crc => {
                    crc_retries += 1;
                    ensure!(
                        crc_retries <= 5,
                        "receptor: erro de CRC persistente no comando {cmd:#04x}"
                    );
                }
            }
        }
    }
    Ok(())
}

fn dongle_read(h: &HidDevice, cmd: u8, sub: u8, len: usize) -> Result<Vec<u8>> {
    let req = proto::dongle_read_request(cmd, sub);
    for _ in 0..3 {
        // Drop stale replies from earlier requests, or they get matched to this one.
        let mut stale = [0u8; 20];
        while h.read_timeout(&mut stale, 20)? > 0 {}
        write_retry(h, &req)?;
        let mut chunks: Vec<Option<Vec<u8>>> = Vec::new();
        let mut buf = [0u8; 20];
        loop {
            if h.read_timeout(&mut buf, 1000)? == 0 {
                break;
            }
            if buf[0] != 0x13 || (buf[1] & 0x7F) != (cmd & 0x7F) || buf[4] >> 4 != sub {
                continue;
            }
            let total = (buf[2] & 0x7F) as usize;
            let idx = buf[3] as usize;
            let l = (buf[4] & 0x0F) as usize;
            if chunks.is_empty() {
                chunks = vec![None; total];
            }
            if total == 0 || total != chunks.len() || idx >= total {
                continue;
            }
            chunks[idx].get_or_insert_with(|| buf[5..5 + l].to_vec());
            if idx + 1 == total {
                break;
            }
        }
        if chunks.is_empty() || chunks.iter().any(Option::is_none) {
            continue;
        }
        let data: Vec<u8> = chunks
            .into_iter()
            .flat_map(Option::unwrap_or_default)
            .collect();
        // Observed: a 512-byte key layer comes back as 36 packets (504 bytes); the tail is unused slots.
        if data.len() + 14 >= len {
            let mut out = data;
            out.resize(len, 0);
            return Ok(out);
        }
    }
    bail!("receptor sem resposta à leitura (cmd {cmd:#04x})")
}
