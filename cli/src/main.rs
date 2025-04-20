use clap::{Parser, Subcommand};
use std::fs;
use std::process::Command;

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
    /// 创建一个新的视觉小说项目
    Create {
        /// 项目名称
        name: String,
    },
    /// 编译视觉小说项目
    Compile {
        /// 项目名称
        name: String,
    },
    // 移除了 Help 子命令，因为 clap 已经自动提供了帮助功能
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { name } => create_project(&name),
        Commands::Compile { name } => compile_project(&name),
        // 也需要移除这里的 Help 匹配分支
    }
}

// 移除 print_help 函数，因为不再需要它

fn create_project(name: &str) {
    let project_dir = format!("./{}", name);
    
    // 创建项目目录
    if fs::create_dir_all(&project_dir).is_err() {
        eprintln!("创建项目目录失败。");
        return;
    }

    // 创建基本的main.rs文件
    let main_rs_content = r#"
fn main() {
    println!("你好，视觉小说！");
}
"#;

    let main_rs_path = format!("{}/main.rs", project_dir);
    if fs::write(main_rs_path, main_rs_content).is_err() {
        eprintln!("创建main.rs文件失败。");
        return;
    }

    println!("项目 '{}' 创建成功。", name);
}

fn compile_project(name: &str) {
    let project_dir = format!("./{}", name);
    
    // 检查项目目录是否存在
    if !std::path::Path::new(&project_dir).exists() {
        eprintln!("项目 '{}' 不存在。请先创建项目。", name);
        return;
    }

    // 编译项目
    let output = Command::new("cargo")
        .arg("build")
        .current_dir(&project_dir)
        .output()
        .expect("编译项目失败。");

    if output.status.success() {
        println!("项目 '{}' 编译成功。", name);
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("编译项目 '{}' 失败: {}", name, error_message);
    }
}