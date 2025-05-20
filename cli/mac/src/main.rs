// main.rs
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

// 嵌入资源
const EMBEDDED_FONT_FIRA_MONO: &[u8] = include_bytes!("../../crates/assets/fonts/FiraMono-Medium.ttf");
const EMBEDDED_FONT_GENSEN_MARU: &[u8] = include_bytes!("../../crates/assets/fonts/GenSenMaruGothicTW-Bold.ttf");
const EMBEDDED_RAVEN_EXE: &[u8] = include_bytes!("../../crates/assets/Raven");
const EMBEDDED_PORTRAIT_ALICE: &[u8] = include_bytes!("../../crates/assets/portraits/alice.png");
const EMBEDDED_PORTRAIT_BOB: &[u8] = include_bytes!("../../crates/assets/portraits/bob.png");
const EMBEDDED_PORTRAIT_NARRATOR: &[u8] = include_bytes!("../../crates/assets/portraits/narrator.png");
const EMBEDDED_MAIN_YAML: &[u8] = include_bytes!("../../crates/assets/main.yaml");
const EMBEDDED_DIALOGUES_YAML: &[u8] = include_bytes!("../../crates/assets/dialogues.yaml");

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
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name } => create_project(&name),
    }
}

fn create_project(name: &str) {
    let project_dir = Path::new(name);
    
    // 创建目录结构
    let assets = project_dir.join("assets");
    let dirs = [
        assets.join("fonts"),
        assets.join("portraits"),
        assets.join("backgrounds"),
        assets.join("audio"),
        assets.join("configs"),
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

    println!("\n项目创建成功！目录结构：");
    print_project_tree(name);
}

fn write_embedded_file(content: &[u8, ], path: &Path) {
    fs::write(path, content)
        .unwrap_or_else(|_| panic!("写入文件 {} 失败", path.display()));
}

fn print_project_tree(project_dir: &str) {
    println!(
        r#"
{}
├── Raven
└── assets/
    ├── portraits/
    │   ├── alice.png     ({} KB)
    │   ├── bob.png       ({} KB)
    │   └── narrator.png  ({} KB)
    ├── fonts/
    │   ├── FiraMono-Medium.ttf
    │   └── GenSenMaruGothicTW-Bold.ttf
    ├── main.yaml
    └── dialogues.yaml"#,
        project_dir,
        EMBEDDED_PORTRAIT_ALICE.len() / 1024,
        EMBEDDED_PORTRAIT_BOB.len() / 1024,
        EMBEDDED_PORTRAIT_NARRATOR.len() / 1024
    );
}