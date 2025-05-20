use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

// 嵌入所有资源文件
const EMBEDDED_FONT_FIRA_MONO: &[u8] = include_bytes!("../../crates/assets/fonts/FiraMono-Medium.ttf");
const EMBEDDED_FONT_GENSEN_MARU: &[u8] = include_bytes!("../../crates/assets/fonts/GenSenMaruGothicTW-Bold.ttf");
const EMBEDDED_RAVEN_EXE: &[u8] = include_bytes!("../../crates/assets/Raven.exe");
const EMBEDDED_PORTRAIT_ALICE: &[u8] = include_bytes!("../../crates/assets/portraits/alice.png");
const EMBEDDED_PORTRAIT_BOB: &[u8] = include_bytes!("../../crates/assets/portraits/bob.png");
const EMBEDDED_PORTRAIT_NARRATOR: &[u8] = include_bytes!("../../crates/assets/portraits/narrator.png");
const EMBEDDED_MAIN_YAML: &[u8] = include_bytes!("../../crates/assets/main.yaml");
const EMBEDDED_DIALOGUES_YAML: &[u8] = include_bytes!("../../crates/assets/dialogues.yaml");

#[derive(Parser)]

#[command(name = "visual_novel_cli")]
#[command(about = "视觉小说项目的命令行工具")]
#[command(about = r"  ____                                 
 |  _ \    __ _  __   __   ___   _ __  
 | |_) |  / _` | \ \ / /  / _ \ | '_ \ 
 |  _ <  | (_| |  \ V /  |  __/ | | | |
 |_| \_\  \__,_|   \_/    \___| |_| |_|
                                        ")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建新视觉小说项目
    Create {
        /// 项目名称
        name: String,
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Create { name } => create_project(&name),
    }
}

fn create_project(name: &str) {
    let project_dir = format!("./{}", name);
    
    // 创建目录结构
    let dirs = [
        format!("{}/assets/fonts", project_dir),
        format!("{}/assets/portraits", project_dir),
        format!("{}/assets/backgrounds", project_dir),
        format!("{}/assets/audio", project_dir),
        format!("{}/assets/configs", project_dir),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir).unwrap_or_else(|_| panic!("创建目录 {} 失败", dir));
    }

    // 写入嵌入式资源
    write_embedded_file(EMBEDDED_RAVEN_EXE, &format!("{}/Raven.exe", project_dir));
    write_embedded_file(EMBEDDED_MAIN_YAML, &format!("{}/assets/main.yaml", project_dir));
    write_embedded_file(EMBEDDED_DIALOGUES_YAML, &format!("{}/assets/dialogues.yaml", project_dir));
    
    // 写入字体
    write_embedded_file(EMBEDDED_FONT_FIRA_MONO, &format!("{}/assets/fonts/FiraMono-Medium.ttf", project_dir));
    write_embedded_file(EMBEDDED_FONT_GENSEN_MARU, &format!("{}/assets/fonts/GenSenMaruGothicTW-Bold.ttf", project_dir));
    
    // 写入立绘
    write_embedded_file(EMBEDDED_PORTRAIT_ALICE, &format!("{}/assets/portraits/alice.png", project_dir));
    write_embedded_file(EMBEDDED_PORTRAIT_BOB, &format!("{}/assets/portraits/bob.png", project_dir));
    write_embedded_file(EMBEDDED_PORTRAIT_NARRATOR, &format!("{}/assets/portraits/narrator.png", project_dir));

    println!("\n项目创建成功！目录结构：");
    print_project_tree(&project_dir);
}

fn write_embedded_file(content: &[u8], path: &str) {
    fs::write(path, content).unwrap_or_else(|_| panic!("写入文件 {} 失败", path));
}

fn print_project_tree(project_dir: &str) {
    println!("{}
├── Raven.exe
└── assets/
    ├── portraits/
    │   ├── alice.png     大小：{} KB
    │   ├── bob.png       大小：{} KB
    │   └── narrator.png  大小：{} KB
    ├── fonts/
    │   ├── FiraMono-Medium.ttf
    │   └── GenSenMaruGothicTW-Bold.ttf
    ├── main.yaml
    └── dialogues.yaml",
        project_dir,
        EMBEDDED_PORTRAIT_ALICE.len() / 1024,
        EMBEDDED_PORTRAIT_BOB.len() / 1024,
        EMBEDDED_PORTRAIT_NARRATOR.len() / 1024
    );
}