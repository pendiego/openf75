# OpenF75: open-source AULA F75 configurator (Rust + Tauri, Windows)

## Context
Diego wants an open-source replacement for the AULA F75 OEM software (`C:\Program Files (x86)\AULA\F75\OemDrv.exe`, 32-bit MFC, PDB `d:\OEM\Beiying\Src\UI4\Debug\OemDrv.pdb`). It is built in Rust, with the HID protocol recovered by reverse engineering that binary. Requested scope:
- lighting effects (brightness, speed, colour), per-key RGB, base and Fn layer remapping, sleep and debounce, on-board macros, audio-reactive lighting and profiles (save, load, import, export);
- a library, a CLI and a Tauri 2 + Svelte desktop app;
- USB cable (`258A:010C`) and 2.4 GHz receiver (`3554:FA09`); Bluetooth is not covered;
- Windows only, MIT license.

Target directory: `C:\Users\diego\Documents\Projetos\aula`. It is empty and is not a git repo. Use the toolchain already installed: rustc/cargo 1.98.1 MSVC, node 24.19, npm 12.2 and gh.

RE workbench: Craft Anything 0.6.1 (`C:\Users\diego\tools\craft-anything\craft-anything-cli.exe`, portable), already registered as MCP server `craft-anything` in `~/.omp/agent/mcp.json` (args `["mcp"]`, so Clean room by default). Ghidra 12.1.4 is at `C:\Users\diego\tools\ghidra_12.1.4_PUBLIC` and Java 21 is found, but Craft Anything does not know the Ghidra path yet (`engines.ghidra.install_dir = null`). Observed: Deep mode without Ghidra finds no function at `0x48b4d0` (`FUN_0048b4d0`), so the built-in engine alone cannot replace the Ghidra evidence. Step 0 wires it up.

Provenance rule: code is written from scratch using the protocol facts below. Do not copy code from `trenbolone1122/aula-f75-linux`, which has no license. Do not ship any OEM file (keyimg.png, skins, text.xml, KB.ini, DLLs). Layout coordinates and protocol constants are facts and may be embedded.

## Protocol facts (recovered with Ghidra 12.1.4 headless from OemDrv.exe SHA-256 `169d8279…e34e`, installer `f9938e1e…ad886`)
Evidence files are in the session's `local/ghidra/` directory (`dump.c`, `dump2.c`, `q1.c`), and the function addresses below are VAs. Anything marked [INFERENCE] has a fallback that is already decided in Approach; Step 0 tries to settle each one with Craft Anything before coding.

