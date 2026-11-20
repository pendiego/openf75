# Protocolo HID do AULA F75 (recuperado por engenharia reversa)

Somente fatos (offsets, constantes, VAs do OemDrv.exe). Nenhum código do fabricante é distribuído.


## Discovery
- **Cable** (class CDevG5KB, device type 0x18, `FUN_0048b120`): the HID collection with usage page `0xFF00`, usage `0x01` and feature report length 520 (0x208). The report ID is `0x06`.
- **Receiver** (class CDev3632, `FUN_0040c530`, matched by VID/PID): the collection with usage page `0xFF02`, usage `0x02` and input/output length 20. On this PC that is `\\?\hid#vid_3554&pid_fa09&mi_01&col01…`, observed live with HidP_GetCaps. The other collections are the keyboard, consumer, mouse and system interfaces plus an `0xFF04` feature-8 collection; ignore them.
- **Cable model check** (`FUN_0048c550`): send feature `06 82 01 00 01 00 06 00` + zeros (520 bytes), wait 20 ms, then get feature 520. Bytes `[8..14]` hold the 6-byte "Psd". KB.ini variants:
  - `03 00 00 00 00 cd`: tri-mode ANSI F75 (Diego's model);
  - `03 00 00 00 00 e1`: wired-only F75, same key table;
  - `03 00 00 00 01 91`: F75 KR, different layout, unsupported.
- **Receiver online flag** (vtable +0x2c, `FUN_00486c10`): read command `0x07`, 1 byte. A value of 0 means the keyboard is asleep or off (OEM text: "The device is asleep or disconnected"). The OEM ApplySetting refuses to write while this is 0 (`FUN_004873d0`). Unsolicited input reports with `b1==0x0a, b5==0x02` carry the online flag in `b6`.
- **Firmware version**: HID attribute `VersionNumber` (hidapi `release_number`).

## Cable transport (feature report 0x06, 520 bytes; `FUN_0048b4d0`, `FUN_0048bc00`)
- Frame layout: `[0]=0x06 [1]=cmd [2]=sub [3]=0 [4]=total_chunks [5]=chunk_idx [6..8]=chunk_len u16 LE [8..]=payload`. Chunks are at most 512 bytes; `total = ceil(len/512)`.
- Write: sleep `delay` ms (20, or 7 for realtime), then `HidD_SetFeature`, retrying up to 3 times with 70 ms between attempts.
- Read: send the same header with `cmd|0x80` (for example `0x84`), sub and `[6..8]`=requested chunk length. Sleep 20 ms, call `HidD_GetFeature` with buffer[0]=0x06, and take the payload from byte 8. Repeat per 512-byte chunk with chunk_idx incremented. Reads are not retried.

## Receiver transport (output/input report 0x13, 20 bytes; `FUN_004862f0`, `FUN_004865d0`, `FUN_00485fa0`, `FUN_00486f40`)
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

## Command table
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

## LED/settings block (128 bytes; editor `FUN_0048d810`)
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

## Effects (KB.ini `LedOpt` rows; name = `tc_kb_led<row>`; hw id = column 0)
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

## RGB palette table (512 bytes)
- 21 rows × 21 bytes. Row 0 is all zeros.
- Row `r+1` holds 7 colours × `R,G,B` for effect row `r`. `RGBIndex=0,1,2` means R, G, B in that order.
- `[506]=0x5A`, `[507]=0xA5` (KB.ini `CRC=1`); the rest is zero.
- The colour nibble selects one of the 7 colours in the current row. A custom colour therefore means writing it into the row's selected slot.

## Per-key buffer (384 bytes)
- R plane `[0..126)`, G plane `[126..252)`, B plane `[252..378)`, then 6 zero bytes.
- Index = matrix index of the key (see the Layout table). The layout is planar, not interleaved (GameRGBIndex 0,1,2).
- It is displayed when the effect is row 19 (hw 21, `[9]=1`).

## Realtime frame (378 bytes; `FUN_0047f250`)
- 126 LEDs × interleaved `R,G,B` at `led*3` (led = matrix index).
- The OEM sends this full frame with sub 0 over the receiver. `FUN_0047f250` also has a branch when device flag `+0x234 == 1` that sends a 378-byte (`0x17a`) buffer through the device send at vtable `+0x38` (derived). Not verified: the command byte in that buffer, and whether flag 1 is the cable. The other branch builds grouped entries with a `0x88` stride, which is the compact sub-2 shape. Contingency B stays the fallback if the hardware rejects the full frame.

## Key codes (4 bytes per slot, layer = 128 slots × 4 = 512; `FUN_0048c820`)
- Keyboard: `[0x00, modifier_bits, 0x00, hid_usage]`. Modifier bits: LCtrl 0x01, LShift 0x02, LAlt 0x04, LGui 0x08, RCtrl 0x10, RShift 0x20, RAlt 0x40, RGui 0x80.
- Consumer: `[0x02, 0x00, usage_hi, usage_lo]`.
- Macro: `[0x03, mode, count, macro_index]`. mode 1 = repeat `count` times (1..255); 2 = repeat until the key is released (count 1); 4 = repeat until any key is pressed (count 1).
- Fn key: `[0x0D,0,0,0]`. Disabled: `[0,0,0,0]`.
- Anything else, such as the Fn-layer firmware functions `[0x07,…]`, `[0x08,…]` and `[0x09,…]`, is preserved as raw.
- The OEM writes 4 layers; F75 uses layer 0 (base) and layer 1 (Fn).

## Macro buffer (`FUN_0048d2d0`, `FUN_0048cf00`)
- Header: N × `(offset u16 LE, size u16 LE)`, with offset counted from the buffer start (it includes the N×4 header).
- Each macro blob is `[name_byte_len u8][name UTF-16LE, max 30 chars][events…]`. The blob size includes the name.
- Event, 4 bytes:
  - byte 0 = `(type<<4) | ((delay_ms>>16)&0x0F) | (0x80 if key-up)`, where type 0 = key and type 1 = modifier (0xE0..0xE7);
  - byte 1 = `delay>>8`, byte 2 = `delay&0xFF`, with delay clamped to 0xFFFFF;
  - byte 3 = HID usage.
- The OEM capacity is 4096 bytes (MacroBufferSize default), and only the used length is sent.

## Sleep values
- UI seconds 60, 90, 120, 180, 240, 300, 600, 900, 1200 map to HW values 2, 3, 4, 6, 8, 10, 20, 30, 40.
- Never = 0.
- The OEM notes that sleep applies only in wireless mode.


## Observado em hardware (receptor 2.4G, 2026-11)
- Leituras 0x07, 0x44 (LED), 0x41 (camadas) e 0x42 (por tecla) respondem; 0x49 (paleta) não responde.
- Uma leitura de camada (512 bytes) volta em 36 pacotes (504 bytes). Os pacotes carregam `sub` em `b4>>4`: filtre por ele e descarte respostas atrasadas de pedidos anteriores antes de cada leitura.
- Os dados por tecla só persistem se gravados com o efeito self-define selecionado; a ordem correta de restauração é LED (self-define), por tecla, LED original, paleta.
- Macro: ack por pacote de 14 bytes, 100 ms entre páginas de 512 bytes (OEM `FUN_004868a0`).
