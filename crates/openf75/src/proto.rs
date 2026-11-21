//! Wire protocol: command table, packet and frame builders, LED block, palette,
//! per-key buffer, realtime frame and key codes. Pure: no I/O.

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::layout::EFFECTS;

pub const CABLE_VIDPID: (u16, u16) = (0x258A, 0x010C);
pub const DONGLE_VIDPID: (u16, u16) = (0x3554, 0xFA09);
pub const LED_LEN: usize = 128;
pub const LAYER_LEN: usize = 512;
pub const PERKEY_LEN: usize = 384;
pub const TABLE_LEN: usize = 512;
pub const FRAME_LEN: usize = 378;
pub const MACRO_CAP: usize = 4096;
pub const KNOWN_PSD: [[u8; 6]; 2] = [[3, 0, 0, 0, 0, 0xcd], [3, 0, 0, 0, 0, 0xe1]];
/// Receiver read command: 1 byte, 0 = keyboard asleep or off.
pub const ONLINE_READ: u8 = 0x07;
/// Cable read command for the model Psd (sub 1).
pub const PSD_READ: u8 = 0x82;

/// Command table. Write codes via `cable()` / `dongle()`, read codes via `cable_read()` / `dongle_read()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    Matrix,
    Led,
    Macro,
    PerKey,
    Realtime,
    Table,
    Reset,
}

impl Cmd {
    pub fn cable(self) -> u8 {
        match self {
            Cmd::Matrix => 0x03,
            Cmd::Led => 0x04,
            Cmd::Macro => 0x05,
            Cmd::PerKey => 0x06,
            Cmd::Realtime => 0x08,
            Cmd::Table => 0x0A,
            Cmd::Reset => 0x11,
        }
    }

    pub fn dongle(self) -> u8 {
        match self {
            Cmd::Matrix => 0x01,
            Cmd::Led => 0x04,
            Cmd::Macro => 0x03,
            Cmd::PerKey => 0x02,
            Cmd::Realtime => 0x88,
            Cmd::Table => 0x09,
            Cmd::Reset => 0x06,
        }
    }

    /// Cable read code (`write | 0x80`); `None` where the OEM never reads back.
    pub fn cable_read(self) -> Option<u8> {
        match self {
            Cmd::Matrix | Cmd::Led | Cmd::PerKey | Cmd::Table => Some(self.cable() | 0x80),
            _ => None,
        }
    }

    /// Receiver read code; `None` where the OEM never reads back.
    pub fn dongle_read(self) -> Option<u8> {
        match self {
            Cmd::Matrix => Some(0x41),
            Cmd::Led => Some(0x44),
            Cmd::PerKey => Some(0x42),
            Cmd::Table => Some(0x49),
            _ => None,
        }
    }
}

/// Byte 19 of a receiver packet: sum of bytes 0..19, mod 256.
pub fn checksum(p: &[u8; 20]) -> u8 {
    p[..19].iter().fold(0u8, |a, b| a.wrapping_add(*b))
}

/// Receiver write packets: 14 data bytes each. Caller keeps `data.len() <= 14 * 255`.
pub fn dongle_packets(cmd: u8, sub: u8, data: &[u8]) -> Vec<[u8; 20]> {
    let total = data.len().div_ceil(14);
    assert!(
        total <= 255,
        "transferência do receptor acima de 255 pacotes"
    );
    data.chunks(14)
        .enumerate()
        .map(|(i, c)| {
            let mut p = [0u8; 20];
            p[0] = 0x13;
            p[1] = cmd;
            p[2] = total as u8;
            p[3] = i as u8;
            p[4] = ((sub & 0x0F) << 4) | c.len() as u8;
            p[5..5 + c.len()].copy_from_slice(c);
            p[19] = checksum(&p);
            p
        })
        .collect()
}

/// Receiver read request: `13 cmd 01 00 sub<<4 … cks`.
pub fn dongle_read_request(cmd: u8, sub: u8) -> [u8; 20] {
    let mut p = [0u8; 20];
    p[0] = 0x13;
    p[1] = cmd;
    p[2] = 0x01;
    p[4] = (sub & 0x0F) << 4;
    p[19] = checksum(&p);
    p
}

