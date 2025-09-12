# Ton - 桌面悬浮工具

一个基于 Tauri + Vue 3 + TypeScript + UnoCSS + Pinia 的桌面悬浮工具，具有透明背景和弹出配置页面的特性。

## ✨ 特性

- 🖥️ **桌面悬浮**: 窗口始终在最顶层，不显示在任务栏中
- 🔍 **透明背景**: 支持可调节透明度的玻璃态效果
- 🎨 **动态边框**: 鼠标划过时显示边框，支持自定义颜色和宽度
- ⚙️ **弹出配置**: 点击设置按钮弹出现代化配置界面
- 🎯 **拖拽移动**: 支持拖拽窗口位置
- 🎨 **UnoCSS**: 使用原子化CSS框架，样式简洁高效
- 📱 **响应式**: 支持窗口大小调整
- 🔧 **TypeScript**: 完整的类型支持

## 🚀 快速开始

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri:dev
```

### 构建应用

```bash
pnpm tauri:build
```

## 🛠️ 技术栈

- **前端框架**: Vue 3 + TypeScript
- **状态管理**: Pinia
- **样式框架**: UnoCSS
- **桌面框架**: Tauri 2.0
- **构建工具**: Vite
- **包管理器**: pnpm

## 📁 项目结构

```
src/
├── components/          # Vue 组件
│   ├── FloatingWindow.vue   # 主悬浮窗口组件
│   └── SettingsModal.vue    # 设置弹窗组件
├── store/              # Pinia 状态管理
│   └── app.ts             # 应用配置状态
├── views/              # 页面视图
├── router/             # 路由配置
└── main.ts             # 应用入口

src-tauri/              # Tauri 后端
├── src/                # Rust 源码
├── Cargo.toml          # Rust 依赖
└── tauri.conf.json     # Tauri 配置
```

## ⚙️ 配置说明

### Tauri 配置 (`src-tauri/tauri.conf.json`)

- `transparent: true` - 启用窗口透明
- `decorations: false` - 隐藏窗口边框
- `alwaysOnTop: true` - 窗口始终在最顶层
- `skipTaskbar: true` - 不显示在任务栏

### 窗口特性

- **透明度调节**: 10%-100% 可调
- **圆角半径**: 0-20px 可调
- **边框样式**: 颜色、宽度可自定义
- **窗口大小**: 支持最小尺寸限制

## 🎨 自定义样式

项目使用 UnoCSS，支持原子化CSS类名：

```vue
<!-- 使用预设的按钮样式 -->
<button class="btn btn-primary">
确定
</button>

<!-- 使用颜色变量 -->
<div class="bg-primary text-white">
主要颜色
</div>

<!-- 使用响应式设计 -->
<div class="w-100 md:w-50 lg:w-25">
响应式宽度
</div>
```

## 📝 开发说明

### 推荐 IDE 设置

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?rust-lang.rust-analyzer)

### TypeScript 支持

项目使用 Vue 3 `<script setup>` SFC，详情请参考 [script setup 文档](https://v3.vuejs.org/api/sfc-script-setup.html)。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License
