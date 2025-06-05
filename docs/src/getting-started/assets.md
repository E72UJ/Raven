# 资产文件结构
```
const EMBEDDED_FONT_FIRA_MONO: &[u8] = include_bytes!("../../crates/assets/fonts/FiraMono-Medium.ttf");
const EMBEDDED_FONT_GENSEN_MARU: &[u8] = include_bytes!("../../crates/assets/fonts/GenSenMaruGothicTW-Bold.ttf");
const EMBEDDED_FONT_SOURCE_HAN: &[u8] = include_bytes!("../../crates/assets/fonts/SourceHanSansSC-Medium.otf");
const EMBEDDED_RAVEN_EXE: &[u8] = include_bytes!("../../crates/assets/Raven");
const EMBEDDED_PORTRAIT_ALICE: &[u8] = include_bytes!("../../crates/assets/portraits/alice.png");
const EMBEDDED_PORTRAIT_BOB: &[u8] = include_bytes!("../../crates/assets/portraits/bob.png");
const EMBEDDED_PORTRAIT_NARRATOR: &[u8] = include_bytes!("../../crates/assets/portraits/narrator.png");
const EMBEDDED_SVG_SCIHUB: &[u8] = include_bytes!("../../crates/assets/portraits/SciHub.svg");
const EMBEDDED_MAIN_YAML: &[u8] = include_bytes!("../../crates/assets/main.yaml");
const EMBEDDED_DIALOGUES_YAML: &[u8] = include_bytes!("../../crates/assets/dialogues.yaml");
const EMBEDDED_CHAR_HEROINE_DEFAULT: &[u8] = include_bytes!("../../crates/assets/characters/heroine/default.png");
const EMBEDDED_CHAR_PROTAGONIST_DEFAULT: &[u8] = include_bytes!("../../crates/assets/characters/protagonist/default.png");
const EMBEDDED_CHAR_VILLAIN_DEFAULT: &[u8] = include_bytes!("../../crates/assets/characters/villain/default.png");
const EMBEDDED_SVG_LONG: &[u8] = include_bytes!("../../crates/assets/characters/svg/long.svg");
const EMBEDDED_SWF_66: &[u8] = include_bytes!("../../crates/assets/swf/66.swf"); // 新增SWF资源
```

| 文件/文件夹 | 描述 |
| --- | --- |
| `assets` | 存放项目所需的资源文件,如图片、字体等 |
| `Cargo.toml` | Rust 项目的配置文件,定义了项目的名称、版本、依赖等信息 |
| `Cargo.lock` | 记录了项目依赖的确切版本,确保项目在不同环境下都使用相同的依赖版本 |
| `cli` | 命令行工具相关的源代码 |
| `crates` | 存放项目发布到 crates.io 的压缩包 |
| `crates.zip` | 项目的发布压缩包 |
| `demo` | 项目的示例代码 |
| `docs` | 项目的文档 |
| `examples` | 存放项目的示例代码 |
| `LICENSE-APACHE.txt` | 项目的 Apache 2.0 许可证 |
| `README.md` | 项目的自述文件 |
| `src` | 项目的源代码文件夹 |
| `target` | Rust 编译器生成的目标文件存放目录 |