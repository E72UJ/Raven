use clap::{Parser, Subcommand};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

// 嵌入资源常量
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

#[derive(Parser)]
#[command(name = "Raven")]
#[command(version = "0.1.0")]
#[command(about = "Raven 视觉小说引擎 CLI", long_about = r#"
  ____                      
 |  _ \    __ _  __   __   ___   _ __  
 | |_) |  / _` | \ \ / /  / _ \ | '_ \ 
 |  _ <  | (_| |  \ V /  |  __/ | | | |
 |_| \_\  \__,_|   \_/    \___| |_| |_|
"#)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建新视觉小说项目
    New {
        /// 项目名称（直接跟在命令后）
        #[arg(index = 1)]
        name: String,
    },

    /// 运行已创建的项目
    Run {
        /// 项目名称（直接跟在命令后）
        #[arg(index = 1)]
        project_name: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name } => create_project(&name),
        Commands::Run { project_name } => run_project(&project_name),
    }
}

fn create_project(name: &str) {
    let project_dir = Path::new(name);
    
    // 创建目录结构
    let assets = project_dir.join("assets");
    let dirs = [
        assets.join("fonts"),
        assets.join("portraits"),
        assets.join("characters/svg"),
        assets.join("characters/heroine"),
        assets.join("characters/protagonist"),
        assets.join("characters/villain"),
        assets.join("backgrounds"),
        assets.join("audio"),
        assets.join("configs"),
        assets.join("swf"), // 新增SWF目录
    ];

    for dir in &dirs {
        fs::create_dir_all(dir)
            .unwrap_or_else(|_| panic!("创建目录 {} 失败", dir.display()));
    }

    // 写入嵌入式资源
    write_embedded_file(EMBEDDED_RAVEN_EXE, &project_dir.join("Raven"));
    write_embedded_file(EMBEDDED_MAIN_YAML, &assets.join("main.yaml"));
    write_embedded_file(EMBEDDED_DIALOGUES_YAML, &assets.join("dialogues.yaml"));
    
    // 写入字体
    write_embedded_file(
        EMBEDDED_FONT_FIRA_MONO,
        &assets.join("fonts/FiraMono-Medium.ttf"),
    );
    write_embedded_file(
        EMBEDDED_FONT_GENSEN_MARU,
        &assets.join("fonts/GenSenMaruGothicTW-Bold.ttf"),
    );
    write_embedded_file(
        EMBEDDED_FONT_SOURCE_HAN,
        &assets.join("fonts/SourceHanSansSC-Medium.otf"),
    );
    
    // 写入立绘
    write_embedded_file(
        EMBEDDED_PORTRAIT_ALICE,
        &assets.join("portraits/alice.png"),
    );
    write_embedded_file(
        EMBEDDED_PORTRAIT_BOB,
        &assets.join("portraits/bob.png"),
    );
    write_embedded_file(
        EMBEDDED_PORTRAIT_NARRATOR,
        &assets.join("portraits/narrator.png"),
    );
    write_embedded_file(
        EMBEDDED_SVG_SCIHUB,
        &assets.join("portraits/SciHub.svg"),
    );

    // 写入角色资源
    write_embedded_file(
        EMBEDDED_CHAR_HEROINE_DEFAULT,
        &assets.join("characters/heroine/default.png"),
    );
    write_embedded_file(
        EMBEDDED_CHAR_PROTAGONIST_DEFAULT,
        &assets.join("characters/protagonist/default.png"),
    );
    write_embedded_file(
        EMBEDDED_CHAR_VILLAIN_DEFAULT,
        &assets.join("characters/villain/default.png"),
    );
    write_embedded_file(
        EMBEDDED_SVG_LONG,
        &assets.join("characters/svg/long.svg"),
    );

    // 新增SWF资源写入
    write_embedded_file(
        EMBEDDED_SWF_66,
        &assets.join("swf/66.swf"),
    );

    println!("\n项目创建成功！目录结构：");
    print_project_tree(name);
}

fn write_embedded_file(content: &[u8], path: &Path) {
    fs::write(path, content)
        .unwrap_or_else(|_| panic!("写入文件 {} 失败", path.display()));
}

fn print_project_tree(project_dir: &str) {
    println!(
        r#"
{}
├── Raven
└── assets/
    ├── swf/
    │   └── 66.swf       ({} KB)
    ├── portraits/
    │   ├── alice.png     ({} KB)
    │   ├── bob.png       ({} KB)
    │   ├── narrator.png  ({} KB)
    │   └── SciHub.svg    ({} KB)
    ├── characters/
    │   ├── svg/
    │   │   └── long.svg  ({} KB)
    │   ├── heroine/
    │   │   └── default.png ({} KB)
    │   ├── protagonist/
    │   │   └── default.png ({} KB)
    │   └── villain/
    │       └── default.png ({} KB)
    ├── fonts/
    │   ├── FiraMono-Medium.ttf
    │   ├── GenSenMaruGothicTW-Bold.ttf
    │   └── SourceHanSansSC-Medium.otf
    ├── main.yaml
    └── dialogues.yaml"#,
        project_dir,
        EMBEDDED_SWF_66.len() / 1024,
        EMBEDDED_PORTRAIT_ALICE.len() / 1024,
        EMBEDDED_PORTRAIT_BOB.len() / 1024,
        EMBEDDED_PORTRAIT_NARRATOR.len() / 1024,
        EMBEDDED_SVG_SCIHUB.len() / 1024,
        EMBEDDED_SVG_LONG.len() / 1024,
        EMBEDDED_CHAR_HEROINE_DEFAULT.len() / 1024,
        EMBEDDED_CHAR_PROTAGONIST_DEFAULT.len() / 1024,
        EMBEDDED_CHAR_VILLAIN_DEFAULT.len() / 1024
    );
}

fn run_project(project_name: &str) {
    let executable_path = Path::new(project_name).join("Raven");
    
    // 验证可执行文件存在
    if !executable_path.exists() {
        panic!("❌ 找不到可执行文件：{}", executable_path.display());
    }

    // 设置执行权限（Unix 系统）
    set_executable_permissions(&executable_path);

    // 运行程序
    println!("🚀 正在启动项目：{}", project_name);
    let status = Command::new(&executable_path)
        .status()
        .unwrap_or_else(|e| panic!("💥 运行失败：{}", e));

    // 处理退出状态
    match status.code() {
        Some(code) if code != 0 => eprintln!("⚠️ 程序异常退出，状态码：{}", code),
        None => eprintln!("⚠️ 程序被信号终止"),
        _ => (),
    }
}

fn set_executable_permissions(path: &Path) {
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(path)
            .unwrap_or_else(|_| panic!("🔧 无法获取文件元数据：{}", path.display()))
            .permissions();
        
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)
            .unwrap_or_else(|_| panic!("🔧 无法设置执行权限：{}", path.display()));
    }

    #[cfg(windows)]
    {
        let _ = path;
    }
}