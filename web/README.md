# WortMeister 网页版（PWA）使用与开发说明

这是 WortMeister 德语单词训练器的网页版（PWA，渐进式网页应用）。它复用 Rust
核心逻辑（编译成 WebAssembly），可在电脑和手机浏览器里运行，并能"添加到主屏幕"
像原生 App 一样离线使用。免费、无需 Apple 账号、无需 Mac。

---

## 功能

- **学习卡片**：正面德语单词，可朗读；点"显示答案"看中文、例句与例句翻译；再用
  Again / Hard / Good / Easy 四个按钮按 SM-2 间隔重复算法打分。
- **听写模式**：自动朗读德语单词，你输入拼写，点"检查"后由 Rust 核心做
  **容错比对**（忽略大小写、标点、空格，并把德语变音折叠，如 `Fuesse` = `Füße`），
  显示"正确/不对 + 正确答案"，再打分。
- **学习统计**：今日复习数、连续学习天数、今日到期、已掌握、已学习、未学习、
  正确率、词库总数，以及掌握进度条；可一键清除所有进度。
- **离线可用**：首次联网加载后，Service Worker 缓存全部资源，断网也能用。
- **进度本地保存**：学习进度存在浏览器 localStorage，不上传任何服务器。
- **深色模式**：跟随系统自动切换；界面适配 iPhone 刘海与底部安全区。

---

## 目录结构

```
web/
├─ wasm/                 Rust -> WebAssembly 的壳 crate（依赖 ../core）
│  ├─ Cargo.toml
│  └─ src/lib.rs         导出 parse/levels/make_queue/grade/check/due/stats
├─ public/               静态网站（部署到 GitHub Pages 的内容）
│  ├─ index.html         首页 / 复习页 / 统计页三屏结构
│  ├─ styles.css         移动端样式
│  ├─ main.js            加载 WASM、驱动 UI、朗读、localStorage 存取
│  ├─ manifest.webmanifest  PWA 清单
│  ├─ sw.js              Service Worker（离线缓存）
│  ├─ icons/             App 图标 192 / 512 / 180
│  ├─ pkg/               wasm-pack 产物（构建生成，默认 .gitignore）
│  └─ vocabulary.csv     构建时从 ../data 拷贝进来
├─ build_and_serve.bat   Windows 一键构建 + 本地预览
└─ README.md             本文件
```

---

## 本地运行（Windows）

前置：安装 Rust（rustup）、`wasm-pack`（`cargo install wasm-pack`）、Python 3。

```bat
cd WortMeister\web
build_and_serve.bat
```

浏览器打开 http://localhost:8777 。

> 必须用 http:// 访问，不要直接双击 index.html。file:// 下浏览器会拦截 WASM
> 模块加载与 Service Worker 注册。

手动分步（任意平台）：

```bash
# 1) 构建 WASM 到 public/pkg
wasm-pack build web/wasm --target web --out-dir ../public/pkg --release
# 2) 拷贝词库
cp data/vocabulary.csv web/public/vocabulary.csv
# 3) 本地服务
cd web/public && python -m http.server 8777
```

---

## 免费部署到 GitHub Pages

1. 仓库推到 GitHub（分支 main / master）。
2. 仓库 Settings -> Pages -> Build and deployment -> Source 选 **GitHub Actions**。
3. 推送后 `.github/workflows/pages.yml` 自动构建并发布，完成后给出网址：
   `https://<用户名>.github.io/<仓库名>/`

所有资源路径均为相对路径，部署在子路径下也能正常工作。

---

## 装到 iPhone

1. 用 **Safari**（必须 Safari）打开上面的 Pages 网址。
2. 底部"分享" -> **添加到主屏幕** -> 添加。
3. 主屏幕出现图标，点开即全屏 App，首次联网后可离线使用。

---

## 常见问题

- **听写不检查拼写？** 已修复。此前"检查"后界面会立刻重绘并清空输入，导致结果一闪
  而过。现在检查结果保存在状态里，会稳定显示"正确/不对 + 正确答案"，输入框在检查
  后锁定，进入下一张卡时才清空。
- **没有学习统计？** 已新增统计页（首页"学习统计"按钮进入），数据由 Rust 核心的
  `stats()` 计算，三端口径一致。
- **朗读没声音？** iOS 首次需用户交互后才允许播放；点一次"朗读"按钮即可。系统需装有
  德语语音；没有时会用默认语音发音。
- **换了词库或界面后手机还是旧的？** 修改静态资源后，把 `public/sw.js` 里的
  `CACHE_VERSION`（如 `wortmeister-v1` -> `v2`）加一，旧缓存会被刷新。

---

## 与其他版本的关系

- 桌面版（egui，仓库根 `src/`）、iOS 原生方案（`ios/`，见 `IOS_PLAN.md`）与本网页版
  共用同一个 `core/` 核心库，SM-2 算法、CSV 解析、听写判题、统计只有一份实现。
- 整体方案与取舍见仓库根的 `PWA_PLAN.md`。
