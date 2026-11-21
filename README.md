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

## Releases e atualização automática
O app verifica uma nova versão ao abrir e mostra um aviso com o botão "Atualizar". Falhas na verificação (sem rede, `tauri dev`, repositório sem release) são ignoradas em silêncio. A atualização é baixada, conferida pela assinatura do updater e instalada pelo instalador NSIS, que fecha e reabre o app.

Para publicar uma versão:
1. Aumente `version` em `app/src-tauri/tauri.conf.json`, `app/src-tauri/Cargo.toml` e `app/package.json`; atualize `app/package-lock.json` (`npm install --package-lock-only` em `app/`) e `Cargo.lock` (`cargo check -p app`).
2. Faça o commit, crie a tag `vX.Y.Z` e envie: `git tag vX.Y.Z` e `git push origin vX.Y.Z`.
3. O workflow `.github/workflows/release.yml` compila o instalador NSIS, assina e publica o release com `latest.json`.

Segredos do repositório (Settings → Secrets and variables → Actions):
- `TAURI_SIGNING_PRIVATE_KEY`: conteúdo de `C:\Users\diego\.tauri\openf75.key`.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: conteúdo de `C:\Users\diego\.tauri\openf75.key.password`.

A chave privada nunca entra no repositório. **Se ela for perdida, os apps já instalados não aceitam mais atualizações assinadas com uma nova chave**: eles precisam ser reinstalados à mão, e o campo `pubkey` de `tauri.conf.json` precisa ser trocado.

## Limitações
- **Feche o OemDrv.exe** (software da AULA) antes de gravar; os dois disputam a mesma interface HID.
- Pelo receptor: a leitura da paleta de cores (`0x49`) não responde: a cor exibida é a padrão, o backup não inclui a paleta, e cor personalizada é recusada (use os slots 0-6 ou o cabo). O áudio e o tempo real rodam a ~3 fps (melhor no cabo). Uma camada de teclas volta com 504 bytes; o resto é preenchido com zeros.
- O caminho do cabo foi implementado a partir do protocolo recuperado, mas só foi testado em unidade.
- Protocolo: veja `docs/PROTOCOL.md`.
