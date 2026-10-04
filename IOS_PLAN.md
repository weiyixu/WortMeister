# WortMeister iOS 版方案（Rust 核心 + SwiftUI 界面）

本文件说明如何把现有的桌面版 Rust 程序（egui）变成一个可以装到 iPhone /
上架 App Store 的 iOS App，以及 GitHub Actions 能做到哪一步、哪一步必须要
Apple 开发者账号。

---

## 1. 为什么不直接把 egui 搬到 iOS

现有桌面版用的是 `eframe` / `egui` + `tts` crate：

- `eframe` 没有官方 iOS 支持，要跑在 iOS 上得自己写 winit + Objective-C 启动壳，
  触屏、中文输入法、键盘弹出、安全区都要手动处理，坑多且不稳定。
- `tts` crate 的 iOS 后端支持有限。
- 桌面版把 `vocabulary.csv` / `progress.json` 读写在“exe 旁边”，而 iOS 沙盒
  **不允许**这样，必须改成 bundle 只读资源 + Documents 可写目录。

因此更稳妥、长期可维护的做法是业界常用的 **“Rust 核心 + 原生界面”** 架构：

```
          +-----------------------------+
          |   SwiftUI (iOS 原生界面)     |   <- ios/WortMeister/*.swift
          |   触屏、中文键盘、语音朗读     |
          +--------------+--------------+
                         | C FFI (JSON 字符串)
          +--------------v--------------+
          |   Rust 核心 (wortmeister_core)|  <- core/src/*.rs
          |   CSV 解析 / SM-2 算法 / 判题  |
          +-----------------------------+
```

界面各平台各写一套，但**学习逻辑只有一份 Rust 代码**，桌面和 iOS 完全一致。

---

## 2. 本次已经生成的内容

```
WortMeister/
├─ core/                         # 新增：可复用的 Rust 核心库（无 UI、无文件系统）
│  ├─ Cargo.toml                 #   crate-type = staticlib/cdylib/rlib
│  ├─ include/wortmeister_core.h #   给 Swift 用的 C 头文件
│  └─ src/
│     ├─ lib.rs                  #   模块入口 + 单元测试（已通过 4/4）
│     ├─ model.rs                #   Word / CardProgress / ProgressFile
│     ├─ srs.rs                  #   SM-2 间隔重复算法
│     ├─ session.rs              #   组队列 / 判分 / 取 level
│     ├─ util.rs                 #   日期 / 答案归一化 / CSV 解析（内存版）
│     └─ ffi.rs                  #   C FFI：JSON 进、JSON 出
│
├─ ios/                          # 新增：SwiftUI App
│  ├─ build_rust_xcframework.sh  #   把 Rust 编译成 iOS XCFramework
│  └─ WortMeister/
│     ├─ WortMeisterApp.swift     #   App 入口
│     ├─ RootView.swift           #   主页（筛选 + 开始学习/听写）
│     ├─ ReviewView.swift         #   卡片复习 + 听写界面
│     ├─ Store.swift              #   状态管理 + 调用 Rust 核心 + 存进度
│     ├─ WortMeisterCore.swift    #   Swift 封装的 C FFI（Codable <-> JSON）
│     └─ WortMeister-Bridging-Header.h
│
└─ .github/workflows/ios.yml     # 新增：GitHub Actions（模拟器构建，免签名）
```

> 核心库已在本机 `cargo test` 验证通过（CSV 解析、变音折叠、队列上限、SM-2）。

---

## 3. GitHub Actions 能做什么 / 不能做什么

| 目标 | GitHub Actions 可行？ | 需要 Apple 账号？ |
|------|----------------------|------------------|
| 编译 Rust 核心到 iOS（arm64 + 模拟器） | ✅ 可以 | 否 |
| 编译 SwiftUI App 到 **iOS 模拟器**（验证能跑） | ✅ 可以 | 否 |
| 生成可装真机的 **.ipa** | ✅ 可以 | **是**（$99/年 + 证书） |
| 上传 **TestFlight / App Store** | ✅ 可以 | **是** |

- `ios.yml` 里的 `simulator-build` 任务**现在就能用**：装 Rust iOS target、
  跑 `build_rust_xcframework.sh`、跑核心单测、编译模拟器版、上传 XCFramework。
- 底部注释掉的 `device-ipa` 任务是“签名真机包”的模板，等你有了 Apple 账号和
  签名证书，把它解开并填入 Secrets 即可。