/// Cable transfer header: `[06 cmd sub 00 total idx len_lo len_hi]`. Read requests pass `cmd | 0x80`.
pub fn cable_read_header(cmd: u8, sub: u8, chunk_idx: u8, total: u8, len: u16) -> [u8; 520] {
    let mut f = [0u8; 520];
    f[0] = 0x06;
    f[1] = cmd;
    f[2] = sub;
    f[4] = total;
    f[5] = chunk_idx;
    f[6..8].copy_from_slice(&len.to_le_bytes());
    f
}

/// Cable write frames, one per 512-byte chunk.
pub fn cable_frames(cmd: u8, sub: u8, data: &[u8]) -> Vec<[u8; 520]> {
    let chunks: Vec<&[u8]> = data.chunks(512).collect();
    let total = chunks.len() as u8;
    chunks
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let mut f = cable_read_header(cmd, sub, i as u8, total, c.len() as u16);
            f[8..8 + c.len()].copy_from_slice(c);
            f
        })
        .collect()
}

/// LED/settings block, 128 bytes. Unknown bytes are preserved; only the documented offsets are edited.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedBlock(pub [u8; 128]);

impl LedBlock {
    pub fn validate(&self) -> Result<()> {
        if self.0[126..128] != [0x5A, 0xA5] {
            bail!("bloco LED sem trailer 5A A5");
        }
        Ok(())
    }

    /// 0-based API row whose hardware id matches byte `[10]`.
    pub fn effect_row(&self) -> Option<usize> {
        EFFECTS.iter().position(|e| e.hw == self.0[10])
    }

    /// `row` 0..=19 (API row); brightness 0..=9, speed 0..=4, colour nibble 0..=7 (7 = random).
    pub fn set_effect(&mut self, row: usize, brightness: u8, speed: u8, colour: u8) -> Result<()> {
        let Some(e) = EFFECTS.get(row) else {
            bail!("efeito inválido: {row}");
        };
        if brightness > 9 || speed > 4 || colour > 7 {
            bail!("valor fora da faixa: brilho 0..9, velocidade 0..4, cor 0..7");
        }
        self.0[9] = (row == 18) as u8;
        self.0[10] = e.hw;
        self.0[11] = 0x20;
        self.0[0x38] = 0xFF;
        self.0[0x39] = 0xFF;
        let o = 0x3A + 2 * row;
        self.0[o] = brightness;
        self.0[o + 1] = (speed << 4) | colour;
        Ok(())
    }

    /// (brightness, speed, colour nibble) for an API row.
    pub fn slot(&self, row: usize) -> Option<(u8, u8, u8)> {
        if row >= EFFECTS.len() {
            return None;
        }
        let o = 0x3A + 2 * row;
        Some((self.0[o], self.0[o + 1] >> 4, self.0[o + 1] & 0x0F))
    }

    pub fn set_sleep_hw(&mut self, v: u8) {
        self.0[0x18] = v;
    }

    pub fn set_debounce(&mut self, v: u8) -> Result<()> {
        if v > 4 {
            bail!("debounce fora da faixa 0..=4");
        }
        self.0[3] = v;
        Ok(())
    }

    /// Knob mode at `[0x1A]`: 0 = volume/mute (office), 1 = brightness/effect (gaming).
    /// Observed on the receiver by toggling the knob A→B→A; the OEM app never writes it.
    pub fn knob_volume(&self) -> bool {
        self.0[0x1A] == 0
    }

    pub fn set_knob_volume(&mut self, volume: bool) {
        self.0[0x1A] = u8::from(!volume);
    }
}

const PALETTE_DEFAULT: [[u8; 3]; 7] = [
    [0xFF, 0x00, 0x00],
    [0x00, 0xFF, 0x00],
    [0x00, 0x00, 0xFF],
    [0xFF, 0xFF, 0x00],
    [0xFF, 0x00, 0xFF],
    [0x00, 0xFF, 0xFF],
    [0xFF, 0xFF, 0xFF],
];

/// RGB palette table: 21 rows of 21 bytes; row `r+1` holds the 7 colours for API row `r`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteTable(pub [u8; 512]);

impl PaletteTable {
    fn offset(row: usize, slot: usize) -> Option<usize> {
        if row >= EFFECTS.len() || slot >= 7 {
            None
        } else {
            Some((row + 1) * 21 + slot * 3)
        }
    }

