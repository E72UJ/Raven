# MacOS 程序源码编译手册
## 安装rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 运行cargo
```
cargo run
```

## 安装插件库
- bevy
- bevy_flash
- flash_runtime
- swf_animation-main
- swf_macro

## 依赖配置列表
**Raven/crates/bevy_flash/Cargo.toml**
```
[package]
name = "bevy_flash"
version = "0.1.0"
edition = "2024"
authors = ["傲娇小霖霖"]
license = "MIT OR Apache-2.0"
description = "A Bevy plugin for Flash Animation"

[dependencies]
swf_macro = { path = "./swf_macro" }

smallvec = { version = "1.14.0", features = ["union"] }
bytemuck = { version = "1.21.0", features = ["derive"] }

bevy = { path = "../bevy", default-features = false, features = [
    "bevy_asset",
    "bevy_sprite",
] }

thiserror = "1.0"
anyhow = "1.0"
bitflags = "2.5"
lyon_tessellation = "1.0"

copyless = "0.1.5"
uuid = "1.10"
enum-map = "2.7.3"
swf = "0.2"
flash_runtime = { path = "../flash_runtime" }
[dev-dependencies]
bevy = { path = "../bevy", features = ["bevy_dev_tools"] }


[profile.dev]
opt-level = 1

```