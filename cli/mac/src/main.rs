use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::io::{self, Write};

fn main() {
    print_logo();  // 添加主动打印LOGO的代码
    let cli = Cli::parse();
    if let Err(e) = cli.command.run() {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
}

#[derive(Parser)]
#[command(name = "raven-mac-cli")]
#[command(about = "视觉小说项目的命令行工具 mac 版本")]
#[command(version = "1.0")]
// #[command(
//     long_about = r"  ____                                 
//  |  _ \    __ _  __   __   ___   _ __  
//  | |_) |  / _` | \ \ / /  / _ \ | '_ \ 
//  |  _ <  | (_| |  \ V /  |  __/ | | | |
//  |_| \_\  \__,_|   \_/    \___| |_| |_|
//                                         "
// )]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化新项目
    Init {
        /// 项目名称
        #[arg(short, long)]
        project_name: String
    },
    /// 创建新场景
    New {
        /// 场景名称
        #[arg(short, long)]
        scene_name: String
    },
    /// 构建项目
    Build {
        /// 构建类型 (debug/release)
        #[arg(short, long, default_value = "debug")]
        build_type: String
    }
}

impl Commands {
    fn run(&self) -> io::Result<()> {
        match self {
            Commands::Init { project_name } => {
                create_project(project_name)
            }
            Commands::New { scene_name } => {
                create_scene(scene_name)
            }
            Commands::Build { build_type } => {
                build_project(build_type)
            }
        }
    }
}

/// 项目初始化逻辑
fn create_project(project_name: &str) -> io::Result<()> {
    let project_dir = Path::new(project_name);
    
    // 创建项目目录结构
    fs::create_dir(project_dir)?;
    fs::create_dir(project_dir.join("scenes"))?;
    fs::create_dir(project_dir.join("assets"))?;
    
    // 创建配置文件
    let mut config = fs::File::create(project_dir.join("config.yaml"))?;
    writeln!(config, "name: {}", project_name)?;
    writeln!(config, "version: 0.1.0")?;
    
    println!("项目 {} 创建成功！", project_name);
    Ok(())
}

/// 场景创建逻辑
fn create_scene(scene_name: &str) -> io::Result<()> {
    let scene_path = format!("scenes/{}.yaml", scene_name);
    let mut scene_file = fs::File::create(&scene_path)?;
    
    writeln!(scene_file, "characters: []")?;
    writeln!(scene_file, "dialogue: []")?;
    writeln!(scene_file, "background: default.jpg")?;
    
    println!("场景 {} 创建成功！", scene_name);
    Ok(())
}

/// 项目构建逻辑
fn build_project(build_type: &str) -> io::Result<()> {
    println!("开始 {} 构建...", build_type);
    // 这里可以添加实际的构建逻辑
    match build_type {
        "release" => println!("执行优化构建..."),
        _ => println!("执行调试构建...")
    }
    println!("构建完成！");
    Ok(())
}
fn print_logo() {
    println!(r#"
  ____                                 
 |  _ \    __ _  __   __   ___   _ __  
 | |_) |  / _` | \ \ / /  / _ \ | '_ \ 
 |  _ <  | (_| |  \ V /  |  __/ | | | |
 |_| \_\  \__,_|   \_/    \___| |_| |_|
    "#);
    println!(r#"
    编译核心:M4 Pro
    作者:Furau
    "#);    
}