    pub fn get(&self, row: usize, slot: usize) -> Option<[u8; 3]> {
        let o = Self::offset(row, slot)?;
        Some([self.0[o], self.0[o + 1], self.0[o + 2]])
    }

    pub fn set(&mut self, row: usize, slot: usize, rgb: [u8; 3]) -> Result<()> {
        let o = Self::offset(row, slot).ok_or_else(|| anyhow!("posição de paleta inválida"))?;
        self.0[o..o + 3].copy_from_slice(&rgb);
        Ok(())
    }

    pub fn defaults() -> Self {
        let mut t = [0u8; 512];
        for row in 0..EFFECTS.len() {
            for (slot, c) in PALETTE_DEFAULT.iter().enumerate() {
                let o = (row + 1) * 21 + slot * 3;
                t[o..o + 3].copy_from_slice(c);
            }
        }
        t[506] = 0x5A;
        t[507] = 0xA5;
        Self(t)
    }
}

/// Per-key colours, planar: R plane `[0..126)`, G `[126..252)`, B `[252..378)`, then 6 zero bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PerKey(pub [u8; 384]);

impl PerKey {
    pub fn get(&self, matrix: u8) -> Option<[u8; 3]> {
        let m = matrix as usize;
        if m >= 126 {
            return None;
        }
        Some([self.0[m], self.0[126 + m], self.0[252 + m]])
    }

    pub fn set(&mut self, matrix: u8, rgb: [u8; 3]) -> Result<()> {
        let m = matrix as usize;
        if m >= 126 {
            bail!("tecla fora da matriz: {matrix}");
        }
        self.0[m] = rgb[0];
        self.0[126 + m] = rgb[1];
        self.0[252 + m] = rgb[2];
        Ok(())
    }
}

/// Realtime frame: interleaved RGB at `matrix * 3`. Matrix indices >= 126 are ignored.
pub fn realtime_frame(colours: &[(u8, [u8; 3])]) -> [u8; 378] {
    let mut f = [0u8; FRAME_LEN];
    for (m, rgb) in colours {
        let o = *m as usize * 3;
        if o + 3 <= 126 * 3 {
            f[o..o + 3].copy_from_slice(rgb);
        }
    }
    f
}

/// One 4-byte key slot, raw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyCode(pub [u8; 4]);

impl KeyCode {
    pub fn encode(a: &KeyAction) -> Self {
        KeyCode(a.encode())
    }

    pub fn decode(self) -> KeyAction {
        KeyAction::decode(&self.0)
    }
}

/// Decoded key slot. Serialised as `{"type": ..., "value": ...}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum KeyAction {
    Key { mods: u8, usage: u8 },
    Consumer(u16),
    Macro { mode: u8, count: u8, index: u8 },
    Fn,
    Disabled,
    Raw([u8; 4]),
}

impl KeyAction {
    pub fn decode(b: &[u8; 4]) -> Self {
        match *b {
            [0, 0, 0, 0] => Self::Disabled,
            [0, mods, 0, usage] => Self::Key { mods, usage },
            [2, 0, hi, lo] => Self::Consumer(u16::from_be_bytes([hi, lo])),
            [3, mode, count, index] => Self::Macro { mode, count, index },
            [0x0D, 0, 0, 0] => Self::Fn,
            other => Self::Raw(other),
        }
    }

    pub fn encode(&self) -> [u8; 4] {
        match *self {
            Self::Key { mods, usage } => [0, mods, 0, usage],
            Self::Consumer(u) => {
                let [hi, lo] = u.to_be_bytes();
                [2, 0, hi, lo]
            }
            Self::Macro { mode, count, index } => [3, mode, count, index],
            Self::Fn => [0x0D, 0, 0, 0],
            Self::Disabled => [0, 0, 0, 0],
            Self::Raw(b) => b,
        }
    }
}

