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
cargo run --example test
```
### 常用cargo 指令
```
# 完整的代码检查流程
cargo fmt && cargo clippy && cargo test

# 快速检查代码质量
cargo check --all-targets && cargo clippy -- -D warnings

# 发布前检查
cargo fmt --check && cargo clippy && cargo test && cargo package --dry-run

# 清理并重新构建
cargo clean && cargo build --release

# 查看依赖更新
cargo update --dry-run && cargo tree
```
## 查看目录

```find . -name "*.rs" -o -name "Cargo.toml" -o -name "src" -type d | head -20```