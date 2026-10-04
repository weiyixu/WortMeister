# WortMeister PWA 版（免费、自用、装到 iPhone）

你不需要上架 App Store，只是自己用。最省钱省事的方式是 **PWA（渐进式网页应用）**：
做成一个网页，用 iPhone 的 Safari 打开后"添加到主屏幕"，就会出现一个全屏 App
图标，可离线使用、本地保存进度。

- **完全免费**：不需要 Apple 开发者账号，不需要 Mac。
- **不会过期**：不像免费签名侧载那样 7 天失效。
- **逻辑复用**：你的 Rust 核心逻辑编译成 WebAssembly 直接用，SM-2 算法 / CSV
  解析 / 听写判题一行都不用重写。

---

## 1. 架构

```
          +-------------------------------------+
          |  iPhone Safari / 主屏幕图标 (PWA)    |
          |  HTML + CSS + JS  (web/public/*)     |
          |  德语朗读 = 浏览器 Web Speech API     |
          |  进度保存 = localStorage             |
          +------------------+------------------+
                             | 调用 (JSON 字符串)
          +------------------v------------------+
          |  WebAssembly (web/wasm -> pkg/*.wasm) |
          |  由 Rust 核心 wortmeister_core 编译    |
          |  CSV 解析 / SM-2 / 听写判题 / 到期统计 |
          +-------------------------------------+
```

一份 Rust 逻辑，桌面、iOS 原生、网页三端共用。

---

## 2. 本次生成的内容

```
WortMeister/
├─ core/                     # 之前已建：共享 Rust 核心（本次 PWA 复用它）
├─ web/
│  ├─ wasm/                  # 新增：把 core 编译成 WebAssembly 的壳 crate
│  │  ├─ Cargo.toml          #   依赖 core + wasm-bindgen + getrandom(js)
│  │  └─ src/lib.rs          #   导出 parse/queue/grade/check/levels/due 等
│  ├─ public/               # 新增：静态网站（部署到 GitHub Pages 的内容）
│  │  ├─ index.html          #   页面结构（首页 + 复习页）
│  │  ├─ styles.css          #   移动端样式，适配 iPhone 刘海安全区，暗色模式
│  │  ├─ main.js             #   加载 WASM、驱动 UI、朗读、localStorage 存进度
│  │  ├─ manifest.webmanifest#   PWA 清单（名字、图标、全屏）
│  │  ├─ sw.js               #   Service Worker，离线缓存
│  │  ├─ icons/              #   icon-192 / 512 / 180（本次已生成）
│  │  └─ pkg/                #   wasm-pack 产物（本机已构建，CI 会重建）
│  ├─ build_and_serve.bat    # 新增：Windows 一键构建 + 本地预览
│  └─ .gitignore
└─ .github/workflows/pages.yml  # 新增：推送后自动构建并部署到 GitHub Pages
```

> 本机已验证：`wasm-pack` 构建成功，本地 HTTP 服务下所有资源 200 可访问。

---

## 3. 本地预览（Windows）

```bat
cd WortMeister\web
build_and_serve.bat
```

然后浏览器打开 http://localhost:8777 。
（需要已安装 Rust、`wasm-pack`、Python 3。`wasm-pack` 安装：`cargo install wasm-pack`。）

> 必须通过 http:// 访问，不能直接双击 index.html（file:// 下浏览器会拦截 WASM
> 模块和 Service Worker）。

---

## 4. 免费部署到 GitHub Pages

1. 把整个仓库推送到 GitHub（分支 main 或 master）。
2. 仓库 Settings -> Pages -> Build and deployment -> Source 选 **GitHub Actions**。
3. 推送后 `.github/workflows/pages.yml` 会自动：装 Rust + wasm-pack、构建 WASM、
   拷贝词库、把 `web/public` 发布到 Pages。
4. Actions 跑完后，Pages 会给出网址，形如：
   `https://<你的用户名>.github.io/<仓库名>/`

> 注意：由于项目可能部署在子路径下（.../<仓库名>/），代码里所有资源路径都用的是
> **相对路径**，Service Worker 也按相对 scope 注册，子路径下可正常工作。

---

## 5. 装到 iPhone（关键一步）

1. iPhone 用 **Safari**（必须 Safari，不能用微信/Chrome 内置浏览器）打开上面的
   Pages 网址。
2. 点底部"分享"图标 -> **添加到主屏幕** -> 添加。
3. 主屏幕上会出现 WortMeister 图标，点开就是全屏 App，首次联网加载后即可离线使用。

学习进度保存在手机本地（localStorage），不会上传任何服务器。

---

## 6. 功能对照（桌面版 -> PWA）

| 桌面版 | PWA 版 |
|--------|--------|
| egui 窗口 | HTML/CSS 移动端界面 |
| 学习卡片 / 听写 | 同样有，触屏操作 |
| SM-2 间隔重复 | 同一份 Rust 逻辑（WASM） |
| 听写容错（变音折叠） | 同一份 `norm()` 逻辑 |
| `tts` crate 朗读 | 浏览器 Web Speech API（德语 de-DE） |
| progress.json 文件 | localStorage |
| vocabulary.csv 读文件 | fetch 读取打包进站点的 CSV |

---

## 7. 更新词库 / 发布新版本

- 改 `data/vocabulary.csv` 后推送，CI 会自动重新部署。
- 若改了前端静态资源，建议把 `web/public/sw.js` 里的 `CACHE_VERSION`
  （如 `wortmeister-v1` -> `v2`）加一，这样 iPhone 上的旧缓存会被刷新。

---

## 8. 和 iOS 原生方案（IOS_PLAN.md）的关系

- `IOS_PLAN.md` 描述的是"Rust 核心 + SwiftUI 原生 App"方案，适合要上架或要最佳
  原生体验、且有 Mac + Apple 账号的情况。
- 本文件（PWA）适合"免费、自用、Windows 开发"的情况，是你当前的最佳选择。
- 两套共用同一个 `core/`，将来想做原生版时逻辑不用重写。
