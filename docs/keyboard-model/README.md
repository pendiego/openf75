# Referência para modelar o teclado AULA F75

Material para modelar o teclado e manter o modelo 3D do app.

## Estado atual

- O app renderiza `app/src/lib/f75.glb` com three.js em `app/src/lib/Keyboard.svelte`, nas abas Iluminação, Por tecla e Teclas.
- `app/src/lib/f75.glb` já vem dividido por tecla: `build_glb.py` separa cada grupo `Caps*`, `Legend*` e `LegendSide*` em um nó por tecla (o centro de cada triângulo, `px = x / k + 880`, `py = 383 - y / k`, cai num retângulo de `CAPS` em `app/src/lib/f75art.ts`) com extras `{group, key}`, e o resultado é comprimido com meshopt (3,8 MB para 1,0 MB). Os LEDs ficam por baixo das tampas: cada tecla tem um brilho aditivo sobre a placa que só aparece nos vãos (as tampas não são tingidas); nos temas Shine-Through as legendas também acendem.
- Carregamento: o download, o renderer WebGL e a compilação dos shaders começam quando o app importa `Keyboard.svelte`, em paralelo com as primeiras leituras do teclado. Renderer, cena e modelo são únicos na sessão: trocar de aba só move o canvas (≈10 ms, antes ≈300 ms refazendo tudo). Medido no build de produção em Chromium headless, do início da navegação ao primeiro quadro do teclado: ≈450-580 ms antes, ≈240 ms depois.
- Prévia dos efeitos (aba Iluminação): `app/src/lib/fx.ts` simula os 20 efeitos (`makeFx`, cor por tecla em função do tempo, com cor, velocidade e brilho escolhidos) e `Keyboard.svelte` anima a cada quadro com bloom (`UnrealBloomPass`, só quando há LED aceso). Efeitos reativos (Flash away, Ripples shining, Shadow disappear, Reaction) respondem às teclas digitadas no teclado físico (`KeyboardEvent.code` → uso HID → matriz) e aos cliques no modelo; as tampas descem 3,2 mm ao pressionar; sem digitação por 2,5 s, teclas fantasmas são pressionadas. É uma aproximação do firmware (formas e passos de velocidade não medidos no teclado). Com `prefers-reduced-motion`, a prévia fica estática.
- Clique e pintura usam raycast na cena (a tampa visível mais próxima ganha, mesmo com a vista baixa). Anel de seleção, contorno ao passar o mouse, marca de remapeamento (canto de trás à direita do topo da tampa, escondida quando outra peça a cobre) e foco ficam num SVG por cima do canvas: o polígono de cada tecla é o fecho convexo dos cantos da tampa projetados pela câmera, recalculado quando a vista muda. A distância da câmera é a menor que mantém os oito cantos da carcaça no quadro.
- Girar: arrastar fora das teclas (botão esquerdo) ou em qualquer lugar (botão direito); "Vista inicial" ou clique duplo fora das teclas volta à vista inicial.
- Modelo (tema de cores): botão "Modelo" abaixo do teclado abre a grade com uma prévia de cada tema, renderizada no próprio canvas (vista inicial, LEDs apagados) na primeira abertura e guardada até fechar o app. A escolha fica em `localStorage` (`openf75.theme`). As cores vêm de `app/src/lib/f75themes.ts`, gerado da tabela de configurações de `f75.sc` por `python docs/keyboard-model/themes.py` (rodar da raiz). Temas Side-Printed escondem `Legend*` e mostram `LegendSide*`.
- Para trocar o modelo: exportar do SolidCraft o tema Black inteiro (`f75.glb`) e, com "Black (Side-Printed)" ativo, só os corpos `LegendSideR1`–`R6` (`side.glb`); depois, nesta pasta, `python build_glb.py f75.glb side.glb f75-keys.glb` e `npx @gltf-transform/cli meshopt f75-keys.glb ../../app/src/lib/f75.glb` (apague `f75-keys.glb` depois). O contrato é manter os nomes dos corpos, mm com Z para cima e as tampas posicionadas a partir de `CAPS`.
- `Key.rect` em `crates/openf75/src/layout.rs` é o canvas 660x290 do desenho da OEM. Não serve para posicionar teclas no modelo: uma transformação afim única erra até 0,1u.

## Arquivos em `refs/`

|Arquivo|O que é|Licença|
|---|---|---|
|`top-render-1760x766.webp`|Renderização de topo usada antes do modelo 3D; `CAPS` foi medido nela.|CC BY 4.0, keshavV20|
|`sketchfab-8c7ba891-white-blue-green.jpg`|Miniatura do modelo branco/azul/verde (o usado).|CC BY 4.0, keshavV20|
|`sketchfab-5a8f600b-black-yellow.jpg`|Miniatura do modelo preto/amarelo, mesmo autor.|CC BY 4.0, keshavV20|

Fontes:
- https://sketchfab.com/3d-models/aula-f75-mechanical-keyboard-8c7ba891d2374124b020f0eb01173d35
- https://sketchfab.com/3d-models/aula-f75-mechanical-keyboard-5a8f600bae924cbbb2fe9af2b0b3c5ac