/// Decode a 512-byte layer (128 slots).
pub fn layer_actions(layer: &[u8; 512]) -> Vec<KeyAction> {
    layer
        .chunks_exact(4)
        .map(|c| KeyAction::decode(&[c[0], c[1], c[2], c[3]]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dongle_packets_led_block_golden() {
        let pk = dongle_packets(0x04, 0, &[0u8; 128]);
        assert_eq!(pk.len(), 10);
        assert_eq!(&pk[0][..5], &[0x13, 0x04, 0x0A, 0x00, 0x0E]);
        assert_eq!(pk[0][19], 0x2F);
        assert_eq!(&pk[9][..5], &[0x13, 0x04, 0x0A, 0x09, 0x02]);
        assert_eq!(pk[9][19], 0x2C);
        for p in &pk {
            assert_eq!(p[19], checksum(p));
        }
    }

    #[test]
    fn dongle_read_request_layout() {
        let p = dongle_read_request(0x41, 0);
        assert_eq!(&p[..4], &[0x13, 0x41, 0x01, 0x00]);
        assert_eq!(p[19], 0x13 + 0x41 + 0x01);
    }

    #[test]
    fn cable_frames_header_golden() {
        let f = cable_frames(0x03, 1, &[0u8; 512]);
        assert_eq!(f.len(), 1);
        assert_eq!(
            &f[0][..8],
            &[0x06, 0x03, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02]
        );
    }

    #[test]
    fn cable_frames_splits_at_512() {
        let f = cable_frames(0x05, 0, &[7u8; 1000]);
        assert_eq!(f.len(), 2);
        assert_eq!(
            &f[1][..8],
            &[0x06, 0x05, 0x00, 0x00, 0x02, 0x01, 0xE8, 0x01]
        );
        assert_eq!(f[1][8], 7);
    }

    #[test]
    fn key_codes_golden() {
        assert_eq!(KeyAction::Consumer(0xCD).encode(), [0x02, 0x00, 0x00, 0xCD]);
        assert_eq!(
            KeyAction::Key {
                mods: 0x02,
                usage: 0x04
            }
            .encode(),
            [0x00, 0x02, 0x00, 0x04]
        );
        assert_eq!(
            KeyAction::Macro {
                mode: 1,
                count: 3,
                index: 2
            }
            .encode(),
            [0x03, 0x01, 0x03, 0x02]
        );
        assert_eq!(KeyAction::decode(&[0x0D, 0, 0, 0]), KeyAction::Fn);
        assert_eq!(
            KeyAction::decode(&[0x07, 0, 0, 0x1D]),
            KeyAction::Raw([0x07, 0, 0, 0x1D])
        );
        assert_eq!(KeyAction::decode(&[0, 0, 0, 0]), KeyAction::Disabled);
    }

    #[test]
    fn led_block_set_effect_golden() {
        let mut b = LedBlock([0u8; 128]);
        b.set_effect(1, 9, 2, 3).unwrap();
        assert_eq!(b.0[10], 2);
        assert_eq!(b.0[0x3C], 9);
        assert_eq!(b.0[0x3D], 0x23);
        assert_eq!(b.0[9], 0);
        assert_eq!(b.0[11], 0x20);
        assert_eq!(b.effect_row(), Some(1));
        assert_eq!(b.slot(1), Some((9, 2, 3)));
    }

    #[test]
    fn led_block_self_define_flag() {
        let mut b = LedBlock([0u8; 128]);
        b.set_effect(18, 5, 0, 0).unwrap();
        assert_eq!(b.0[9], 1);
        assert_eq!(b.0[10], 21);
        assert_eq!(b.effect_row(), Some(18));
    }

    #[test]
    fn perkey_planar_set() {
        let mut p = PerKey([0u8; 384]);
        p.set(81, [1, 2, 3]).unwrap();
        assert_eq!(p.0[81], 1);
        assert_eq!(p.0[207], 2);
        assert_eq!(p.0[333], 3);
        assert_eq!(p.get(81), Some([1, 2, 3]));
    }

    #[test]
    fn palette_defaults_layout() {
        let t = PaletteTable::defaults();
        assert_eq!(t.get(0, 0), Some([0xFF, 0x00, 0x00]));
        assert_eq!(t.get(0, 6), Some([0xFF, 0xFF, 0xFF]));
        assert_eq!(&t.0[..21], &[0u8; 21]);
        assert_eq!(&t.0[506..508], &[0x5A, 0xA5]);
    }

    #[test]
    fn realtime_frame_interleaved() {
        let f = realtime_frame(&[(2, [1, 2, 3])]);
        assert_eq!(&f[6..9], &[1, 2, 3]);
    }
}
