<div align="center">

<img src="app/app-icon.svg" width="96" height="96" alt="">

# OpenF75

**AULA F75 キーボード用のオープンソース設定ツール。**<br>
ライティング、キーごとの色、キー割り当て、マクロ、プロファイルを、メーカーのソフトウェアなしで扱えます。

[![ダウンロード](https://img.shields.io/badge/%E3%83%80%E3%82%A6%E3%83%B3%E3%83%AD%E3%83%BC%E3%83%89-releases-e8e6df)](https://github.com/pendiego/openf75/releases/latest)
![Windows](https://img.shields.io/badge/Windows-%E3%83%86%E3%82%B9%E3%83%88%E6%B8%88%E3%81%BF-2ea44f?logo=windows&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-%E5%AE%9F%E9%A8%93%E7%9A%84-f0ad4e?logo=linux&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-%E5%AE%9F%E9%A8%93%E7%9A%84-f0ad4e?logo=apple&logoColor=white)
[![MITライセンス](https://img.shields.io/badge/%E3%83%A9%E3%82%A4%E3%82%BB%E3%83%B3%E3%82%B9-MIT-blue)](#ライセンス)

[**ダウンロード**](https://github.com/pendiego/openf75/releases/latest) · [機能](#機能) · [インストール](#インストール) · [CLI](#コマンドライン) · [制限事項](#制限事項)

[English](README.md) · [Português](README.pt-BR.md) · [Español](README.es.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · **日本語** · [简体中文](README.zh-CN.md)

<br>

<img src="docs/screenshots/iluminacao.png" alt="ライティングタブ：Rainbow エフェクトを表示した F75 の 3D モデルと、ライブプレビュー付きのエフェクト一覧" width="860">

</div>

## 機能

| 機能 | できること |
|---|---|
| **ライティング** | キーボードの 20 種類のエフェクトを 3D モデル上でライブプレビュー。明るさ、速度、色を調整 |
| **キーごと** | キーを一つずつ、または範囲で塗り分け、キーボードに書き込み |
| **オーディオ連動** | キーがパソコンで再生中の音のイコライザーになります |
| **キー割り当て** | Base と Fn のレイヤーを再割り当て：キー、メディア、マクロ、または無効化 |
| **マクロ** | イベントごとの遅延付きで操作を記録し、キーボードのメモリに直接保存 |
| **電源** | スリープ時間、デバウンス、ホイールの機能（音量またはライティング） |
| **プロファイル** | 設定の保存、適用、インポート、エクスポート。初回接続時に自動でバックアップ |
| **コマンドパレット** | <kbd>Ctrl</kbd> <kbd>K</kbd>（macOS では <kbd>⌘</kbd> <kbd>K</kbd>）で任意のタブや操作へ移動 |

ライトテーマとダークテーマ、自動アップデートに対応しています。

<table>
  <tr>
    <td><img src="docs/screenshots/teclas.png" alt="キータブ：Base レイヤーの再割り当て"></td>
    <td><img src="docs/screenshots/macros.png" alt="マクロタブ：キー、動作、遅延を設定するイベントエディター"></td>
  </tr>
  <tr>
    <td align="center"><sub>レイヤーの再割り当て</sub></td>
    <td align="center"><sub>マクロエディター</sub></td>
  </tr>
</table>

### 言語（開発中）

アプリの UI は、次のリリースで ポルトガル語（ブラジル）、英語、スペイン語、フランス語、ドイツ語、日本語、簡体字中国語に対応する予定です。初回起動時はシステムの言語に従い、その言語がない場合は英語になります。切り替えはウィンドウ下部のステータスバー（ファームウェアとバージョンの横）にある言語セレクターか、コマンドパレットで言語を検索して行えます。選択内容は保存されます。

CLI と Rust のエラーメッセージはポルトガル語のままです。アプリは既知のエラーを選択した言語で表示します。

### 対応状況

| | USB ケーブル `258A:010C` | 2.4 GHz レシーバー `3554:FA09` | Bluetooth |
|---|:---:|:---:|:---:|
| AULA F75 | ✅ | ✅ | ❌ |
| AULA F75 KR | ❌ | ❌ | ❌ |

| システム | パッケージ | 状態 |
|---|---|---|
| Windows 10/11 | NSIS インストーラー | キーボードで動作確認済み |
| Linux (x64) | `.deb` と AppImage | CI での検証は保留中、実機テストなし |
| macOS 13+（Intel および Apple Silicon） | `.dmg` | CI での検証は保留中、実機テストなし |

## インストール

お使いのシステム向けのパッケージを [リリースページ](https://github.com/pendiego/openf75/releases/latest) からダウンロードしてください。

> [!IMPORTANT]
> Windows では、書き込みの前に **AULA のソフトウェア（`OemDrv.exe`）を終了してください**。両方が同じキーボードのインターフェースを使うため競合します。OpenF75 は AULA のソフトウェアが開いていると警告します。

<details>
<summary><b>Windows</b></summary>

インストーラーを実行してください。ドライバーや追加のルールは不要です。アプリは自動で更新されます。

</details>

<details>
<summary><b>Linux</b></summary>

- **`.deb`**：ログイン中のユーザーがキーボードを使えるようにする udev ルールをインストールします。更新する場合は新しい `.deb` をインストールしてください。
- **AppImage**：自動で更新されますが、udev ルールは含まれません。一度だけ次のコマンドでインストールしてください：

  ```sh
  sudo cp packaging/linux/70-openf75.rules /etc/udev/rules.d/
  sudo udevadm control --reload && sudo udevadm trigger
  ```

このルールは systemd-logind に依存する `TAG+="uaccess"` を使います。キーボードを開けない場合、エラーメッセージがこのルールを指します。

オーディオ連動は既定の出力のモニター（PulseAudio または PipeWire）を取得します。

</details>

<details>
<summary><b>macOS</b></summary>

アプリは ad-hoc 署名されており、公証はされていません（Apple の証明書がないため）。初回はアプリを右クリックして **開く** を選んでください。それでも macOS にブロックされる場合は次を実行します：

```sh
xattr -dr com.apple.quarantine /Applications/OpenF75.app
```

macOS ではアプリがシステムの音声を取得できません。オーディオ連動には [BlackHole](https://github.com/ExistentialAudio/BlackHole) のようなループバックデバイスをインストールし、出力をそこへ送ってください。macOS がマイクの許可を求めてきますが、アプリはその経路でループバックを読み取ります。

</details>

マクロ、プロファイル、バックアップの保存先：

| Windows | macOS | Linux |
|---|---|---|
| `%APPDATA%\openf75\` | `~/Library/Application Support/openf75/` | `$XDG_CONFIG_HOME/openf75/` または `~/.config/openf75/` |

## コマンドライン

CLI `openf75` は、アプリと同じことをスクリプトから行えます。安定版の Rust が必要です。Linux では事前に `libudev-dev`、`libasound2-dev`、`pkg-config` をインストールしてください。

```sh
cargo install --path crates/openf75-cli   # openf75 コマンドを PATH にインストール

openf75 status                                   # 読み取り専用
openf75 dump --out backup.json                   # 読み取り専用：書き込みの前に実行
openf75 effect 2 --brightness 5 --speed 3 --rgb 00FF00
openf75 perkey --all 000000 W=FF0000 A=FF0000
openf75 remap base CapsLock key:0x29
openf75 macro-text F12 "ola"
openf75 sleep 300
openf75 debounce 1
openf75 restore backup.json
```

## 開発

```sh
cd app
npm install
npm run tauri dev            # 開発モードでアプリを起動
npm run check                # svelte-check
cargo test -p openf75        # プロトコルのテスト
```

インストーラーのビルド：Windows では `npm run tauri build`、Linux では `-- --bundles deb,appimage` を追加、macOS では `-- --target universal-apple-darwin --bundles app,dmg` を追加します。

| パス | 内容 |
|---|---|
| [`crates/openf75`](crates/openf75) | Rust ライブラリ：HID プロトコル、デバイス、オーディオ、ファイル |
| [`crates/openf75-cli`](crates/openf75-cli) | CLI `openf75` |
| [`app/`](app) | デスクトップアプリ（Tauri 2 + Svelte 5） |
| [`docs/PROTOCOL.md`](docs/PROTOCOL.md) | リバースエンジニアリングで調べた F75 のプロトコル |

<details>
<summary><b>バージョンを公開する</b></summary>

1. `app/src-tauri/tauri.conf.json`、`app/src-tauri/Cargo.toml`、`app/package.json` の `version` を上げ、`app/package-lock.json`（`app/` で `npm install --package-lock-only`）と `Cargo.lock`（`cargo check -p app`）を更新します。
2. コミットし、タグを作成して push します：`git tag vX.Y.Z` と `git push origin vX.Y.Z`。
3. ワークフロー [`release.yml`](.github/workflows/release.yml) は `windows-latest`、`ubuntu-22.04`、`macos-latest` を一つずつビルドします（`max-parallel: 1`）。`tauri-action` がアップデーターの読む `latest.json` をマージするためです。

アプリは起動時に新しいバージョンを確認し、**更新** ボタンを表示します。更新はアップデーターの署名で検証されます。Linux で自動更新されるのは AppImage だけです。

リポジトリのシークレット（Settings → Secrets and variables → Actions）：
- `TAURI_SIGNING_PRIVATE_KEY`：`%USERPROFILE%\.tauri\openf75.key` の内容。
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`：`%USERPROFILE%\.tauri\openf75.key.password` の内容。

> [!CAUTION]
> 秘密鍵はリポジトリに入れないでください。紛失すると、インストール済みのアプリは更新を受け付けなくなります。その場合は手動で再インストールし、`tauri.conf.json` の `pubkey` を差し替える必要があります。

</details>

## 制限事項

**キーボードと接続**

- **AULA F75 のみ。** F75 KR（別レイアウト）と Bluetooth はサポートされていません。
- **USB ケーブル：** プロトコルはリバースエンジニアリングで実装され、ユニットテストしかありません。キーボードでの動作確認はまだです。
- **2.4 GHz レシーバー：**
  - カラーパレット（`0x49`）が応答しません。表示される色は既定の色で、バックアップにパレットは含まれず、復元時にはスキップされ、カスタムカラーは拒否されます（スロット 0-6 またはケーブルを使ってください）。
  - オーディオ反応モードとリアルタイムモードは約 3 fps で動作します。
  - キーレイヤーは 512 バイト中 504 バイトで届きます。残りはゼロで埋められます。
  - 最初の応答に約 400 ms かかることがあります。
  - キーボードがスリープ中または電源オフのときは書き込みが失敗します。キーを押して起こしてください。
- **スリープ時間：** ワイヤレスモードでのみ有効です。
- **ホイール：** 音量またはライティングを選ぶバイトはレシーバー経由で見つかったもので、ケーブルでは確認されていません。
- **ファームウェア：** OpenF75 はファームウェアを更新しません。AULA のドライバーも書き込みは行わず、現在の F75 パッケージに含まれない外部のアップデーターを呼び出します。
- **マクロ：** 名前を含めて合計 4096 バイト。名前は 30 文字まで。イベントごとの遅延は 1,048,575 ms（約 17 分）まで。

**システム**

- **Windows：** 書き込み前に AULA のソフトウェア（`OemDrv.exe`）を終了してください。提供されるのは x64 の NSIS インストーラーのみです。
- **Linux と macOS はキーボードでテストされておらず、パッケージもまだ CI を通過していません。** これらのシステムで何かを書き込む前に、`openf75 status` と `openf75 dump` を実行してください。
- **Linux：** x64 のみ。キーボードへのアクセスは udev ルールに依存し、そのルールは `TAG+="uaccess"` を使い systemd-logind が必要です。AppImage はルールをインストールしません。自己更新するのは AppImage だけで、`.deb` は手動で更新します。オーディオ反応には PulseAudio または PipeWire が必要です。
- **macOS：** バージョン 13 以降のみ。アプリは公証されていないため、初回起動時に Gatekeeper がブロックします。システムはサウンドのキャプチャを許可しないため、オーディオ反応にはループバックデバイス（BlackHole など）とマイクの許可が必要です。

**言語**

- インターフェースの翻訳はまだ開発中です。CLI と Rust から出るエラーメッセージはポルトガル語で、アプリが知っているエラーだけを翻訳します。

## ライセンス

[MIT](LICENSE)。AULA とは提携しない独立したプロジェクトです。メーカーのファイルはリポジトリに含まれていません。