Não entram aqui: fotos de produto da AULA/Epomaker e qualquer asset extraído do software da OEM (regra do repositório). Podem ser consultadas no navegador como referência visual, nunca copiadas.

## Geometria medida na renderização

Unidades em pixels da imagem 1760x766. 1u (passo entre teclas) = 103 px; tampa de 1u = 96 px de largura; vão entre tampas = 7 px.

|Linha|y1–y2|Teclas (larguras em u)|
|---|---|---|
|F|59–150|Esc em x=60; F1–F4 a partir de x=266; F5–F8 a partir de x=710; F9–F12 a partir de x=1154; passo 103; knob à direita (centro ≈ 1650, 105, raio ≈ 55)|
|Números|197–288|` 1 2 3 4 5 6 7 8 9 0 - = (1u cada), Backspace (2u), Delete (1u)|
|Q|298–388|Tab (1,5u), Q…] (12 × 1u), \ (1,5u), PgUp (1u)|
|A|401–490|CapsLock (1,75u), A…' (11 × 1u), Enter (2,25u), PgDn (1u)|
|Z|503–593|LShift (2,25u), Z…/ (10 × 1u), RShift (1,75u), Up (1u), End (1u)|
|Inferior|608–687|LCtrl, LWin, LAlt (≈1,2u cada), Space (≈6,2u), Fn, RCtrl (≈1,15u cada), vão, Left, Down, Right (1u cada)|

- Coluna esquerda começa em x=60; a última coluna (Delete, PgUp, PgDn, End, Right) termina em x≈1705.
- A linha F fica separada do bloco principal por ≈47 px e tem três grupos de quatro teclas com ≈32 px entre grupos.
- A linha inferior do modelo não segue as larguras padrão (1,25/6,25/1): use os valores de `f75art.ts`.
- Os valores exatos por tecla estão em `app/src/lib/f75art.ts`, indexados por `matrix` (ver `KEYS` em `layout.rs` para nome ↔ matrix).

Cores amostradas do modelo branco/azul/verde:

|Parte|Cor|
|---|---|
|Carcaça|`#d6d4d0`|
|Tampa clara|`#acb3b3`|
|Tampa azul (modificadores, F5–F8)|`#4e5482`|
|Tampa verde (Esc, Enter, Space, setas)|`#3f5a41`|
|Fundo entre as teclas|`#111111`|
|Knob|`#3a393e`|

Dimensões físicas do F75 (≈322 x 143 mm) não foram conferidas em fonte primária: confirme antes de usar em escala real.

## Defeitos do modelo de terceiros (motivos para modelar o próprio)

- Legendas com erros do autor: "Cltr", "PgOn", "~Z", "F8```", "F12`".
- Só existem duas combinações de cor; as outras cores do F75 não têm modelo com licença aberta.
- Linha inferior com larguras fora do padrão.
- Exige crédito CC BY no `README.md`.

## Como a renderização antiga foi gerada (origem de `CAPS`)

Reproduzível sem conta no Sketchfab (o download do arquivo 3D exige login; o visualizador não).

1. Conferir licença: `https://api.sketchfab.com/v3/models/<uid>` (`license.slug`, `isDownloadable`).
2. Abrir um Chromium com WebGL, carregar `https://static.sketchfab.com/api/sketchfab-viewer-1.12.1.js` e iniciar o visualizador num `<iframe>` com `transparent: 1` e a interface desligada.
3. O modelo vem rotacionado. `api.getNodeMap` devolve o nó `Cube.010_0` (instanceID 4); `api.setMatrix(4, diag(sx, sy, sz))` com as escalas do próprio nó remove rotação e translação. As teclas passam a apontar para −Y.
4. `api.setFov(2.7)`, `api.setCameraLookAt([0, -150, 0], [0, 0, 0], 0)`, esperar ≈2,5 s, `api.getScreenShot(3200, 1800, 'image/png', cb)`.
5. Recortar pela caixa do canal alfa, redimensionar para 1760 px de largura, salvar WebP (qualidade 86).
6. Medir as tampas na imagem recortada: máscara de pixels escuros (luminância < 45); linhas com média > 0,5 são os vãos entre fileiras; dentro de cada fileira, colunas com fração escura < 0,6 são tampas. A linha F fica sobre carcaça clara e não separa assim: medir o início de cada grupo e gerar o resto pelo passo.
7. Conferir a contagem por fileira (13, 15, 15, 14, 14, 9) antes de gravar `f75art.ts`.

## Modelo próprio: `f75.sc`

Modelo paramétrico feito no SolidCraft (0.45.0), sem geometria nem asset de terceiros. Abre com o app ou com `solidcraft-cli`.

