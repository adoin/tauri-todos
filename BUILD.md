# 构建和发布指南

## 本地构建

### 安装依赖
```bash
# 安装 Node.js 依赖
pnpm install

# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 安装 Tauri CLI
cargo install tauri-cli --version "^2.0"
```

### 构建命令

#### 构建当前平台
```bash
pnpm tauri:build
```

#### 构建特定平台
```bash
# Windows (x64)
pnpm tauri:build:windows

# macOS (Intel)
pnpm tauri:build:macos

# macOS (Apple Silicon)
pnpm tauri:build:macos-arm

# Linux (x64)
pnpm tauri:build:linux
```

#### 构建所有平台
```bash
pnpm tauri:build:all
```

## 自动发布

### 版本管理
1. 更新 `package.json` 中的版本号
2. 提交并推送到 `main` 分支
3. GitHub Actions 会自动检测版本变化并构建发布

### 版本检测逻辑
- 当 `main` 分支接收到新的提交时，工作流会检查 `package.json` 中的版本号
- 如果版本号与上一个提交不同，则触发构建和发布流程
- 如果版本号相同，则跳过构建（除非是 PR）

### 支持的平台
- **Windows**: x64 (`.msi` 安装包)
- **macOS**: Intel x64 和 Apple Silicon ARM64 (`.dmg` 安装包)
- **Linux**: x64 (`.AppImage` 和 `.deb` 包)

### 发布流程
1. 版本检测
2. 并行构建所有平台
3. 创建 GitHub Release
4. 上传构建产物到 Release

### 注意事项
- 确保在 GitHub 仓库设置中配置了必要的 Secrets（如果需要代码签名）
- 构建产物会保留 30 天
- 发布会在 GitHub Releases 页面可见

## 开发模式
```bash
# 启动开发服务器
pnpm dev

# 启动 Tauri 开发模式
pnpm tauri:dev
```

## 故障排除

### 常见问题
1. **Rust 工具链问题**: 确保安装了正确的 Rust 版本和目标平台
2. **依赖问题**: 运行 `pnpm install` 确保所有依赖都已安装
3. **权限问题**: 确保有足够的权限创建发布

### 调试构建
```bash
# 检查 Rust 工具链
rustc --version
cargo --version

# 检查 Tauri CLI
tauri --version

# 检查目标平台
rustup target list --installed
```
