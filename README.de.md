<div align="center">

<img src="app/app-icon.svg" width="96" height="96" alt="">

# OpenF75

**Quelloffener Konfigurator für die AULA F75 Tastatur.**<br>
Beleuchtung, Farben pro Taste, Neubelegung, Makros und Profile, ohne die Software des Herstellers.

[![Herunterladen](https://img.shields.io/badge/Herunterladen-releases-e8e6df)](https://github.com/pendiego/openf75/releases/latest)
![Windows](https://img.shields.io/badge/Windows-getestet-2ea44f?logo=windows&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-experimentell-f0ad4e?logo=linux&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-experimentell-f0ad4e?logo=apple&logoColor=white)
[![Lizenz MIT](https://img.shields.io/badge/Lizenz-MIT-blue)](#lizenz)

[**Herunterladen**](https://github.com/pendiego/openf75/releases/latest) · [Funktionen](#funktionen) · [Installation](#installation) · [CLI](#kommandozeile) · [Einschränkungen](#einschränkungen)

[English](README.md) · [Português](README.pt-BR.md) · [Español](README.es.md) · [Français](README.fr.md) · **Deutsch** · [日本語](README.ja.md) · [简体中文](README.zh-CN.md)

<br>

<img src="docs/screenshots/iluminacao.png" alt="Reiter Beleuchtung: 3D-Modell der F75 mit dem Effekt Rainbow und der Effektgalerie mit Live-Vorschauen" width="860">

</div>

## Funktionen

| Funktion | Was sie tut |
|---|---|
| **Beleuchtung** | Die 20 Tastatureffekte mit Live-Vorschau auf einem 3D-Modell, Helligkeit, Geschwindigkeit und Farbe |
| **Pro Taste** | Tasten einzeln oder bereichsweise färben und auf die Tastatur schreiben |
| **Audioreaktiv** | Die Tasten werden zu einem Equalizer für den Klang, der auf dem Computer läuft |
| **Tasten** | Die Ebenen Base und Fn neu belegen: Tasten, Medien, Makros oder deaktivieren |
| **Makros** | Sequenzen mit Verzögerung pro Ereignis aufnehmen, direkt im Speicher der Tastatur |
| **Energie** | Ruhezeit, Debounce und Funktion des Mausrads (Lautstärke oder Beleuchtung) |
| **Profile** | Konfigurationen speichern, anwenden, importieren und exportieren; automatische Sicherung bei der ersten Verbindung |
| **Befehlspalette** | <kbd>Ctrl</kbd> <kbd>K</kbd> (<kbd>⌘</kbd> <kbd>K</kbd> unter macOS) für jeden Reiter oder jede Aktion |

Helles und dunkles Design sowie automatische Aktualisierung.

<table>
  <tr>
    <td><img src="docs/screenshots/teclas.png" alt="Reiter Tasten: Neubelegung der Ebene Base"></td>
    <td><img src="docs/screenshots/macros.png" alt="Reiter Makros: Ereigniseditor mit Taste, Aktion und Verzögerung"></td>
  </tr>
  <tr>
    <td align="center"><sub>Neubelegung der Ebenen</sub></td>
    <td align="center"><sub>Makro-Editor</sub></td>
  </tr>
</table>

### Sprachen (in Entwicklung)

Die Oberfläche der App wird in der nächsten Version auf Portugiesisch (Brasilien), Englisch, Spanisch, Französisch, Deutsch, Japanisch und vereinfachtem Chinesisch verfügbar sein. Beim ersten Start übernimmt die App die Systemsprache und fällt auf Englisch zurück, wenn diese nicht unterstützt wird. Zum Wechseln nutzen Sie die Auswahl in der Statusleiste unten im Fenster, neben Firmware und Version, oder suchen die Sprache in der Befehlspalette; die Wahl wird gespeichert.

Die CLI und die Fehlermeldungen von Rust bleiben auf Portugiesisch; die App zeigt bekannte Fehler in der gewählten Sprache an.

### Kompatibilität

| | USB-Kabel `258A:010C` | 2.4-GHz-Empfänger `3554:FA09` | Bluetooth |
|---|:---:|:---:|:---:|
| AULA F75 | ✅ | ✅ | ❌ |
| AULA F75 KR | ❌ | ❌ | ❌ |

| System | Paket | Status |
|---|---|---|
| Windows 10/11 | NSIS-Installer | Mit der Tastatur getestet |
| Linux (x64) | `.deb` und AppImage | CI-Validierung ausstehend, ohne Hardwaretest |
| macOS 13+ (Intel und Apple Silicon) | `.dmg` | CI-Validierung ausstehend, ohne Hardwaretest |

## Installation

Laden Sie das Paket für Ihr System von der [Releases-Seite](https://github.com/pendiego/openf75/releases/latest) herunter.

> [!IMPORTANT]
> Schließen Sie unter Windows **die Software von AULA (`OemDrv.exe`)**, bevor Sie schreiben: Beide Programme konkurrieren um dieselbe Schnittstelle der Tastatur. OpenF75 warnt, wenn sie geöffnet ist.

<details>
<summary><b>Windows</b></summary>

Führen Sie den Installer aus. Es sind kein Treiber und keine zusätzliche Regel nötig. Die App aktualisiert sich selbst.

</details>

<details>
<summary><b>Linux</b></summary>

- **`.deb`**: installiert die udev-Regel, die die Tastatur für den angemeldeten Benutzer freigibt. Zum Aktualisieren installieren Sie das neue `.deb`.
- **AppImage**: aktualisiert sich selbst, die udev-Regel ist aber nicht enthalten. Installieren Sie sie einmalig:

  ```sh
  sudo cp packaging/linux/70-openf75.rules /etc/udev/rules.d/
  sudo udevadm control --reload && sudo udevadm trigger
  ```

Die Regel verwendet `TAG+="uaccess"`, was von systemd-logind abhängt. Wenn die App die Tastatur nicht öffnen kann, verweist die Fehlermeldung darauf.

Der audioreaktive Modus erfasst den Monitor des Standardausgangs (PulseAudio oder PipeWire).

</details>

<details>
<summary><b>macOS</b></summary>

Die App ist ad-hoc signiert und nicht notarisiert (es gibt kein Apple-Zertifikat). Klicken Sie beim ersten Mal mit der rechten Maustaste auf die App und wählen Sie **Öffnen**. Wenn macOS weiterhin blockiert:

```sh
xattr -dr com.apple.quarantine /Applications/OpenF75.app
```

macOS erlaubt Apps nicht, den Systemklang aufzunehmen. Für den audioreaktiven Modus installieren Sie ein Loopback-Gerät wie [BlackHole](https://github.com/ExistentialAudio/BlackHole) und leiten die Ausgabe dorthin. macOS fragt nach der Mikrofonberechtigung: Darüber liest die App den Loopback.

</details>

Makros, Profile und Sicherungen liegen in:

| Windows | macOS | Linux |
|---|---|---|
| `%APPDATA%\openf75\` | `~/Library/Application Support/openf75/` | `$XDG_CONFIG_HOME/openf75/` oder `~/.config/openf75/` |

## Kommandozeile

Die CLI `openf75` macht dasselbe wie die App, für Skripte. Sie benötigt stabiles Rust; unter Linux installieren Sie vorher `libudev-dev`, `libasound2-dev` und `pkg-config`.

```sh
cargo install --path crates/openf75-cli   # installiert den Befehl openf75 in den PATH

openf75 status                                   # nur lesen
openf75 dump --out backup.json                   # nur lesen: vor dem Schreiben ausführen
openf75 effect 2 --brightness 5 --speed 3 --rgb 00FF00
openf75 perkey --all 000000 W=FF0000 A=FF0000
openf75 remap base CapsLock key:0x29
openf75 macro-text F12 "ola"
openf75 sleep 300
openf75 debounce 1
openf75 restore backup.json
```

## Entwicklung

```sh
cd app
npm install
npm run tauri dev            # App im Dev-Modus
npm run check                # svelte-check
cargo test -p openf75        # Protokolltests
```

Build des Installers: `npm run tauri build` unter Windows; unter Linux ergänzen Sie `-- --bundles deb,appimage`; unter macOS `-- --target universal-apple-darwin --bundles app,dmg`.

| Pfad | Inhalt |
|---|---|
| [`crates/openf75`](crates/openf75) | Rust-Bibliothek: HID-Protokoll, Gerät, Audio, Dateien |
| [`crates/openf75-cli`](crates/openf75-cli) | CLI `openf75` |
| [`app/`](app) | Desktop-App (Tauri 2 + Svelte 5) |
| [`docs/PROTOCOL.md`](docs/PROTOCOL.md) | Protokoll der F75, durch Reverse Engineering ermittelt |

<details>
<summary><b>Eine Version veröffentlichen</b></summary>

1. Erhöhen Sie `version` in `app/src-tauri/tauri.conf.json`, `app/src-tauri/Cargo.toml` und `app/package.json`; aktualisieren Sie `app/package-lock.json` (`npm install --package-lock-only` in `app/`) und `Cargo.lock` (`cargo check -p app`).
2. Committen Sie, erstellen Sie den Tag und pushen Sie: `git tag vX.Y.Z` und `git push origin vX.Y.Z`.
3. Der Workflow [`release.yml`](.github/workflows/release.yml) baut auf `windows-latest`, `ubuntu-22.04` und `macos-latest`, nacheinander (`max-parallel: 1`), weil `tauri-action` die `latest.json` zusammenführt, die der Updater liest.

Die App sucht beim Öffnen nach einer neuen Version und zeigt die Schaltfläche **Aktualisieren** an. Die Aktualisierung wird über die Signatur des Updaters geprüft. Unter Linux aktualisiert sich nur das AppImage selbst.

Repository-Geheimnisse (Settings → Secrets and variables → Actions):
- `TAURI_SIGNING_PRIVATE_KEY`: Inhalt von `%USERPROFILE%\.tauri\openf75.key`.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: Inhalt von `%USERPROFILE%\.tauri\openf75.key.password`.

> [!CAUTION]
> Der private Schlüssel gehört niemals ins Repository. Geht er verloren, akzeptieren die installierten Apps keine Updates mehr: Sie müssen dann von Hand neu installiert und der `pubkey` in `tauri.conf.json` ersetzt werden.

</details>

## Einschränkungen

**Tastatur und Verbindung**

- **Nur AULA F75.** Das F75 KR (anderes Layout) und Bluetooth werden nicht unterstützt.
- **USB-Kabel:** Das Protokoll wurde durch Reverse Engineering umgesetzt und hat nur Unit-Tests; es wurde nie mit der Tastatur getestet.
- **2.4-GHz-Empfänger:**
  - Die Farbpalette (`0x49`) reagiert nicht: Die angezeigte Farbe ist die Standardfarbe, Sicherungen enthalten die Palette nicht, das Wiederherstellen überspringt sie, und eigene Farben werden abgelehnt (nutzen Sie die Slots 0-6 oder das Kabel).
  - Audioreaktive und Echtzeit-Modi laufen mit etwa 3 fps.
  - Eine Tastenebene kommt mit 504 von 512 Bytes an; der Rest wird mit Nullen aufgefüllt.
  - Die erste Antwort kann etwa 400 ms dauern.
  - Schreibvorgänge schlagen fehl, solange die Tastatur schläft oder ausgeschaltet ist; drücken Sie eine Taste, um sie aufzuwecken.
- **Ruhezeit:** gilt nur im Funkmodus.
- **Rad:** Das Byte, das Lautstärke oder Beleuchtung auswählt, wurde über den Empfänger gefunden; es wurde nicht über das Kabel bestätigt.
- **Firmware:** OpenF75 aktualisiert keine Firmware. Der AULA-Treiber schreibt sie ebenfalls nicht: Er ruft ein externes Update-Tool auf, das im aktuellen F75-Paket nicht enthalten ist.
- **Makros:** insgesamt 4096 Bytes, Namen eingeschlossen; Namen bis 30 Zeichen; Verzögerungen bis 1.048.575 ms (etwa 17 Min.) pro Ereignis.

**Systeme**

- **Windows:** Schließen Sie die AULA-Software (`OemDrv.exe`) vor dem Schreiben. Es wird nur ein NSIS-Installer für x64 bereitgestellt.
- **Linux und macOS wurden nicht mit der Tastatur getestet,** und die Pakete haben die CI noch nicht bestanden. Führen Sie `openf75 status` und `openf75 dump` aus, bevor Sie auf diesen Systemen etwas schreiben.
- **Linux:** nur x64. Der Tastaturzugriff hängt von der udev-Regel ab, die `TAG+="uaccess"` verwendet und systemd-logind benötigt; das AppImage installiert die Regel nicht. Nur das AppImage aktualisiert sich selbst; die `.deb` wird manuell aktualisiert. Audioreaktive Modi benötigen PulseAudio oder PipeWire.
- **macOS:** nur Version 13 oder neuer. Die App ist nicht notarisiert, daher blockiert Gatekeeper den ersten Start. Das System erlaubt keine Aufnahme seines Klangs: Audioreaktive Modi benötigen ein Loopback-Gerät (BlackHole o. Ä.) und die Mikrofonberechtigung.

**Sprache**

- Die Übersetzungen der Oberfläche sind noch in Arbeit. Die CLI und die Fehlermeldungen aus Rust sind auf Portugiesisch; die App übersetzt nur die Fehler, die sie kennt.

## Lizenz

[MIT](LICENSE). Unabhängiges Projekt, ohne Zugehörigkeit zu AULA. Keine Datei des Herstellers ist Teil des Repositorys.
