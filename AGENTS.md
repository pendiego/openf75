# OpenF75

Configurador open source (MIT) do teclado AULA F75: biblioteca Rust, CLI e app Tauri. Só Windows. Cabo USB `258A:010C` e receptor 2.4G `3554:FA09`; Bluetooth e F75 KR fora de escopo.

Idioma: textos de UI, ajuda da CLI, mensagens de erro e README em pt-BR. Comentários de código e commits em inglês.

## Mapa

| Caminho | O que é |
|---|---|
| `crates/openf75/src/proto.rs` | Enquadramento dos pacotes e blocos (`LedBlock`, `PaletteTable`, `PerKey`, `KeyAction`). Puro, sem I/O; é onde ficam os testes. |
| `crates/openf75/src/device.rs` | I/O HID: `open()`, `Device`, `Snapshot`, `restore`. |
| `crates/openf75/src/layout.rs` | Tabelas `KEYS`, `EFFECTS`, `SLEEP_TABLE`. |
| `crates/openf75/src/macros.rs` | Codificação do buffer de macros. |
| `crates/openf75/src/storage.rs` | Arquivos em `%APPDATA%\openf75\` (`macros.json`, perfis, `backups\first.json`). |
| `crates/openf75/src/audio.rs` | Loopback de áudio para frames em tempo real. |
| `crates/openf75-cli/` | Binário `openf75` (clap). |
| `app/src-tauri/src/lib.rs` | Comandos Tauri. |
| `app/src/lib/api.ts` | Espelho tipado dos comandos; `app/src/lib/tabs/*.svelte` são as telas. |
| `docs/PROTOCOL.md` | Fonte da verdade do protocolo e do fluxo de atualização de firmware da OEM. Atualize junto com qualquer descoberta. |
| `app/app-icon.svg` | Fonte do ícone do app; `app/src-tauri/icons/` e `app/static/favicon.png` são gerados a partir dele. |
| `.github/workflows/release.yml` | Release por tag `v*`: instalador NSIS assinado + `latest.json` do auto-updater. Config do updater em `plugins.updater` de `tauri.conf.json`; `createUpdaterArtifacts` só em `tauri.updater.conf.json`, então build local não precisa da chave. Chave privada em `%USERPROFILE%\.tauri\openf75.key` (+ `.password`), nunca no repo; perder a chave quebra a atualização de quem já instalou. Passos da release no README. |
| `docs/keyboard-model/` | Fonte do modelo 3D (`f75.sc`, scripts, refs). Renders, `.blend` e `.glb` intermediários ficam fora do git. |
| `re/` | Bancada de engenharia reversa, fora do git. Nenhum asset da OEM entra no repositório. |
| `OPENF75_CONFIGURATOR_PLAN.md` | Plano histórico; não é especificação viva. |

## Comandos

```
cargo test -p openf75                      # testes unitários (proto, macros, layout)
cargo build --release -p openf75-cli       # target\release\openf75.exe
cd app && npm run check                    # svelte-check
cd app && npm run tauri dev                # app em modo dev (porta 1420)
cd app && npm run tauri build -- --no-bundle   # target\release\app.exe, ~3,5 min
```

- O wrapper `rtk` esconde a saída do cargo e relata falhas falsas. Rode via `powershell -NoProfile -Command '...'` e leia a saída crua.
- Não há `jq` no PATH; use `ConvertFrom-Json`.
- Feche `app.exe` antes de recompilar o app.

## Regras de hardware

Erros aqui gravam lixo no teclado do usuário.

- **Nunca grave dado sintético.** `Device::table()` devolve a paleta padrão só para exibição; `table_read()` é a leitura real e retorna `None` no receptor. Snapshot, backup, restore e `apply_effect` usam apenas a leitura real.
- **Cor personalizada é recusada** quando a paleta não pode ser lida (receptor). Restore pula a paleta no receptor.
- **Ordem do restore de cores por tecla:** LED em self-define (linha 18), buffer por tecla, LED original. Fora dessa ordem o buffer não persiste.
- **`OemDrv.exe` disputa a mesma interface HID.** `Device::writable()` precisa passar antes de qualquer gravação.
- **Antes de testar gravação no teclado real,** rode `openf75 dump --out <arquivo>`. `status` e `dump` são só leitura. Arquivos `dump*.json` são ignorados pelo git.
- **Não restaure `first.json.synthetic-palette.bak`.**
- O caminho do cabo só tem teste unitário; não foi exercitado em hardware.
- **Atualização de firmware não existe no app.** O `OemDrv.exe` não grava firmware: só executa um atualizador da AULA (`Update.exe` ou um exe baixado) que não vem no pacote oficial. Veja `docs/PROTOCOL.md`, "Firmware update".

## Receptor 2.4G

Detalhes em `docs/PROTOCOL.md`, seção "Observado em hardware".

- `0x49` (paleta) nunca responde. `0x07`, `0x44`, `0x41`, `0x42` respondem.
- Uma camada de 512 bytes chega como 504; aceite `len-14` e preencha com zeros.
- Filtre respostas por `sub` (`byte4 >> 4`) e drene relatórios antigos antes de cada pedido, senão uma resposta atrasada casa com o pedido seguinte.
- A primeira resposta pode levar ~400 ms.

## Convenções

- **Comando Tauri novo:** função em `lib.rs` com `#[tauri::command(async)]`, registro no handler, entrada e tipo em `api.ts`. Comando síncrono roda na thread da UI e congela a janela durante leituras HID.
- **Efeitos:** a API usa linha 0-based (`0..=19`); UI e CLI mostram `row + 1`.
- **`Key.rect`** é `[x1, y1, x2, y2]` num canvas 660×290, não `[x, y, w, h]`.
- **Telas:** sucesso e erro passam por `notify(...)`.
- **Prévia da UI sem teclado:** `npx vite dev` em `app/` e injete um `window.__TAURI_INTERNALS__.invoke` falso no navegador antes do carregamento.
- **Commits:** Conventional Commits, assunto com no máximo 72 caracteres (hook de guarda).
- **Trabalho paralelo:** use um worktree (`git worktree add ../aula-<tarefa> -b <tarefa>`), não edite a árvore principal enquanto outro agente trabalha nela.
- **Ícone:** edite `app/app-icon.svg` e rode `npx tauri icon app-icon.svg -o ../re/iconout` em `app/`. Sem `-o` o comando cria pastas Android/iOS em `icons/`. Copie para `src-tauri/icons/` só os nomes que já existem lá e `64x64.png` para `static/favicon.png`. O `tauri dev` não reembute o ícone sozinho: dê `touch` em `src-tauri/build.rs`.

## UI

- Tokens e classes globais ficam em `app/src/app.css`; estilos com escopo usam só tokens, sem cor fixa.
- `--accent` é claro e faz par com `--accent-ink`. Nunca texto branco sobre `--accent`.
- Estado selecionado é `aria-pressed`, não uma classe.
- Confirmações são inline em dois passos (ver `Energia.svelte`), sem `confirm()`.
- Transições e tweens do Svelte usam `dur(ms)` de `lib/motion.ts`, que zera com movimento reduzido. O bloco CSS de `prefers-reduced-motion` não para animações feitas em JS.
- A vista 3D (`Keyboard.svelte`) anima tudo no loop `frame()`: troca de efeito com fade de LEDs de 0,3 s, "Vista inicial" com retorno suave e inércia ao soltar o giro. Respeita a constante `reduceMotion`. Um efeito estático precisa pintar o quadro final do fade antes de o loop parar.
