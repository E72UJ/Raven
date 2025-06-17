# 常用命令
## 安装 rust 
```
# Linux/macOS
export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows (PowerShell)
$env:RUSTUP_DIST_SERVER = "https://mirrors.ustc.edu.cn/rust-static"
$env:RUSTUP_UPDATE_ROOT = "https://mirrors.ustc.edu.cn/rust-static/rustup"
irm https://sh.rustup.rs -UseBasicParsing | iex
```
### 优化编译
```
cargo build --release
```

### 执行案例程序
```
cargo run --example goto
```
### 常用cargo 指令
```
| `cargo clean` | 清理项目的构建成果 |
| `cargo update` | 更新项目的依赖项 |
| `cargo add` | 添加新的依赖项 |
| `cargo remove` | 移除项目的依赖项 |
```