### Discovery
- **Cable** (class CDevG5KB, device type 0x18, `FUN_0048b120`): the HID collection with usage page `0xFF00`, usage `0x01` and feature report length 520 (0x208). The report ID is `0x06`.
- **Receiver** (class CDev3632, `FUN_0040c530`, matched by VID/PID): the collection with usage page `0xFF02`, usage `0x02` and input/output length 20. On this PC that is `\\?\hid#vid_3554&pid_fa09&mi_01&col01…`, observed live with HidP_GetCaps. The other collections are the keyboard, consumer, mouse and system interfaces plus an `0xFF04` feature-8 collection; ignore them.
- **Cable model check** (`FUN_0048c550`): send feature `06 82 01 00 01 00 06 00` + zeros (520 bytes), wait 20 ms, then get feature 520. Bytes `[8..14]` hold the 6-byte "Psd". KB.ini variants:
  - `03 00 00 00 00 cd`: tri-mode ANSI F75 (Diego's model);
  - `03 00 00 00 00 e1`: wired-only F75, same key table;
  - `03 00 00 00 01 91`: F75 KR, different layout, unsupported.
- **Receiver online flag** (vtable +0x2c, `FUN_00486c10`): read command `0x07`, 1 byte. A value of 0 means the keyboard is asleep or off (OEM text: "The device is asleep or disconnected"). The OEM ApplySetting refuses to write while this is 0 (`FUN_004873d0`). Unsolicited input reports with `b1==0x0a, b5==0x02` carry the online flag in `b6`.
- **Firmware version**: HID attribute `VersionNumber` (hidapi `release_number`).

### Cable transport (feature report 0x06, 520 bytes; `FUN_0048b4d0`, `FUN_0048bc00`)
- Frame layout: `[0]=0x06 [1]=cmd [2]=sub [3]=0 [4]=total_chunks [5]=chunk_idx [6..8]=chunk_len u16 LE [8..]=payload`. Chunks are at most 512 bytes; `total = ceil(len/512)`.
- Write: sleep `delay` ms (20, or 7 for realtime), then `HidD_SetFeature`, retrying up to 3 times with 70 ms between attempts.
- Read: send the same header with `cmd|0x80` (for example `0x84`), sub and `[6..8]`=requested chunk length. Sleep 20 ms, call `HidD_GetFeature` with buffer[0]=0x06, and take the payload from byte 8. Repeat per 512-byte chunk with chunk_idx incremented. Reads are not retried.

### Receiver transport (output/input report 0x13, 20 bytes; `FUN_004862f0`, `FUN_004865d0`, `FUN_00485fa0`, `FUN_00486f40`)
- Packet layout: `[0]=0x13 [1]=cmd [2]=total [3]=idx [4]=(sub<<4)|len(0..14) [5..19]=payload (zero-padded) [19]=sum(bytes 0..19) & 0xFF`.
- Write:
  - split the data into 14-byte chunks;
  - before each packet sleep `delay` (15 ms for settings, 10 ms for realtime);
  - WriteFile with a 500 ms timeout, retried up to 4 times with 10 ms between;
  - unless `cmd & 0x80 != 0` or `cmd == 0x06`, wait for the ack. Read up to 20 reports (1000 ms each) and skip those with `b0 != 0x13` or `b1 != cmd`. `b0 == 0xFF` is an error. `b2 & 0x80` is a CRC error: resend the packet, at most 5 retries. `b3` must equal the packet index.
- Read:
  - send `13 <cmd> 01 00 <sub<<4> 00… <cks>`;
  - collect replies where `(b1 & 0x7F) == (cmd & 0x7F)`, `total = b2 & 0x7F`, `idx = b3`, `len = b4 & 0x0F`, data at `b5`;
  - ignore repeated indices and stop when `idx+1 == total`;
  - on a 1000 ms timeout, retry the whole read twice.

### Command table
| Function | Cable write / read | Receiver write / read | sub | Size |
|---|---|---|---|---|
| Key matrix layer | 0x03 / 0x83 | 0x01 / 0x41 | layer (0 = base, 1 = Fn) | 512 |
| LED/settings block | 0x04 / 0x84 | 0x04 / 0x44 | 0 | 128 |
| Macro buffer | 0x05 (one transfer, chunked at 512) | 0x03, 512-byte pages, sub = page, 100 ms between pages | 0 / page | ≤4096 |
| Per-key colours | 0x06 [INFERENCE write] / 0x86 (cable read, observed in `FUN_0048bc00`) | 0x02 / 0x42 [INFERENCE read] | colour group 0 | 384 |
| Realtime frame | 0x08, delay 7 ms | 0x88 (no ack), delay 10 ms | 0 | 378 |
| RGB palette table | 0x0A / 0x8A | 0x09 / 0x49 [INFERENCE read] | 0 | 512 |
| Model Psd | — / 0x82 (sub 1) | — | | 6 |
| Online | — | — / 0x07 | 0 | 1 |
| Factory reset | 0x11 (1 zero byte) | 0x06 (1 zero byte, no ack) | 0 | 1 |

The OEM write order is in `FUN_0048dbc0`:
1. dirty key layers (50 ms apart);
2. the macro buffer;
3. read the LED block (retry up to 5 times until `[126..128] == 5A A5`, 100 ms apart), edit it, sleep 60 ms, write it back;
4. if the selected mode is not self-define, sleep 130 ms and write the RGB table;
5. if the selected mode is self-define, sleep and write the per-key group.

There is no separate save command: writes persist. The receiver path sets busy, sleeps 200 ms and runs the same sequence (`FUN_004873d0`).

### LED/settings block (128 bytes; editor `FUN_0048d810`)
- `[3]` debounce (written because KB.ini `ShowDebounce=0x04000101`; the OEM default is `(0x04000101>>8)&0xFF = 1`). The range 0..=4 is an [INFERENCE] from the high byte.
- `[9]` is 1 when the selected mode is self-define (row index 18).
- `[10]` hardware effect id.
- `[11]` = StartGp (default 0x20; KB.ini has none) + colour group 0, so 0x20.
- `[0x18]` sleep: SleepHW value, or 0 = never.
- `[0x38] = [0x39] = 0xFF`.
- For each effect row `r` (0-based, 20 rows):
  - `[0x3A+2r]` = brightness 0..9;
  - `[0x3B+2r]` = `(speed 0..4) << 4 | colour_nibble`, where the nibble is a palette index 0..6, or 7 = random/colourful (RandomVal default 7).
- `[126]=0x5A`, `[127]=0xA5`.
- Every other byte (including `0x16`, `0x17`, `0x1A`) is preserved as read.

### Effects (KB.ini `LedOpt` rows; name = `tc_kb_led<row>`; hw id = column 0)
Row → hw id → flags (speed, brightness, colour/random):
- 1 Fixed on → 1 → (no, yes, yes)
- 2 Respire → 2
- 3 Rainbow → 3 → (yes, yes, no)
- 4 Flash away → 4
- 5 Raindrops → 5
- 6 Rainbow wheel → 6
- 7 Ripples shining → 7
- 8 Stars twinkle → 8
- 9 Shadow disappear → 9
- 10 Retro snake → 10
- 11 Neon stream → 11
- 12 Reaction → 12
- 13 Sine wave → 13
- 14 Retinue scanning → 14
- 15 Rotating windmill → 15 → (yes, yes, no)
- 16 Colorful waterfall → 16 → (yes, yes, no)
- 17 Blossoming → 17 → (yes, yes, no)
- 18 Rotating storm → 18
- 19 Self-define (per-key) → 21 → (no, no, no)
- 20 Off → 0 → (no, no, no)

Rows without explicit flags are (yes, yes, yes).

### RGB palette table (512 bytes)
- 21 rows × 21 bytes. Row 0 is all zeros.
- Row `r+1` holds 7 colours × `R,G,B` for effect row `r`. `RGBIndex=0,1,2` means R, G, B in that order.
- `[506]=0x5A`, `[507]=0xA5` (KB.ini `CRC=1`); the rest is zero.
- The colour nibble selects one of the 7 colours in the current row. A custom colour therefore means writing it into the row's selected slot.

### Per-key buffer (384 bytes)
- R plane `[0..126)`, G plane `[126..252)`, B plane `[252..378)`, then 6 zero bytes.
- Index = matrix index of the key (see the Layout table). The layout is planar, not interleaved (GameRGBIndex 0,1,2).
- It is displayed when the effect is row 19 (hw 21, `[9]=1`).

### Realtime frame (378 bytes; `FUN_0047f250`)
- 126 LEDs × interleaved `R,G,B` at `led*3` (led = matrix index).
- The OEM sends this full frame with sub 0 over the receiver. `FUN_0047f250` also has a branch when device flag `+0x234 == 1` that sends a 378-byte (`0x17a`) buffer through the device send at vtable `+0x38` (derived). Not verified: the command byte in that buffer, and whether flag 1 is the cable. The other branch builds grouped entries with a `0x88` stride, which is the compact sub-2 shape. Contingency B stays the fallback if the hardware rejects the full frame.

### Key codes (4 bytes per slot, layer = 128 slots × 4 = 512; `FUN_0048c820`)
- Keyboard: `[0x00, modifier_bits, 0x00, hid_usage]`. Modifier bits: LCtrl 0x01, LShift 0x02, LAlt 0x04, LGui 0x08, RCtrl 0x10, RShift 0x20, RAlt 0x40, RGui 0x80.
- Consumer: `[0x02, 0x00, usage_hi, usage_lo]`.
- Macro: `[0x03, mode, count, macro_index]`. mode 1 = repeat `count` times (1..255); 2 = repeat until the key is released (count 1); 4 = repeat until any key is pressed (count 1).
- Fn key: `[0x0D,0,0,0]`. Disabled: `[0,0,0,0]`.
- Anything else, such as the Fn-layer firmware functions `[0x07,…]`, `[0x08,…]` and `[0x09,…]`, is preserved as raw.
- The OEM writes 4 layers; F75 uses layer 0 (base) and layer 1 (Fn).

### Macro buffer (`FUN_0048d2d0`, `FUN_0048cf00`)
- Header: N × `(offset u16 LE, size u16 LE)`, with offset counted from the buffer start (it includes the N×4 header).
- Each macro blob is `[name_byte_len u8][name UTF-16LE, max 30 chars][events…]`. The blob size includes the name.
- Event, 4 bytes:
  - byte 0 = `(type<<4) | ((delay_ms>>16)&0x0F) | (0x80 if key-up)`, where type 0 = key and type 1 = modifier (0xE0..0xE7);
  - byte 1 = `delay>>8`, byte 2 = `delay&0xFF`, with delay clamped to 0xFFFFF;
  - byte 3 = HID usage.
- The OEM capacity is 4096 bytes (MacroBufferSize default), and only the used length is sent.

### Sleep values
- UI seconds 60, 90, 120, 180, 240, 300, 600, 900, 1200 map to HW values 2, 3, 4, 6, 8, 10, 20, 30, 40.
- Never = 0.
- The OEM notes that sleep applies only in wireless mode.

## Approach

### Step 0: Craft Anything RE workbench (settle the [INFERENCE] items)
- **Setup** (once):
  - `craft-anything-cli settings-set --key engines.ghidra.install_dir --value '"C:/Users/diego/tools/ghidra_12.1.4_PUBLIC"'`, then `craft-anything-cli doctor` must list Ghidra as found;
  - project `.omp/mcp.json` with server name `craft-anything` (same name, so it shadows the user entry for this repo only): command = the exe above, args `["mcp", "--deep", "--project", "C:\\Users\\diego\\Documents\\Projetos\\aula\\re\\oemdrv.cany"]`. Restart the omp session so it loads;
  - in the MCP session call `enable_toolset` for `native` (and `runtime` for Step 6/verification captures). Do not use `--all-tools`: 323 tools flood the context.
  - Fallback if the MCP does not load: every tool is also a CLI subcommand with dashes, e.g. `craft-anything-cli --deep --project re\oemdrv.cany native-function-pseudocode "C:\Program Files (x86)\AULA\F75\OemDrv.exe" --function 0x48b4d0`.
- **Workflow**:
  1. `core_target_open` on `C:\Program Files (x86)\AULA\F75\OemDrv.exe`;
  2. `core_unknown_add` once per open question below, with the probes listed;
  3. probe with `native_function_pseudocode`, `native_function_callers_list`, `native_xrefs_list`, `native_strings_find`, `native_bytes_find`;
  4. settled → `core_finding_record` (confidence `observed` or `derived`, evidence ids from step 3) and update the Protocol facts here and in Assumptions, dropping the [INFERENCE] tag; not settled → `core_unknown_edit` with what was tried, and keep the decided fallback.
- **Open questions**:
  - per-key read: **partly settled**. Cable read command `0x86` confirmed in `FUN_0048bc00` (fnd_1). Receiver read `0x42` still open.
  - receiver palette read: **open**. `0x49` wrappers `FUN_00486b70` (receiver) and `FUN_0048aab0` (cable) both send sub 0. They are reached only through vtable slots, so no named caller was found.
  - debounce range: **open**. The INI key `OPT/Debounce` is read into `+0x1f58`. The valid range and clamp were not found.
  - cable realtime: **partly settled** (fnd_2). A 378-byte send path exists, but its command byte and cable mapping are unverified. Hardware test decides.
  - receiver macro pages: **settled** (fnd_3). Ack is per 14-byte packet inside `FUN_004862f0`. Pages are separated by 100 ms only, with no page-level ack.
- **Provenance**: the `.cany` project holds decompiled OEM code, so `re/` is gitignored and never shipped. Only facts (offsets, constants, addresses) move into this plan and `docs/PROTOCOL.md`; no pseudo-C is pasted into the repo.
- Non-goal: `craft-anything-cli rebuild` / `rebuild_start`. It would have its own agent build a parallel project from a Deep read of the OEM, which breaks the provenance rule.

### Step 1: Workspace scaffold
- `git init`. Add `LICENSE` (MIT, "Copyright (c) 2026 Diego Ruas") and `.gitignore` (`target/`, `node_modules/`, `app/build/`, `app/.svelte-kit/`, `re/`, `.omp/`).
- Root `Cargo.toml`: workspace members `crates/openf75`, `crates/openf75-cli`, `app/src-tauri`; `resolver = "2"`. Set `[profile.release] lto = true, codegen-units = 1, strip = true`.
- `crates/openf75` (lib) deps:
  - `hidapi = { version = "2", default-features = false, features = ["windows-native"] }`;
  - `anyhow = "1"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `cpal = "0.15"`, `rustfft = "6"`.
- `crates/openf75-cli` (bin name `openf75`) deps: `openf75`, `clap = { version = "4", features = ["derive"] }`, `anyhow`.
- Contingency: if `windows-native` fails to compile or lacks feature reports, use default hidapi features. MSVC is present.

### Step 2: Pure protocol layer (`crates/openf75/src/{layout.rs, proto.rs, macros.rs}`; no I/O)
- **`layout.rs`**:
  - `pub struct Key { pub name: &'static str, pub matrix: u8, pub rect: [u16; 4], pub default_usage: Option<u8> }`;
  - `pub const KEYS: [Key; 81]`, copied from the Layout table below, in its order;
  - `pub fn by_name(&str) -> Option<&Key>` (case-insensitive);
  - `pub const EFFECTS: [Effect; 20]` with `Effect { row: u8, hw: u8, name: &'static str, speed: bool, brightness: bool, colour: bool }` from the Effects list;
  - `SLEEP_TABLE: [(u32 secs, u8 hw); 9]`.
- **`proto.rs`**:
  - constants: `CABLE_VIDPID = (0x258A, 0x010C)`, `DONGLE_VIDPID = (0x3554, 0xFA09)`, `LED_LEN = 128`, `LAYER_LEN = 512`, `PERKEY_LEN = 384`, `TABLE_LEN = 512`, `FRAME_LEN = 378`, `MACRO_CAP = 4096`, `KNOWN_PSD: [[u8; 6]; 2] = [[3,0,0,0,0,0xcd],[3,0,0,0,0,0xe1]]`;
  - `pub enum Cmd { Matrix, Led, Macro, PerKey, Realtime, Table, Reset }` with `fn cable(self) -> u8` and `fn dongle(self) -> u8` returning the command table values;
  - `pub fn checksum(p: &[u8; 20]) -> u8` (sum of bytes 0..19);
  - `pub fn dongle_packets(cmd: u8, sub: u8, data: &[u8]) -> Vec<[u8; 20]>`;
  - `pub fn dongle_read_request(cmd: u8, sub: u8) -> [u8; 20]`;
  - `pub fn cable_frames(cmd: u8, sub: u8, data: &[u8]) -> Vec<[u8; 520]>`;
  - `pub fn cable_read_header(cmd: u8, sub: u8, chunk_idx: u8, total: u8, len: u16) -> [u8; 520]`.
- **`LedBlock(pub [u8; 128])`**:
  - `fn validate(&self) -> Result` checks the 5A A5 trailer;
  - `fn effect_row() -> Option<usize>` matches `[10]` against EFFECTS hw. API rows are 0-based (0..=19, self-define = 18, off = 19); only the CLI and UI show the 1-based row numbers from the Effects list;
  - `fn set_effect(row, brightness, speed, colour_nibble)` writes `[9]` (1 iff row == 18), `[10]`, `[11]=0x20`, `[0x38]=[0x39]=0xFF` and the row's slot pair;
  - `fn slot(row) -> (brightness, speed, nibble)`;
  - `fn set_sleep_hw(u8)` writes `[0x18]`; `fn set_debounce(u8)` writes `[3]` and rejects values above 4.
- **`PaletteTable(pub [u8; 512])`**:
  - `get(row, slot) -> [u8; 3]` and `set(row, slot, rgb)` at offset `(row+1)*21 + slot*3`;
  - `fn defaults() -> Self`: every row gets `[red, green, blue, yellow, magenta, cyan, white]` (FF0000, 00FF00, 0000FF, FFFF00, FF00FF, 00FFFF, FFFFFF), plus the trailer.
- **`PerKey(pub [u8; 384])`**: `get(matrix) -> [u8; 3]`, `set(matrix, rgb)` (planar).
- `pub fn realtime_frame(colours: &[(u8 matrix, [u8; 3])]) -> [u8; 378]`.
- **`KeyCode([u8; 4])`**:
  - `enum KeyAction { Key { mods: u8, usage: u8 }, Consumer(u16), Macro { mode: u8, count: u8, index: u8 }, Fn, Disabled, Raw([u8; 4]) }`;
  - `decode` and `encode` exactly per Key codes;
  - serde-serialisable as a tagged enum.
- **`macros.rs`**:
  - `#[derive(Serialize, Deserialize, Clone)] pub struct Macro { pub name: String, pub events: Vec<MacroEvent> }` and `pub struct MacroEvent { pub usage: u8, pub down: bool, pub delay_ms: u32 }`;
  - `pub fn encode_buffer(macros: &[Macro]) -> Result<Vec<u8>>`: header plus blobs per the Macro buffer facts; type = 1 when `usage` is in 0xE0..=0xE7, else 0;
  - errors: a name longer than 30 chars, or a total above 4096 → `"macros excedem 4096 bytes"`;
  - `pub fn text_to_events(text: &str) -> Vec<MacroEvent>` for ASCII printable characters and `\n`: down/up pairs, 10 ms delays, LShift 0xE1 wrapped around shifted characters, US layout.
- Unit tests in `proto.rs` and `macros.rs` with golden bytes:
  - `dongle_packets(0x04, 0, &[0u8; 128])` yields 10 packets; packet 0 is `13 04 0A 00 0E …` and the last is `13 04 0A 09 02 …`, each with a valid checksum;
  - `cable_frames(0x03, 1, &[0; 512])` has header `06 03 01 00 01 00 00 02`;
  - `KeyAction::Consumer(0xCD)` encodes to `02 00 00 CD`, `Key { mods: 0x02, usage: 0x04 }` to `00 02 00 04`, and `Macro { mode: 1, count: 3, index: 2 }` to `03 01 03 02`;
  - a one-macro buffer with name "ab", events [down 0x04 delay 300, up 0x04 delay 0] equals `04 00 0D 00 | 04 61 00 62 00 | 00 01 2C 04 | 80 00 00 04` (header offset 4, size 13 = 1 + 4 + 8);
  - `PerKey::set(81, [1,2,3])` puts bytes at 81, 207 and 333;
  - `LedBlock::set_effect(row 1, b9, s2, nibble 3)` writes `[10]=2`, `[0x3C]=9`, `[0x3D]=0x23`.

### Step 3: HID transport and device API (`crates/openf75/src/device.rs`)
- **`Transport`**: `pub enum Transport { Cable(HidDevice), Dongle(HidDevice) }`.
- **`pub fn open() -> Result<Device>`**:
  - enumerate `HidApi::new()?.device_list()`;
  - prefer the cable (usage_page 0xFF00, usage 1) and fall back to the receiver (usage_page 0xFF02, usage 2);
  - neither found → `"Teclado AULA F75 não encontrado (cabo ou receptor 2.4G)"`.
- **Device state**: `pub struct Device { t: Transport, pub kind: Kind /*Cable|Dongle*/, pub psd: Option<[u8; 6]>, pub fw: u16 }`.
- For the cable, `open` reads the Psd. `pub fn writable(&self) -> Result<()>` fails with `"modelo não suportado: psd=…"` when the psd is not in `KNOWN_PSD`.
- `writable` also fails with `"Feche o software da AULA (OemDrv.exe) antes de gravar"` when `tasklist /FI "IMAGENAME eq OemDrv.exe" /NH` output contains `OemDrv.exe`. Reason: the OEM polls the same collection and would interleave multi-packet transfers.
- For the receiver, `writable` fails with `"Teclado dormindo ou desligado: pressione uma tecla"` when `online()` is 0.
- Every write method calls `writable()` first. Read methods never do.
- **Primitives**:
  - `fn write(cmd: Cmd, sub: u8, data: &[u8])` and `fn read(cmd: Cmd, sub: u8, len: usize) -> Result<Vec<u8>>`, implementing the transport facts (delays, retries, ack matching) per kind;
  - cable reads use `get_feature_report` into a 520 buffer with `[0]=6`;
  - receiver reads use `read_timeout(&mut [0; 20], 1000)`;
  - realtime (`Cmd::Realtime`) uses delay 7 on the cable and 10 on the receiver, no ack, and no 5A A5 checks.
- **High-level methods**:
  - `pub fn online(&self) -> Result<bool>`: receiver = read 0x07, 1 byte, `!= 0`; cable = true;
  - `pub fn led(&self) -> Result<LedBlock>`: read 128 bytes, retrying up to 5 times 100 ms apart until `validate()` passes; otherwise error `"bloco LED inválido (sem 5A A5)"`;
  - `pub fn set_led(&self, b: &LedBlock)`: validate, sleep 60 ms, write;
  - `pub fn table(&self) -> Result<PaletteTable>`: read 512 bytes. If the read fails or the trailer is missing, return `PaletteTable::defaults()`;
  - `pub fn set_table(&self, &PaletteTable)`: sleep 130 ms, write;
  - `pub fn layer(&self, n: u8) -> Result<[u8; 512]>`: reject n > 1. Error if the data is all 0x00 or all 0xFF;
  - `pub fn set_layer(&self, n, data)`: preserve slot 127 as read, write, sleep 50 ms;
  - `pub fn perkey(&self) -> Result<PerKey>`: read 384 bytes; on failure return all-zero;
  - `pub fn set_perkey(&self, &PerKey)`: sleep 100 ms, write;
  - `pub fn set_macros(&self, buf: &[u8])`: cable = one write of cmd 0x05; receiver = 512-byte pages with cmd 0x03, sub = page, sleeping 100 ms between pages;
  - `pub fn realtime(&self, frame: &[u8; 378])`;
  - `pub fn factory_reset(&self)`.
- **Composite operations**, reusing the methods above in the OEM order:
  - `apply_effect(row, brightness, speed, Colour)`, where `enum Colour { Slot(u8), Custom([u8; 3]), Random }`:
    1. read the LED block and edit it;
    2. for `Custom`, read the table, write the RGB into the slot currently selected for that row (or slot 0 if the nibble is 7), and use that slot;
    3. write the LED block;
    4. if `row != 18`, write the table.
  - `apply_perkey(&PerKey)`: set effect row 18 in the LED block, write the LED block, then write the per-key buffer.
- **Backups** (`pub fn snapshot(&self) -> Result<Snapshot>`):
  - `Snapshot { led: Vec<u8>, table: Vec<u8>, perkey: Vec<u8>, layers: [Vec<u8>; 2], macros: Vec<Macro> }`;
  - `macros` comes from storage, because the device macro buffer is not read back;
  - `pub fn restore(&self, s: &Snapshot)` writes layers 0 and 1, the macros buffer (if any), LED, table and per-key, in that order.
- **`storage.rs`**:
  - base directory `%APPDATA%\openf75\`, created on demand;
  - `macros.json` (`Vec<Macro>`), `profiles\<name>.json` (`Profile { name, snapshot }`), `backups\<YYYYMMDD-HHMMSS>.json`;
  - `pub fn ensure_first_backup(dev)` writes a backup only when `backups\` is empty. It is called on the first successful `open()` from the app and CLI.
  - Timestamps use `std::time::SystemTime`, formatted with UTC fields computed manually. No chrono.
- **Macro delete** (`pub fn delete_macro(dev, i)`):
  1. read layers 0 and 1;
  2. if any slot decodes to `Macro { index: i }`, error `"macro em uso nas teclas: …"` with the key names;
  3. otherwise decrement every `index > i`, write both layers, re-encode and write the buffer, and save `macros.json`.
- **Macro save** (`pub fn save_macros(dev, &[Macro])`) encodes, writes the buffer and saves `macros.json`.

### Step 4: Audio-reactive lighting (`crates/openf75/src/audio.rs`)
- `pub struct AudioViz { stop: Arc<AtomicBool>, thread: Option<JoinHandle<()>> }` with `pub fn start(dev: Arc<Mutex<Option<Device>>>, gain: f32) -> Result<AudioViz>` and `stop(self)`. This is the same handle type as the app's `AppState.dev`; the CLI wraps its device the same way. A frame is skipped when the slot is `None`, and the thread exits on the first send error.
- Capture: cpal WASAPI loopback, via `default_host().default_output_device()` and `build_input_stream` with `default_output_config()`. Mix to mono into a 2048-sample ring (`Arc<Mutex<VecDeque<f32>>>`). Build and hold the `cpal::Stream` inside the worker thread, because it is `!Send`. `start` returns an error if the build fails, reported back through a `std::sync::mpsc` channel before the loop begins.
- A worker thread runs every 33 ms (≈30 fps):
  1. Hann window, rustfft 2048, 15 log-spaced bands from 60 Hz to 12 kHz;
  2. magnitude → dB → level in 0..1 using gain (UI range 0.5–4.0), attack immediate, release ×0.85 per frame;
  3. a key's band = `floor(((x1+x2)/2 − 30) / 600 × 15)` clamped to 0..14;
  4. a key's row = by y1: 30 → 5, 80/81 → 4, 119/120 → 3, 158/159 → 2, 195/196 → 1, ≥234 → 0;
  5. the key lights if `level ≥ (row+1)/6`, coloured green rows 0–1, yellow rows 2–3, red rows 4–5; otherwise black;
  6. lock the device and send `realtime_frame`.
- On stop: join the thread, then rewrite the current LED block once (`set_led(&led()?)`) so the firmware returns to the stored effect.
- Contingency A: if streamed frames do not show on the keyboard, `start` saves the LED block, switches to row 18 (self-define) before streaming, and `stop` restores the saved block.
- Contingency B: if the cable ignores the full frame, send the compact sub-2 format: groups `[R,G,B,count,led…]`, terminated by 3 zero bytes, length ≤ 378.
- Contingency C: if cpal loopback fails on input-stream build, replace capture with the `wasapi` crate (`wasapi = "0.15"`, loopback on the default render device).

### Step 5: CLI (`crates/openf75-cli/src/main.rs`, clap derive)
- Subcommands:
  - `status`: kind, fw, psd hex, online, OEM running, effect name + slot;
  - `dump [--out file.json]`: snapshot printed as hex or saved;
  - `restore <file.json>`;
  - `effect <name-or-row> [--brightness 0-9] [--speed 0-4] [--slot 0-6 | --rgb RRGGBB | --random]`. The row is the 1-based number from the Effects list (`2` = Respire); the name match is case-insensitive;
  - `perkey [--all RRGGBB] [KEY=RRGGBB …]`, using key names from `layout::by_name`;
  - `remap <base|fn> <KEY> <target>`, with target `key:<usage hex>[+ctrl|shift|alt|win…]`, `media:<usage hex>`, `macro:<index>[:times:N|:hold|:any]`, `off`, `raw:XXXXXXXX`;
  - `macro-text <KEY> <text>`: append `text_to_events` as a macro, save, and assign to the base layer with mode 1 count 1;
  - `sleep <secs|never>` (must match a SLEEP_TABLE entry);
  - `debounce <0-4>`;
  - `reset --yes`;
  - `audio --seconds N [--gain 1.5]`.
- Each command opens the device, calls `ensure_first_backup`, and prints errors as `erro: …` with exit code 1.

### Step 6: Tauri app (`app/`)
- **Scaffold**: from the repo root run `npx create-tauri-app@latest app --template svelte-ts --manager npm --identifier io.github.pendiego.openf75 --yes --force`, then `npm install` in `app/`.
  - Set productName `OpenF75` and `bundle.targets: ["nsis"]`.
  - Add `tauri-plugin-dialog = "2"` with `.plugin(tauri_plugin_dialog::init())`, permission `dialog:default`, and `npm i @tauri-apps/plugin-dialog`.
  - `src-tauri` depends on `openf75 = { path = "../../crates/openf75" }` and `serde`.
- **State**: `struct AppState { dev: Arc<Mutex<Option<Device>>>, viz: Mutex<Option<AudioViz>> }`. A `with_dev` helper opens the device lazily and drops it (sets `None`) on any transport error, so the next call reconnects.
  - Shortcut: no hotplug thread; the UI polls `status` every 2 s. Upgrade only if reconnect latency annoys.
  - Commands return `Result<T, String>`.
- **Commands** (`src-tauri/src/lib.rs`), each a thin wrapper over the Step 3/4 functions:
  - `layout() -> Vec<Key>` (`KEYS` serialised, so `Key` derives `Serialize`);
  - `status() -> Status { connected, kind, fw, psd, online, oem_running, writable_error: Option<String> }`;
  - `read_lighting() -> Lighting { row, slots: Vec<(u8, u8, u8)>, palette: Vec<[[u8; 3]; 7]>, perkey: Vec<[u8; 3]> /*indexed by matrix, 126*/ }`;
  - `apply_effect(row, brightness, speed, colour)`;
  - `apply_perkey(colours: Vec<(u8, [u8; 3])>)`;
  - `read_layer(layer) -> Vec<KeyAction>` (128 entries);
  - `set_key(layer, matrix, action: KeyAction)`: read, modify one slot, write;
  - `list_macros`, `save_macros(macros)`, `delete_macro(index)`;
  - `set_sleep(secs: Option<u32>)`, `set_debounce(v)`, `get_power() -> (Option<u32>, u8)`;
  - `factory_reset()`, `backup_now() -> path`, `restore_backup(path)`;
  - `list_profiles`, `save_profile(name)` (snapshot + macros), `apply_profile(name)` (restore + overwrite `macros.json`), `delete_profile(name)`, `export_profile(name, path)`, `import_profile(path)`;
  - `audio_start(gain)`, `audio_stop()`.
- **UI** (`src/routes/+page.svelte` plus components in `src/lib/`; Portuguese labels; dark theme with plain CSS variables, no UI library):
  - header with connection badge (Cabo / Receptor 2.4G / Desconectado), firmware and a red banner with `writable_error`;
  - tabs **Iluminação · Por tecla · Teclas · Macros · Energia · Áudio · Perfis**.
- **`Keyboard.svelte`** (shared):
  - SVG `viewBox="0 0 660 290"`, one `<rect>` per `KEYS` entry with a non-zero rect (Wheel is excluded) and a centred label;
  - props `fill: (matrix) => string`, `selected: Set<number>` and an `onpick(matrix, shiftKey)` event;
  - mouse-drag painting on the Por tecla tab.
  - The layout data is mirrored to the front end through a `layout()` command returning `KEYS` (serde). There is no TS copy.
- **Iluminação**: effect list (20 rows); brightness slider 0–9 and speed slider 0–4 shown only per the Effect flags; 7 palette swatches for the row plus a "Aleatório" toggle; an `<input type=color>` editing the selected swatch; an "Aplicar" button.
- **Por tecla**: colour picker, a paint/select mode, "Preencher tudo" and "Aplicar".
- **Teclas**:
  - layer toggle Base/Fn; clicking a key opens a side panel with the decoded current action and an editor;
  - editor kinds: Tecla (select from `hid.ts` + 8 modifier checkboxes), Mídia (Play/Pause 0xCD, Próxima 0xB5, Anterior 0xB6, Parar 0xB7, Mudo 0xE2, Vol+ 0xE9, Vol− 0xEA, Calculadora 0x192, E-mail 0x18A, Navegador 0x223, Meu computador 0x194), Macro (macro select + mode N vezes / Até soltar / Até outra tecla + count), Desativada, Raw (8 hex chars);
  - the FN key (matrix 53) is shown read-only, so the Fn layer cannot be locked out.
- **`src/lib/hid.ts`**: the only TS table, `{ code: KeyboardEvent.code, label, usage }` for HID usages 0x04–0x65 and 0xE0–0xE7 (US names).
- **Macros**:
  - list with new/rename/delete; recorder: a focused box capturing `keydown`/`keyup` with `preventDefault`, mapping `event.code` through `hid.ts` (unmapped codes ignored) and storing the delay as the ms difference;
  - editable event table (usage, ↓/↑, delay); "Salvar no teclado" calls `save_macros`;
  - limits shown: 30-character name, 4096-byte total (computed in Rust; the error is shown).
- **Energia**: sleep select (Nunca, 1, 1,5, 2, 3, 4, 5, 10, 15, 20 min) with the note "só no modo sem fio"; debounce select 0–4; "Restaurar padrão de fábrica" with a confirm dialog → `factory_reset`.
- **Áudio**: gain slider 0.5–4 and Iniciar/Parar. While running, the other write actions are disabled in the UI.
- **Perfis**: list, save current, apply, delete, export/import via `@tauri-apps/plugin-dialog` `save`/`open` (filter `json`), and "Fazer backup agora" / "Restaurar backup".
- **OEM UI reference** (optional, before building the tabs): `mkdir re`, then call MCP `native_window_capture` with `{"program": "C:\\Program Files (x86)\\AULA\\F75\\OemDrv.exe", "output": "C:\\Users\\diego\\Documents\\Projetos\\aula\\re\\oem-ui.png", "ui_tree": "C:\\Users\\diego\\Documents\\Projetos\\aula\\re\\oem-ui.json", "wait_secs": 8}`. Every path must be absolute and its folder must exist; the MCP key is `wait_secs` (`wait-secs` is rejected; the dashed `--wait-secs` is only the CLI flag). Use it to inventory the OEM controls and ranges (combo items, slider limits). Reference only; nothing from it is shipped. The tool closes the program afterwards; still run the `Stop-Process -Name OemDrv` prerequisite before any hardware write.

### Step 7: Docs
- `README.md` (PT-BR):
  - what it is and that it is unaffiliated with AULA;
  - features, build/run commands, CLI usage;
  - "feche o OemDrv.exe";
  - limitations: Bluetooth, F75 KR, receiver macro/audio speed.
- `docs/PROTOCOL.md`: the Protocol facts section above, with function addresses, as the RE record. Write it from the settled Step 0 findings (`core_investigation_show`), facts only. `core_report_write` may produce a local draft in `re/`, which is not committed because it can quote pseudo-C.

## Layout table (KB.ini `[KEY]`, tri-mode ANSI; rect = x1,y1,x2,y2 in a 660×290 canvas; usage = default HID)
| name | matrix | rect | usage |
|---|---|---|---|
| Esc | 0 | 30,30,55,57 | 0x29 |
| F1 | 12 | 107,30,132,57 | 0x3A |
| F2 | 18 | 144,30,169,57 | 0x3B |
| F3 | 24 | 183,30,208,57 | 0x3C |
| F4 | 30 | 222,30,247,57 | 0x3D |
| F5 | 36 | 269,30,294,57 | 0x3E |
| F6 | 42 | 305,30,330,57 | 0x3F |
| F7 | 48 | 343,30,368,57 | 0x40 |
| F8 | 54 | 388,30,413,57 | 0x41 |
| F9 | 60 | 439,30,464,57 | 0x42 |
| F10 | 66 | 477,30,502,57 | 0x43 |
| F11 | 72 | 516,30,541,57 | 0x44 |
| F12 | 78 | 555,30,580,57 | 0x45 |
| End | 88 | 605,195,630,222 | 0x4D |
| ` | 1 | 31,81,56,108 | 0x35 |
| 1 | 7 | 70,81,95,108 | 0x1E |
| 2 | 13 | 108,81,133,108 | 0x1F |
| 3 | 19 | 147,81,172,108 | 0x20 |
| 4 | 25 | 184,81,209,108 | 0x21 |
| 5 | 31 | 223,81,248,108 | 0x22 |
| 6 | 37 | 262,81,287,108 | 0x23 |
| 7 | 43 | 300,80,325,107 | 0x24 |
| 8 | 49 | 338,81,363,108 | 0x25 |
| 9 | 55 | 377,81,402,108 | 0x26 |
| 0 | 61 | 416,81,441,108 | 0x27 |
| - | 67 | 453,81,478,108 | 0x2D |
| = | 73 | 491,81,516,108 | 0x2E |
| Backspace | 79 | 531,81,591,108 | 0x2A |
| Delete | 85 | 604,81,629,108 | 0x4C |
| Tab | 2 | 34,120,78,147 | 0x2B |
| Q | 8 | 89,119,114,146 | 0x14 |
| W | 14 | 127,119,152,146 | 0x1A |
| E | 20 | 165,119,190,146 | 0x08 |
| R | 26 | 204,119,229,146 | 0x15 |
| T | 32 | 242,119,267,146 | 0x17 |
| Y | 38 | 280,119,305,146 | 0x1C |
| U | 44 | 318,119,343,146 | 0x18 |
| I | 50 | 356,119,381,146 | 0x0C |
| O | 56 | 395,119,420,146 | 0x12 |
| P | 62 | 433,119,458,146 | 0x13 |
| [ | 68 | 471,120,496,147 | 0x2F |
| ] | 74 | 509,119,534,146 | 0x30 |
| \ | 80 | 549,119,593,146 | 0x31 |
| PgUp | 86 | 605,120,630,147 | 0x4B |
| CapsLock | 3 | 33,158,85,185 | 0x39 |
| A | 9 | 98,158,123,185 | 0x04 |
| S | 15 | 136,158,161,185 | 0x16 |
| D | 21 | 174,158,199,185 | 0x07 |
| F | 27 | 213,158,238,185 | 0x09 |
| G | 33 | 252,158,277,185 | 0x0A |
| H | 39 | 290,158,315,185 | 0x0B |
| J | 45 | 327,158,352,185 | 0x0D |
| K | 51 | 366,158,391,185 | 0x0E |
| L | 57 | 405,158,430,185 | 0x0F |
| ; | 63 | 443,158,468,185 | 0x33 |
| ' | 69 | 481,158,506,185 | 0x34 |
| Enter | 81 | 521,158,591,185 | 0x28 |
| LShift | 4 | 33,195,105,222 | 0xE1 |
| Z | 10 | 116,195,141,222 | 0x1D |
| X | 16 | 155,195,180,222 | 0x1B |
| C | 22 | 193,195,218,222 | 0x06 |
| V | 28 | 232,195,257,222 | 0x19 |
| B | 34 | 270,196,295,223 | 0x05 |
| N | 40 | 309,195,334,222 | 0x11 |
| M | 46 | 347,195,372,222 | 0x10 |
| , | 52 | 384,195,409,222 | 0x36 |
| . | 58 | 424,195,449,222 | 0x37 |
| / | 64 | 463,195,488,222 | 0x38 |
| RShift | 70 | 502,195,558,222 | 0xE5 |
| Up | 82 | 567,195,592,222 | 0x52 |
| LCtrl | 5 | 34,234,69,261 | 0xE0 |
| LWin | 11 | 79,234,114,261 | 0xE3 |
| LAlt | 17 | 122,235,157,262 | 0xE2 |
| Space | 35 | 174,235,400,262 | 0x2C |
| RCtrl | 59 | 463,236,497,263 | 0xE4 |
| FN | 53 | 414,235,449,262 | none (code `0D 00 00 00`) |
| Left | 77 | 529,234,554,261 | 0x50 |
| Down | 83 | 568,234,593,261 | 0x51 |
| Right | 89 | 605,234,630,261 | 0x4F |
| PgDn | 87 | 604,159,629,186 | 0x4E |
| Wheel (knob) | 84 | 0,0,0,0 | none (raw `07 00 00 1D`) |

## Critical files & anchors
- `C:\Program Files (x86)\AULA\F75\Dev\kb\1\KB.ini`: source of the layout, effects, sleep and Psd values (read-only reference; do not ship).
- `crates/openf75/src/device.rs` `Device::write/read`: the only place with timing, retry and ack logic; everything else is pure.
- `crates/openf75/src/proto.rs` `LedBlock::set_effect`: must preserve unknown bytes; only the listed offsets change.
- `app/src-tauri/src/lib.rs` `with_dev`: the single device handle shared with the audio thread (Mutex), so transfers never interleave.

## Verification
Prerequisites:
- `C:\Users\diego\Documents\Projetos\aula`, receiver plugged in (it is now), keyboard awake;
- close OEM software first: `powershell -NoProfile -Command "Stop-Process -Name OemDrv -ErrorAction SilentlyContinue"`.

0. Step 0 done: `craft-anything-cli doctor` lists Ghidra as found, and `core_unknowns_show` shows each question either settled by a finding or still open with its fallback noted in Assumptions.
1. `cargo test -p openf75` passes, including the golden-byte tests in Step 2.
2. Read-only on hardware: `cargo run -p openf75-cli -- status` prints `Receptor 2.4G`, `online: sim` and an effect name. `cargo run -p openf75-cli -- dump --out %TEMP%\f75-before.json`:
   - the LED block ends `5A A5`;
   - layer 0 slot 0 is `00 00 00 29` (Esc);
   - `%APPDATA%\openf75\backups\` now holds exactly one file.
3. Effect write: `openf75 effect 2 --brightness 5 --speed 3 --rgb 00FF00`, then `openf75 status` shows `Respire, brilho 5, velocidade 3` with a palette slot `00FF00`. Ask Diego to confirm the keyboard breathes green.
4. Per-key: `openf75 perkey --all 000000 W=FF0000 A=FF0000 S=FF0000 D=FF0000`. Status shows `Self-define`; `dump` shows per-key bytes R at 14/9/15/21 = FF. Ask Diego to confirm that WASD lights red.
5. Remap: `openf75 remap base CapsLock key:0x29` (Caps → Esc). Re-reading layer 0 shows slot 3 = `00 00 00 29`. Ask Diego to press Caps in Notepad; it must act as Esc.
6. Macro: `openf75 macro-text F12 "ola"` on the receiver. Pressing F12 in Notepad types `ola`. If nothing happens on the receiver, ask Diego to switch to cable and repeat; record the result in README limitations.
7. Sleep/debounce: `openf75 sleep 300` then `status` shows sleep 300 s (`[0x18]=0x0A`); `openf75 debounce 1` reads back 1.
8. Audio: play music, then `openf75 audio --seconds 10`. Diego confirms the bars move. If not, apply Step 4 Contingency A, then B.
9. Restore: `openf75 restore %TEMP%\f75-before.json`. `dump` equals the before file byte for byte (LED, table, layers).
10. App:
    - `cd app; npm run check` passes;
    - launch the app with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 npm run tauri dev` (bash async) and drive it per skill `tauri-cdp-invoke-smoke-windows`;
    - `invoke('status')` returns `connected: true`; `invoke('read_layer', {layer: 0})[0]` decodes to `Key{usage: 0x29}`;
    - screenshot each tab and read the PNGs; each tab renders the keyboard SVG with 80 keys and no console errors;
    - apply one effect from the Iluminação tab by clicking via CDP and confirm by `invoke('read_lighting')`;
    - finally `npm run tauri build` produces `target\release\bundle\nsis\OpenF75_*_x64-setup.exe`;
    - capture the release build (not just the dev server). Resolve the absolute exe path from Cargo, not a hard-coded `target\release`: `TD=$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)`, `BIN=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | select(.manifest_path | test("src-tauri")) | .targets[] | select(.kind | index("bin")) | .name')`, `EXE=$(cygpath -w "$TD/release/$BIN.exe")`, `OUT=$(cygpath -w "$TEMP/openf75-release.png")`. Check `test -f "$EXE"`, then call MCP `native_window_capture` with `{"program": "<EXE>", "output": "<OUT>", "wait_secs": 8}`, the expanded absolute values substituted in (CLI equivalent: `craft-anything-cli native-window-capture --program "$EXE" --output "$OUT" --wait-secs 8`). Read the PNG: the window opens and renders the keyboard.
11. Cable path:
    - ask Diego to set the side switch to cable and plug the USB-C cable, then repeat steps 2–3 and 9 (`status` must print `Cabo`, `psd 03 00 00 00 00 cd`);
    - if the cable is unavailable, report the cable path as verified only by unit tests.

## Assumptions & contingencies
- Diego's keyboard is the tri-mode ANSI F75 (Psd `…00 cd`, `Dev\kb\1`). F75 KR and Bluetooth are out of scope: the app shows "não suportado" or does not detect them.
- [INFERENCE] reads 0x86/0x42 (per-key) and 0x49 (receiver palette): Step 0 tries to settle them first. If still unsettled or wrong on hardware, fall back to all-black per-key and `PaletteTable::defaults()`, as specified in Step 3.
- Craft Anything is early alpha. If its Ghidra integration fails on OemDrv.exe, run Ghidra 12.1.4 headless directly (as the original evidence was produced) and record the answers in this plan by hand.
- [INFERENCE] debounce range 0–4: if a write of 4 reads back different, clamp the UI to the values that read back correctly.
- Receiver macros: the OEM path exists (512-byte pages). If step 6 fails on the receiver but works on the cable, the app disables "Salvar no teclado" on the receiver with the message "Macros exigem o cabo USB".
- Realtime/audio over the receiver works at about 3 fps (27 packets × 10 ms); it is allowed but labelled "melhor no cabo".
