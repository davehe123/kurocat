# 团子桌宠

团子从 KuroCat 4.7.0 的 Tauri / Rust / Canvas 桌宠引擎出发。双击 Windows 版会打开独立控制面板并显示烟灰虎斑桌宠。关闭面板后，可从托盘重新打开，右键桌宠也有同一入口。

## 控制面板

- 切换角色，预览敲键和点击动画，调整大小和键盘配列。
- 设置置顶、呼噜、按键气泡和密码输入保护。
- 自定义毛色、瞳色和按键强调色。生成的角色可在控制面板、桌宠和 MCP 中继续使用。
- 通过「显示桌宠」和「移回桌面右下角」恢复可见位置。

## Windows 免安装构建

在仓库根目录运行。工具链使用 Node.js、Rust、Microsoft C++ Build Tools 和 WebView2。

```powershell
npm ci
npm run build:windows:portable
```

生成文件位于 `src-tauri/target/release/TuanziCat.exe`。用资源管理器双击打开。Windows 需有 WebView2 Runtime。

配置目录为 `%APPDATA%\tuanzi-cat`，本地 MCP 默认端口为 `27421`。保留原有 MCP 协议和键鼠动画引擎。内置版本检查指向这个 Fork 的 Releases。

上游来源 [qianyiai/kurocat](https://github.com/qianyiai/kurocat)。许可证见 [LICENSE](LICENSE)。
