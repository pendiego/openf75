//! Physical key layout (KB.ini tri-mode ANSI) and lighting effect table.
//! Coordinates and hardware ids are protocol facts; no OEM assets are shipped.

use serde::Serialize;

/// One physical key. `rect` is x1,y1,x2,y2 on a 660x290 canvas.
/// `default_usage` is the HID usage the key sends by default (None = no HID usage).
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Key {
    pub name: &'static str,
    pub matrix: u8,
    pub rect: [u16; 4],
    pub default_usage: Option<u8>,
}

/// Lighting effect. `row` is the 0-based API row (0..=19); the UI and CLI show `row + 1`.
/// `hw` is the value written to LED block byte `[10]`. Flags: speed, brightness, colour.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effect {
    pub row: u8,
    pub hw: u8,
    pub name: &'static str,
    pub speed: bool,
    pub brightness: bool,
    pub colour: bool,
}

/// (UI seconds, hardware value). "Never" is hardware value 0 and is not in this table.
pub const SLEEP_TABLE: [(u32, u8); 9] = [
    (60, 2),
    (90, 3),
    (120, 4),
    (180, 6),
    (240, 8),
    (300, 10),
    (600, 20),
    (900, 30),
    (1200, 40),
];

pub static KEYS: [Key; 81] = [
    Key {
        name: "Esc",
        matrix: 0,
        rect: [30, 30, 55, 57],
        default_usage: Some(0x29),
    },
    Key {
        name: "F1",
        matrix: 12,
        rect: [107, 30, 132, 57],
        default_usage: Some(0x3A),
    },
    Key {
        name: "F2",
        matrix: 18,
        rect: [144, 30, 169, 57],
        default_usage: Some(0x3B),
    },
    Key {
        name: "F3",
        matrix: 24,
        rect: [183, 30, 208, 57],
        default_usage: Some(0x3C),
    },
    Key {
        name: "F4",
        matrix: 30,
        rect: [222, 30, 247, 57],
        default_usage: Some(0x3D),
    },
    Key {
        name: "F5",
        matrix: 36,
        rect: [269, 30, 294, 57],
        default_usage: Some(0x3E),
    },
    Key {
        name: "F6",
        matrix: 42,
        rect: [305, 30, 330, 57],
        default_usage: Some(0x3F),
    },
    Key {
        name: "F7",
        matrix: 48,
        rect: [343, 30, 368, 57],
        default_usage: Some(0x40),
    },
    Key {
        name: "F8",
        matrix: 54,
        rect: [388, 30, 413, 57],
        default_usage: Some(0x41),
    },
    Key {
        name: "F9",
        matrix: 60,
        rect: [439, 30, 464, 57],
        default_usage: Some(0x42),
    },
    Key {
        name: "F10",
        matrix: 66,
        rect: [477, 30, 502, 57],
        default_usage: Some(0x43),
    },
    Key {
        name: "F11",
        matrix: 72,
        rect: [516, 30, 541, 57],
        default_usage: Some(0x44),
    },
    Key {
        name: "F12",
        matrix: 78,
        rect: [555, 30, 580, 57],
        default_usage: Some(0x45),
    },
    Key {
        name: "End",
        matrix: 88,
        rect: [605, 195, 630, 222],
        default_usage: Some(0x4D),
    },
    Key {
        name: "`",
        matrix: 1,
        rect: [31, 81, 56, 108],
        default_usage: Some(0x35),
    },
    Key {
        name: "1",
        matrix: 7,
        rect: [70, 81, 95, 108],
        default_usage: Some(0x1E),
    },
    Key {
        name: "2",
        matrix: 13,
        rect: [108, 81, 133, 108],
        default_usage: Some(0x1F),
    },
    Key {
        name: "3",
        matrix: 19,
        rect: [147, 81, 172, 108],
        default_usage: Some(0x20),
    },
    Key {
        name: "4",
        matrix: 25,
        rect: [184, 81, 209, 108],
        default_usage: Some(0x21),
    },
    Key {
        name: "5",
        matrix: 31,
        rect: [223, 81, 248, 108],
        default_usage: Some(0x22),
    },
    Key {
        name: "6",
        matrix: 37,
        rect: [262, 81, 287, 108],
        default_usage: Some(0x23),
    },
    Key {
        name: "7",
        matrix: 43,
        rect: [300, 80, 325, 107],
        default_usage: Some(0x24),
    },
    Key {
        name: "8",
        matrix: 49,
        rect: [338, 81, 363, 108],
        default_usage: Some(0x25),
    },
    Key {
        name: "9",
        matrix: 55,
        rect: [377, 81, 402, 108],
        default_usage: Some(0x26),
    },
    Key {
        name: "0",
        matrix: 61,
        rect: [416, 81, 441, 108],
        default_usage: Some(0x27),
    },
    Key {
        name: "-",
        matrix: 67,
        rect: [453, 81, 478, 108],
        default_usage: Some(0x2D),
    },
    Key {
        name: "=",
        matrix: 73,
        rect: [491, 81, 516, 108],
        default_usage: Some(0x2E),
    },
    Key {
        name: "Backspace",
        matrix: 79,
        rect: [531, 81, 591, 108],
        default_usage: Some(0x2A),
    },
    Key {
        name: "Delete",
        matrix: 85,
        rect: [604, 81, 629, 108],
        default_usage: Some(0x4C),
    },
    Key {
        name: "Tab",
        matrix: 2,
        rect: [34, 120, 78, 147],
        default_usage: Some(0x2B),
    },
    Key {
        name: "Q",
        matrix: 8,
        rect: [89, 119, 114, 146],
        default_usage: Some(0x14),
    },
    Key {
        name: "W",
        matrix: 14,
        rect: [127, 119, 152, 146],
        default_usage: Some(0x1A),
    },
    Key {
        name: "E",
        matrix: 20,
        rect: [165, 119, 190, 146],
        default_usage: Some(0x08),
    },
    Key {
        name: "R",
        matrix: 26,
        rect: [204, 119, 229, 146],
        default_usage: Some(0x15),
    },
    Key {
        name: "T",
        matrix: 32,
        rect: [242, 119, 267, 146],
        default_usage: Some(0x17),
    },
    Key {
        name: "Y",
        matrix: 38,
        rect: [280, 119, 305, 146],
        default_usage: Some(0x1C),
    },
    Key {
        name: "U",
        matrix: 44,
        rect: [318, 119, 343, 146],
        default_usage: Some(0x18),
    },
    Key {
        name: "I",
        matrix: 50,
        rect: [356, 119, 381, 146],
        default_usage: Some(0x0C),
    },
    Key {
        name: "O",
        matrix: 56,
        rect: [395, 119, 420, 146],
        default_usage: Some(0x12),
    },
    Key {
        name: "P",
        matrix: 62,
        rect: [433, 119, 458, 146],
        default_usage: Some(0x13),
    },
    Key {
        name: "[",
        matrix: 68,
        rect: [471, 120, 496, 147],
        default_usage: Some(0x2F),
    },
    Key {
        name: "]",
        matrix: 74,
        rect: [509, 119, 534, 146],
        default_usage: Some(0x30),
    },
    Key {
        name: "\\",
        matrix: 80,
        rect: [549, 119, 593, 146],
        default_usage: Some(0x31),
    },
    Key {
        name: "PgUp",
        matrix: 86,
        rect: [605, 120, 630, 147],
        default_usage: Some(0x4B),
    },
    Key {
        name: "CapsLock",
        matrix: 3,
        rect: [33, 158, 85, 185],
        default_usage: Some(0x39),
    },
    Key {
        name: "A",
        matrix: 9,
        rect: [98, 158, 123, 185],
        default_usage: Some(0x04),
    },
    Key {
        name: "S",
        matrix: 15,
        rect: [136, 158, 161, 185],
        default_usage: Some(0x16),
    },
    Key {
        name: "D",
        matrix: 21,
        rect: [174, 158, 199, 185],
        default_usage: Some(0x07),
    },
    Key {
        name: "F",
        matrix: 27,
        rect: [213, 158, 238, 185],
        default_usage: Some(0x09),
    },
    Key {
        name: "G",
        matrix: 33,
        rect: [252, 158, 277, 185],
        default_usage: Some(0x0A),
    },
    Key {
        name: "H",
        matrix: 39,
        rect: [290, 158, 315, 185],
        default_usage: Some(0x0B),
    },
    Key {
        name: "J",
        matrix: 45,
        rect: [327, 158, 352, 185],
        default_usage: Some(0x0D),
    },
    Key {
        name: "K",
        matrix: 51,
        rect: [366, 158, 391, 185],
        default_usage: Some(0x0E),
    },
    Key {
        name: "L",
        matrix: 57,
        rect: [405, 158, 430, 185],
        default_usage: Some(0x0F),
    },
    Key {
        name: ";",
        matrix: 63,
        rect: [443, 158, 468, 185],
        default_usage: Some(0x33),
    },
    Key {
        name: "'",
        matrix: 69,
        rect: [481, 158, 506, 185],
        default_usage: Some(0x34),
    },
    Key {
        name: "Enter",
        matrix: 81,
        rect: [521, 158, 591, 185],
        default_usage: Some(0x28),
    },
    Key {
        name: "LShift",
        matrix: 4,
        rect: [33, 195, 105, 222],
        default_usage: Some(0xE1),
    },
    Key {
        name: "Z",
        matrix: 10,
        rect: [116, 195, 141, 222],
        default_usage: Some(0x1D),
    },
    Key {
        name: "X",
        matrix: 16,
        rect: [155, 195, 180, 222],
        default_usage: Some(0x1B),
    },
    Key {
        name: "C",
        matrix: 22,
        rect: [193, 195, 218, 222],
        default_usage: Some(0x06),
    },
    Key {
        name: "V",
        matrix: 28,
        rect: [232, 195, 257, 222],
        default_usage: Some(0x19),
    },
    Key {
        name: "B",
        matrix: 34,
        rect: [270, 196, 295, 223],
        default_usage: Some(0x05),
    },
    Key {
        name: "N",
        matrix: 40,
        rect: [309, 195, 334, 222],
        default_usage: Some(0x11),
    },
    Key {
        name: "M",
        matrix: 46,
        rect: [347, 195, 372, 222],
        default_usage: Some(0x10),
    },
    Key {
        name: ",",
        matrix: 52,
        rect: [384, 195, 409, 222],
        default_usage: Some(0x36),
    },
    Key {
        name: ".",
        matrix: 58,
        rect: [424, 195, 449, 222],
        default_usage: Some(0x37),
    },
    Key {
        name: "/",
        matrix: 64,
        rect: [463, 195, 488, 222],
        default_usage: Some(0x38),
    },
    Key {
        name: "RShift",
        matrix: 70,
        rect: [502, 195, 558, 222],
        default_usage: Some(0xE5),
    },
    Key {
        name: "Up",
        matrix: 82,
        rect: [567, 195, 592, 222],
        default_usage: Some(0x52),
    },
    Key {
        name: "LCtrl",
        matrix: 5,
        rect: [34, 234, 69, 261],
        default_usage: Some(0xE0),
    },
    Key {
        name: "LWin",
        matrix: 11,
        rect: [79, 234, 114, 261],
        default_usage: Some(0xE3),
    },
    Key {
        name: "LAlt",
        matrix: 17,
        rect: [122, 235, 157, 262],
        default_usage: Some(0xE2),
    },
    Key {
        name: "Space",
        matrix: 35,
        rect: [174, 235, 400, 262],
        default_usage: Some(0x2C),
    },
    Key {
        name: "RCtrl",
        matrix: 59,
        rect: [463, 236, 497, 263],
        default_usage: Some(0xE4),
    },
    Key {
        name: "FN",
        matrix: 53,
        rect: [414, 235, 449, 262],
        default_usage: None,
    },
    Key {
        name: "Left",
        matrix: 77,
        rect: [529, 234, 554, 261],
        default_usage: Some(0x50),
    },
    Key {
        name: "Down",
        matrix: 83,
        rect: [568, 234, 593, 261],
        default_usage: Some(0x51),
    },
    Key {
        name: "Right",
        matrix: 89,
        rect: [605, 234, 630, 261],
        default_usage: Some(0x4F),
    },
    Key {
        name: "PgDn",
        matrix: 87,
        rect: [604, 159, 629, 186],
        default_usage: Some(0x4E),
    },
    Key {
        name: "Wheel (knob)",
        matrix: 84,
        rect: [0, 0, 0, 0],
        default_usage: None,
    },
];

