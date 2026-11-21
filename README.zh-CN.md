<div align="center">

<img src="app/app-icon.svg" width="96" height="96" alt="">

# OpenF75

**AULA F75 键盘的开源配置工具。**<br>
灯效、逐键颜色、按键重映射、宏和配置方案，无需厂商软件。

[![下载](https://img.shields.io/badge/%E4%B8%8B%E8%BD%BD-releases-e8e6df)](https://github.com/pendiego/openf75/releases/latest)
![Windows](https://img.shields.io/badge/Windows-%E5%B7%B2%E6%B5%8B%E8%AF%95-2ea44f?logo=windows&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-%E5%AE%9E%E9%AA%8C%E6%80%A7-f0ad4e?logo=linux&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-%E5%AE%9E%E9%AA%8C%E6%80%A7-f0ad4e?logo=apple&logoColor=white)
[![MIT 许可证](https://img.shields.io/badge/%E8%AE%B8%E5%8F%AF%E8%AF%81-MIT-blue)](#许可证)

[**下载**](https://github.com/pendiego/openf75/releases/latest) · [功能](#功能) · [安装](#安装) · [CLI](#命令行) · [限制](#限制)

[English](README.md) · [Português](README.pt-BR.md) · [Español](README.es.md) · [Français](README.fr.md) · [Deutsch](README.de.md) · [日本語](README.ja.md) · **简体中文**

<br>

<img src="docs/screenshots/iluminacao.png" alt="灯效标签页：显示 Rainbow 效果的 F75 3D 模型，以及带实时预览的效果库" width="860">

</div>

## 功能

| 功能 | 作用 |
|---|---|
| **灯效** | 键盘的 20 种效果，在 3D 模型上实时预览，可调亮度、速度和颜色 |
| **逐键** | 逐个或按区域为按键上色，并写入键盘 |
| **音频响应** | 按键变成电脑正在播放的声音的均衡器 |
| **按键** | 重映射 Base 和 Fn 层：按键、多媒体、宏或禁用 |
| **宏** | 录制带每个事件延迟的按键序列，直接保存到键盘内存 |
| **电源** | 休眠时间、消抖以及滚轮功能（音量或灯效） |
| **配置方案** | 保存、应用、导入和导出配置；首次连接时自动备份 |
| **命令面板** | <kbd>Ctrl</kbd> <kbd>K</kbd>（macOS 上为 <kbd>⌘</kbd> <kbd>K</kbd>）跳转到任意标签页或操作 |

支持浅色和深色主题，并可自动更新。

<table>
  <tr>
    <td><img src="docs/screenshots/teclas.png" alt="按键标签页：Base 层的重映射"></td>
    <td><img src="docs/screenshots/macros.png" alt="宏标签页：带按键、动作和延迟的事件编辑器"></td>
  </tr>
  <tr>
    <td align="center"><sub>层重映射</sub></td>
    <td align="center"><sub>宏编辑器</sub></td>
  </tr>
</table>

### 语言（开发中）

应用界面将在下一个版本中提供葡萄牙语（巴西）、英语、西班牙语、法语、德语、日语和简体中文。首次启动时，应用会跟随系统语言；如果系统语言不在列表中，则使用英语。切换语言可使用窗口底部状态栏（与固件和版本号并列）中的选择器，或在命令面板中搜索语言；所选语言会被保存。

CLI 和 Rust 错误信息仍将使用葡萄牙语；应用会以所选语言显示已知错误。

### 兼容性

| | USB 线 `258A:010C` | 2.4 GHz 接收器 `3554:FA09` | 蓝牙 |
|---|:---:|:---:|:---:|
| AULA F75 | ✅ | ✅ | ❌ |
| AULA F75 KR | ❌ | ❌ | ❌ |

| 系统 | 安装包 | 状态 |
|---|---|---|
| Windows 10/11 | NSIS 安装程序 | 已用该键盘测试 |
| Linux (x64) | `.deb` 和 AppImage | CI 验证待完成，未做硬件测试 |
| macOS 13+（Intel 和 Apple Silicon） | `.dmg` | CI 验证待完成，未做硬件测试 |

## 安装

从 [发布页面](https://github.com/pendiego/openf75/releases/latest) 下载适合你系统的安装包。

> [!IMPORTANT]
> 在 Windows 上，写入前请**关闭 AULA 的软件（`OemDrv.exe`）**：两者会争用同一个键盘接口。OpenF75 检测到它打开时会发出提示。

<details>
<summary><b>Windows</b></summary>

运行安装程序即可。无需驱动或额外规则。应用会自动更新。

</details>

<details>
<summary><b>Linux</b></summary>

- **`.deb`**：安装 udev 规则，使键盘对当前登录的用户开放。更新时请安装新的 `.deb`。
- **AppImage**：会自动更新，但 udev 规则不随附。请安装一次：

  ```sh
  sudo cp packaging/linux/70-openf75.rules /etc/udev/rules.d/
  sudo udevadm control --reload && sudo udevadm trigger
  ```

该规则使用依赖 systemd-logind 的 `TAG+="uaccess"`。如果应用无法打开键盘，错误信息会指向此规则。

音频响应会捕获默认输出的监视器（PulseAudio 或 PipeWire）。

</details>

<details>
<summary><b>macOS</b></summary>

应用采用 ad-hoc 签名，未经过公证（没有 Apple 证书）。第一次请右键点击应用并选择 **打开**。如果 macOS 仍然阻止，请运行：

```sh
xattr -dr com.apple.quarantine /Applications/OpenF75.app
```

macOS 不允许应用捕获系统声音。要使用音频响应，请安装 [BlackHole](https://github.com/ExistentialAudio/BlackHole) 之类的回环设备，并将输出发送到该设备。macOS 会请求麦克风权限：应用正是通过它读取回环音频的。

</details>

宏、配置方案和备份保存在：

| Windows | macOS | Linux |
|---|---|---|
| `%APPDATA%\openf75\` | `~/Library/Application Support/openf75/` | `$XDG_CONFIG_HOME/openf75/` 或 `~/.config/openf75/` |

## 命令行

`openf75` 命令行工具与应用功能相同，便于脚本调用。需要稳定版 Rust；在 Linux 上请先安装 `libudev-dev`、`libasound2-dev` 和 `pkg-config`。

```sh
cargo install --path crates/openf75-cli   # 将 openf75 命令安装到 PATH

openf75 status                                   # 只读
openf75 dump --out backup.json                   # 只读：写入前执行
openf75 effect 2 --brightness 5 --speed 3 --rgb 00FF00
openf75 perkey --all 000000 W=FF0000 A=FF0000
openf75 remap base CapsLock key:0x29
openf75 macro-text F12 "ola"
openf75 sleep 300
openf75 debounce 1
openf75 restore backup.json
```

## 开发

```sh
cd app
npm install
npm run tauri dev            # 开发模式运行应用
npm run check                # svelte-check
cargo test -p openf75        # 协议测试
```

构建安装程序：Windows 上使用 `npm run tauri build`；Linux 上追加 `-- --bundles deb,appimage`；macOS 上使用 `-- --target universal-apple-darwin --bundles app,dmg`。

| 路径 | 内容 |
|---|---|
| [`crates/openf75`](crates/openf75) | Rust 库：HID 协议、设备、音频、文件 |
| [`crates/openf75-cli`](crates/openf75-cli) | `openf75` 命令行 |
| [`app/`](app) | 桌面应用（Tauri 2 + Svelte 5） |
| [`docs/PROTOCOL.md`](docs/PROTOCOL.md) | 通过逆向工程获得的 F75 协议 |

<details>
<summary><b>发布版本</b></summary>

1. 提高 `app/src-tauri/tauri.conf.json`、`app/src-tauri/Cargo.toml` 和 `app/package.json` 中的 `version`；更新 `app/package-lock.json`（在 `app/` 中运行 `npm install --package-lock-only`）和 `Cargo.lock`（运行 `cargo check -p app`）。
2. 提交、创建标签并推送：`git tag vX.Y.Z` 和 `git push origin vX.Y.Z`。
3. [`release.yml`](.github/workflows/release.yml) 工作流会依次在 `windows-latest`、`ubuntu-22.04` 和 `macos-latest` 上构建（`max-parallel: 1`），因为 `tauri-action` 需要合并更新程序读取的 `latest.json`。

应用启动时会检查新版本，并显示 **更新** 按钮。更新通过更新程序的签名校验。Linux 上只有 AppImage 会自动更新。

仓库机密（Settings → Secrets and variables → Actions）：
- `TAURI_SIGNING_PRIVATE_KEY`：`%USERPROFILE%\.tauri\openf75.key` 的内容。
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`：`%USERPROFILE%\.tauri\openf75.key.password` 的内容。

> [!CAUTION]
> 私钥绝不能进入仓库。一旦丢失，已安装的应用将不再接受更新：届时必须手动重新安装，并替换 `tauri.conf.json` 中的 `pubkey`。

</details>

## 限制

**键盘与连接**

- **仅限 AULA F75。** 不支持 F75 KR（布局不同）和蓝牙。
- **USB 数据线：** 协议基于逆向工程实现，仅有单元测试，从未在该键盘上实际验证。
- **2.4 GHz 接收器：**
  - 调色板（`0x49`）无响应：显示的颜色为默认颜色，备份不包含调色板，恢复时跳过它，自定义颜色会被拒绝（请使用槽位 0-6 或改用数据线）。
  - 音频响应和实时模式约以 3 fps 运行。
  - 按键层以 512 字节中的 504 字节到达，其余部分用零填充。
  - 首次回复可能需要约 400 ms。
  - 键盘休眠或关机时写入会失败；按任意键唤醒。
- **休眠时间：** 仅在无线模式下生效。
- **滚轮：** 用于选择音量或灯光的字节是通过接收器找到的，尚未通过数据线确认。
- **固件：** OpenF75 不更新固件。AULA 驱动同样不写入固件：它调用一个外部更新程序，该程序不包含在当前的 F75 安装包中。
- **宏：** 总计 4096 字节（含名称）；名称最多 30 个字符；每个事件的延迟最多 1,048,575 ms（约 17 分钟）。

**系统**

- **Windows：** 写入前请关闭 AULA 软件（`OemDrv.exe`）。仅提供 x64 NSIS 安装包。
- **Linux 和 macOS 未用该键盘测试，安装包也尚未通过 CI。** 在这些系统上写入任何内容前，请先运行 `openf75 status` 和 `openf75 dump`。
- **Linux：** 仅支持 x64。键盘访问依赖 udev 规则，该规则使用 `TAG+="uaccess"` 并需要 systemd-logind；AppImage 不会安装该规则。只有 AppImage 会自动更新，`.deb` 需手动更新。音频响应需要 PulseAudio 或 PipeWire。
- **macOS：** 仅支持 13 或更高版本。应用未经公证，因此 Gatekeeper 会阻止首次启动。系统不允许捕获其声音：音频响应需要回环设备（BlackHole 等）以及麦克风权限。

**语言**

- 界面翻译仍在开发中。CLI 以及来自 Rust 的错误信息为葡萄牙语；应用仅翻译其已知的错误。

## 许可证

[MIT](LICENSE)。独立项目，与 AULA 无关联。仓库中不包含任何厂商文件。