- Escala: 1u = 103 px = 19,05 mm. Origem no centro; `x = (px - 880) * k`, `y = (383 - py) * k`, com `k = 19,05 / 103`. As tampas saem direto de `CAPS`.
- Referencial: o plano das teclas é horizontal, para a vista de topo ficar perpendicular às tampas. A cunha da carcaça está na base, que desce 8 mm atrás (frente 16 mm, traseira 24 mm).
- Corpos: `CaseBottom`, `CaseTop`, `Plate` (placa e carcaças dos switches), `Indicator`, `Knob`, `KnobRing`, 15 grupos de tampas `Caps<linha><papel>` e 15 de legendas `Legend<linha><papel>` (linha `R1`–`R6`, papel `Alpha`, `Mod` ou `Accent`). Cada grupo é um corpo só com várias peças soltas: as teclas não são corpos individuais.
- Parâmetros: `cap_base` (altura da base das tampas; a borda da carcaça fica em 16 mm), `cap_h` (altura bruta da tampa antes do corte de perfil) e `cap_taper` (ângulo das quatro paredes, 13°). Tampas, switches e legendas seguem `cap_base`.
- Pegada das tampas: cada tampa tem 96 px (17,76 mm) de profundidade centrada na fileira, então as de 1u são quadradas. Isso difere de `CAPS`, que mede só a parte visível na renderização em perspectiva.
- Perfil por fileira: `CapTopCutR1`–`R6` cortam o topo de cada fileira num plano inclinado (altura atrás/à frente sobre `cap_base`, em mm: R1 10,5/8,5; R2 10,0/8,3; R3 8,8/7,8; R4 7,8/7,6; R5 7,8/8,8; R6 7,8/9,0), e os sketches de legenda ficam nesses planos.
- Formato das tampas: paredes a `cap_taper` nos quatro lados (topo quadrado nas de 1u), `CornerFillet*` arredonda as quinas verticais (R1,2) e `TopFillet*` a borda do topo (R0,6). As legendas laterais seguem a face frontal.
- Proporções e detalhes tirados de fotos de um F75 Black físico (topo, lateral e traseira): tampas 8 mm acima da borda, legendas no canto superior esquerdo, luz de status entre Esc e F1, knob com anel metálico num rebaixo e painel traseiro, do lado do Esc, com Type-C, chave BT/Wired/2.4G e USB-A.
- Legendas: Arial Rounded MT Bold (fonte instalada no Windows, referida pelo nome), símbolos ⇧ ↵ ← ⇤ ⇥ e setas em Segoe UI Symbol. Letras com 3,4 mm de altura de maiúscula, números e símbolos em duas linhas de 3,2 mm, F1–F12 2,1 mm, modificadores 2,2 mm com ícones de 3,8 mm, todos no canto superior esquerdo do topo; setas centradas com 4,4 mm. Em outra máquina sem essas fontes o sketch não reconstrói.
- Cores: uma linha da tabela de configurações por tema. Da lista "Color Theme" da Epomaker x AULA F75: Black, Black (Side-Printed), Light Blue, Green, Pink (Side-Printed), Purple, Blue (Side-Printed), Gradient Pink (Side-Printed) e Sea Salt Blue. Só na página oficial (`aulastar.com/mechanical-keyboard/176.html`), sem nome publicado, com nomes descritivos aqui: Cream Pink, Black (Shine-Through), White (Shine-Through), Black Topographic (Side-Printed) e White Topographic (Side-Printed).
- Trocar de tema: MANAGE › CONFIGURATIONS › Configurations e o botão Activate da linha, ou `config.activate {"row": "<tema>"}`.
- Temas Side-Printed suprimem as legendas de topo e constroem `LegendSideR1`–`R6` na face frontal das tampas; a primeira troca entre os dois tipos reconstrói as legendas (≈9 s).
- Os temas Topographic têm só as cores de base: a estampa de curvas de nível não foi modelada.
- As cores de tampa, carcaça e knob foram amostradas das fotos de produto (consultadas, não copiadas) e corrigidas a olho onde a amostra caiu em sombra ou no brilho RGB; as cores de legenda são a olho.
- O viewport não é fiel às cores: a vista de topo estoura os tons claros e as legendas pequenas saem pretas por causa das arestas. Confira as cores com `appearance.list`.
- Base: dois pés dobráveis fechados (`Feet`, em rebaixos de 1,5 mm perto da traseira) e quatro almofadas de borracha ovais (`RubberPads`: duas na frente, duas nos cantos traseiros). Os pés seguem a cor de `CaseBottom` em cada tema; as almofadas são pretas.
- Painel traseiro: entrada Type-C metálica com língua, USB-A com língua, chave BT/Wired/2.4G com seletor (`PortMetal`, `PortInner`) e textos impressos (`PortLabels`). O seletor é fixo em 2.4G.
- Medidas verticais, ângulos de perfil e raios são estimados pelas fotos, não medidos com paquímetro. Os topos das tampas são planos, sem a concavidade real, e os pés estão só na posição fechada.

## Verificação

- `cd app && npm run check` e `npx vite build`.
- Prévia com backend simulado (ver `AGENTS.md`, "Prévia da UI sem teclado"; Chromium, o WebKit headless não tem WebGL): na aba Por tecla, pintar deve colorir a tampa clicada; na aba Teclas, o anel de seleção deve contornar a tampa clicada, também depois de girar a vista. Conferir em 1100x720 e 900x600.