**结论**：GitHub Actions 可以完成从 Rust 到模拟器 App 的全部编译；但“装到你自己
的 iPhone”或“上架”这一步，Apple 规定必须有付费开发者账号和代码签名，这是 CI
绕不过去的硬性要求。

---

## 4. 需要你在一台 Mac 上做的一次性操作

Windows 上无法创建 Xcode 工程，因此以下步骤需要一台 Mac（或 CI 的 macOS runner
+ 一个预先建好的工程文件）：

1. 安装 Xcode 和 Rust（`rustup`）。
2. 新建一个 iOS App 工程，保存为 `ios/WortMeister.xcodeproj`，Interface 选
   SwiftUI。把 `ios/WortMeister/` 下已生成的 6 个 Swift/头文件加入工程（删掉
   Xcode 自动生成的模板 `ContentView.swift`）。
3. 生成 Rust 库：
   ```bash
   cd WortMeister
   chmod +x ios/build_rust_xcframework.sh
   ./ios/build_rust_xcframework.sh      # 产出 ios/WortMeisterCore.xcframework
   ```
4. 在 Xcode 里：
   - 把 `ios/WortMeisterCore.xcframework` 拖进工程（Frameworks, Libraries, and
     Embedded Content）。
   - Build Settings -> **Objective-C Bridging Header** 设为
     `WortMeister/WortMeister-Bridging-Header.h`。
   - Build Settings -> **Header Search Paths** 加入 `../core/include`。
   - 把 `data/vocabulary.csv` 加入工程并勾选 Target membership（作为只读资源，
     文件名需为 `vocabulary.csv`）。
5. 选中 iPhone 模拟器，Run。应该能看到主页、学习卡片、听写判题、德语朗读。

> 做完第 2 步并把 `WortMeister.xcodeproj` 提交到仓库后，`ios.yml` 的
> “Build app for Simulator”步骤就会在 CI 上真正编译 App，而不再只是提示。

---

## 5. 上架 App Store 的完整路径（需要 Apple 账号）

1. 注册 **Apple Developer Program**（个人 $99/年）。
2. 在 Xcode 用你的开发者账号登录，设置 Signing & Capabilities 的 Team。
3. 本机 Archive 一次确认能导出 .ipa，或配置 CI 签名：
   - 导出证书 (.p12) 和描述文件 (.mobileprovision)，`base64` 后存入仓库
     Secrets：`BUILD_CERTIFICATE_BASE64`、`P12_PASSWORD`、
     `PROVISIONING_PROFILE_BASE64`、`KEYCHAIN_PASSWORD`、`APPLE_TEAM_ID`。
   - 解开 `ios.yml` 里的 `device-ipa` 任务。
4. 用 `xcrun altool` / `xcrun notarytool` 或 Xcode Organizer 上传到
   TestFlight，内测通过后提交 App Store 审核。

---

## 6. 跟桌面版的关系（不破坏现有功能）

- 本次改动是**纯新增**：`core/`、`ios/`、`.github/workflows/ios.yml`，没有修改
  桌面版 `src/` 下的任何文件，Windows/macOS 桌面版照常构建。
- 建议后续把桌面版 `src/model.rs`、`src/srs.rs` 的逻辑也改为依赖 `core/`，这样
  桌面和 iOS 共享同一份算法，避免两边分叉。可在桌面 `Cargo.toml` 加：
  ```toml
  wortmeister_core = { path = "core" }
  ```
  然后桌面侧只保留 egui UI、字体加载和 `tts` 朗读。

---

## 7. FFI 约定（给维护者）

所有数据以 UTF-8 JSON 字符串跨越边界，返回统一信封：

```json
{ "ok": true,  "data": <结果> }
{ "ok": false, "error": "<错误信息>" }
```

- 输入 `const char*` 由调用方拥有，Rust 不释放。
- 返回的 `char*` 由调用方拥有，**必须**用 `wm_string_free` 释放一次
  （Swift 封装已用 `defer` 自动处理）。
- Rust 侧用 `catch_unwind` 兜住 panic，不会跨 FFI 边界 unwind。

导出函数：`wm_version` / `wm_parse_words` / `wm_make_queue` / `wm_grade_card`
/ `wm_check_answer` / `wm_levels` / `wm_string_free`。
