# OpenF75

Configurador open source (MIT) para o teclado AULA F75, em Rust. Não é afiliado à AULA.

Biblioteca (`crates/openf75`), CLI (`openf75`) e app desktop (Tauri 2 + Svelte, `app/`). Windows apenas; cabo USB (`258A:010C`) e receptor 2.4 GHz (`3554:FA09`). Bluetooth e o F75 KR não são suportados.

## Recursos
- Efeitos de iluminação (brilho, velocidade, cor), cores por tecla, remapeamento das camadas Base e Fn, sleep e debounce.
- Macros gravadas no teclado, iluminação reativa ao áudio, perfis (salvar, aplicar, importar, exportar) e backup automático na primeira conexão (`%APPDATA%\openf75\backups`).

## Uso
```
cargo build --release -p openf75-cli
target\release\openf75 status
target\release\openf75 effect 2 --brightness 5 --speed 3 --rgb 00FF00
target\release\openf75 perkey --all 000000 W=FF0000 A=FF0000
target\release\openf75 remap base CapsLock key:0x29
target\release\openf75 macro-text F12 "ola"
target\release\openf75 sleep 300
target\release\openf75 debounce 1
target\release\openf75 dump --out backup.json
target\release\openf75 restore backup.json
```
App: `cd app && npm install && npm run tauri dev` (ou `npm run tauri build`).

## Limitações
- **Feche o OemDrv.exe** (software da AULA) antes de gravar; os dois disputam a mesma interface HID.
- Pelo receptor: a leitura da paleta de cores (`0x49`) não responde, então a cor atual de um efeito não é exibida e uma cor personalizada regrava a paleta inteira com os padrões. O áudio e o tempo real rodam a ~3 fps (melhor no cabo). Uma camada de teclas volta com 504 bytes; o resto é preenchido com zeros.
- O caminho do cabo foi implementado a partir do protocolo recuperado, mas só foi testado em unidade.
- Protocolo: veja `docs/PROTOCOL.md`.