/// Effects in API row order (0-based). Row 18 is self-define (per-key), row 19 is Off.
pub static EFFECTS: [Effect; 20] = [
    Effect {
        row: 0,
        hw: 1,
        name: "Fixed on",
        speed: false,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 1,
        hw: 2,
        name: "Respire",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 2,
        hw: 3,
        name: "Rainbow",
        speed: true,
        brightness: true,
        colour: false,
    },
    Effect {
        row: 3,
        hw: 4,
        name: "Flash away",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 4,
        hw: 5,
        name: "Raindrops",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 5,
        hw: 6,
        name: "Rainbow wheel",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 6,
        hw: 7,
        name: "Ripples shining",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 7,
        hw: 8,
        name: "Stars twinkle",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 8,
        hw: 9,
        name: "Shadow disappear",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 9,
        hw: 10,
        name: "Retro snake",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 10,
        hw: 11,
        name: "Neon stream",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 11,
        hw: 12,
        name: "Reaction",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 12,
        hw: 13,
        name: "Sine wave",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 13,
        hw: 14,
        name: "Retinue scanning",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 14,
        hw: 15,
        name: "Rotating windmill",
        speed: true,
        brightness: true,
        colour: false,
    },
    Effect {
        row: 15,
        hw: 16,
        name: "Colorful waterfall",
        speed: true,
        brightness: true,
        colour: false,
    },
    Effect {
        row: 16,
        hw: 17,
        name: "Blossoming",
        speed: true,
        brightness: true,
        colour: false,
    },
    Effect {
        row: 17,
        hw: 18,
        name: "Rotating storm",
        speed: true,
        brightness: true,
        colour: true,
    },
    Effect {
        row: 18,
        hw: 21,
        name: "Self-define",
        speed: false,
        brightness: false,
        colour: false,
    },
    Effect {
        row: 19,
        hw: 0,
        name: "Off",
        speed: false,
        brightness: false,
        colour: false,
    },
];

/// Case-insensitive lookup by key name.
pub fn by_name(name: &str) -> Option<&'static Key> {
    KEYS.iter().find(|k| k.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_sizes_and_lookup() {
        assert_eq!(KEYS.len(), 81);
        assert_eq!(EFFECTS.len(), 20);
        assert_eq!(by_name("esc").map(|k| k.matrix), Some(0));
        assert_eq!(by_name("FN").and_then(|k| k.default_usage), None);
        assert!(by_name("nope").is_none());
    }
}
