use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

const MACRO_CAP: usize = 4096;
const NAME_MAX: usize = 30;
const TEXT_DELAY_MS: u32 = 10;
const DELAY_MAX: u32 = 0xFFFFF;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Macro {
    pub name: String,
    pub events: Vec<MacroEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MacroEvent {
    pub usage: u8,
    pub down: bool,
    pub delay_ms: u32,
}

fn encode_event(e: &MacroEvent) -> [u8; 4] {
    let kind: u8 = if (0xE0..=0xE7).contains(&e.usage) {
        1
    } else {
        0
    };
    let delay = e.delay_ms.min(DELAY_MAX);
    let up = if e.down { 0 } else { 0x80 };
    [
        (kind << 4) | ((delay >> 16) as u8 & 0x0F) | up,
        (delay >> 8) as u8,
        delay as u8,
        e.usage,
    ]
}

/// Header of N `(offset u16 LE, size u16 LE)` pairs, then `[name_byte_len][UTF-16LE name][events]` blobs.
pub fn encode_buffer(macros: &[Macro]) -> Result<Vec<u8>> {
    let header_len = macros.len() * 4;
    let mut blobs = Vec::new();
    let mut headers = Vec::with_capacity(header_len);
    for m in macros {
        if m.name.chars().count() > NAME_MAX {
            bail!("nome da macro excede {NAME_MAX} caracteres");
        }
        let name: Vec<u8> = m
            .name
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        let mut blob = Vec::with_capacity(1 + name.len() + 4 * m.events.len());
        blob.push(name.len() as u8);
        blob.extend_from_slice(&name);
        for e in &m.events {
            blob.extend_from_slice(&encode_event(e));
        }
        let offset = header_len + blobs.len();
        if header_len + blobs.len() + blob.len() > MACRO_CAP {
            bail!("macros excedem 4096 bytes");
        }
        headers.extend_from_slice(&(offset as u16).to_le_bytes());
        headers.extend_from_slice(&(blob.len() as u16).to_le_bytes());
        blobs.extend_from_slice(&blob);
    }
    let mut out = headers;
    out.extend_from_slice(&blobs);
    Ok(out)
}

/// US-layout key for a printable ASCII char: (HID usage, shift needed).
fn key_for(c: char) -> Option<(u8, bool)> {
    Some(match c {
        'a'..='z' => (0x04 + (c as u8 - b'a'), false),
        'A'..='Z' => (0x04 + (c as u8 - b'A'), true),
        '1'..='9' => (0x1E + (c as u8 - b'1'), false),
        '0' => (0x27, false),
        '\n' => (0x28, false),
        ' ' => (0x2C, false),
        '!' => (0x1E, true),
        '@' => (0x1F, true),
        '#' => (0x20, true),
        '$' => (0x21, true),
        '%' => (0x22, true),
        '^' => (0x23, true),
        '&' => (0x24, true),
        '*' => (0x25, true),
        '(' => (0x26, true),
        ')' => (0x27, true),
        '-' => (0x2D, false),
        '_' => (0x2D, true),
        '=' => (0x2E, false),
        '+' => (0x2E, true),
        '[' => (0x2F, false),
        '{' => (0x2F, true),
        ']' => (0x30, false),
        '}' => (0x30, true),
        '\\' => (0x31, false),
        '|' => (0x31, true),
        ';' => (0x33, false),
        ':' => (0x33, true),
        '\'' => (0x34, false),
        '"' => (0x34, true),
        '`' => (0x35, false),
        '~' => (0x35, true),
        ',' => (0x36, false),
        '<' => (0x36, true),
        '.' => (0x37, false),
        '>' => (0x37, true),
        '/' => (0x38, false),
        '?' => (0x38, true),
        _ => return None,
    })
}

/// Unsupported characters are skipped silently.
pub fn text_to_events(text: &str) -> Vec<MacroEvent> {
    const LSHIFT: u8 = 0xE1;
    let mut out = Vec::new();
    let mut push = |usage, down| {
        out.push(MacroEvent {
            usage,
            down,
            delay_ms: TEXT_DELAY_MS,
        });
    };
    for c in text.chars() {
        let Some((usage, shift)) = key_for(c) else {
            continue;
        };
        if shift {
            push(LSHIFT, true);
        }
        push(usage, true);
        push(usage, false);
        if shift {
            push(LSHIFT, false);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_two_letter_macro() {
        let m = Macro {
            name: "ab".into(),
            events: vec![
                MacroEvent {
                    usage: 0x04,
                    down: true,
                    delay_ms: 300,
                },
                MacroEvent {
                    usage: 0x04,
                    down: false,
                    delay_ms: 0,
                },
            ],
        };
        let buf = encode_buffer(&[m]).unwrap();
        assert_eq!(
            buf,
            [
                0x04, 0x00, 0x0D, 0x00, 0x04, 0x61, 0x00, 0x62, 0x00, 0x00, 0x01, 0x2C, 0x04, 0x80,
                0x00, 0x00, 0x04
            ]
        );
    }

    #[test]
    fn modifier_event_type_bit() {
        assert_eq!(
            encode_event(&MacroEvent {
                usage: 0xE1,
                down: true,
                delay_ms: 0
            })[0],
            0x10
        );
    }

    #[test]
    fn text_shifted_char_wrapped_in_lshift() {
        let ev = text_to_events("A");
        assert_eq!(ev.len(), 4);
        assert_eq!((ev[0].usage, ev[0].down), (0xE1, true));
        assert_eq!((ev[1].usage, ev[1].down), (0x04, true));
        assert_eq!((ev[2].usage, ev[2].down), (0x04, false));
        assert_eq!((ev[3].usage, ev[3].down), (0xE1, false));
    }
}
