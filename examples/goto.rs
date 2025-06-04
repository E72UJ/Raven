use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
// 新的结构体
#[derive(Deserialize, Debug)]
struct AssetPaths {
    characters: CharacterPaths,
    backgrounds: BackgroundPaths,
    audio: AudioPaths,
    videos: VideoPaths,
    swf: SwfPaths,
}

#[derive(Deserialize, Debug)]
struct CharacterPaths {
    protagonist: String,
    heroine: String,
    villain: String,
    none: String,
}

#[derive(Deserialize, Debug)]
struct BackgroundPaths {
    school: String,
    cafe: String,
    park: String,
}

#[derive(Deserialize, Debug)]
struct AudioPaths {
    bgm: BgmPaths,
    sfx: SfxPaths,
    click_sound: String,
    backclick_sound: String,
}

#[derive(Deserialize, Debug)]
struct BgmPaths {
    main_theme: String,
    battle: String,
}

#[derive(Deserialize, Debug)]
struct SfxPaths {
    doorbell: String,
    explosion: String,
}

#[derive(Deserialize, Debug)]
struct VideoPaths {
    opening: String,
    ending: String,
}

#[derive(Deserialize, Debug)]
struct SwfPaths {
    mini_game: String,
    special_effect: String,
}

// 新的结构体结束
// 游戏窗口管理结构题
// 游戏设置结构体
#[derive(Debug, Deserialize)]
struct winadmin {
    resize: bool,
}
// swf 结构体
#[derive(Debug, Deserialize)]
struct swfbox {
    name: String,
    visible: bool
}

//  yaml读取主要程序
#[derive(Debug, Deserialize)]
struct MainConfig {
    title: String,
    window: winadmin,
    assets: AssetPaths,
    // 其他字段...
}

// 定义对话结构
#[derive(Debug, Deserialize)]
struct Dialogue {
    character: String,
    label: Option<String>,
    text: String,
    portrait: String,
    swf: Option<swfbox>,
    jump: Option<String>,
    // 其他字段...
}
// 对话管理工具
struct DialogueManager {
    dialogues: Vec<Dialogue>,
    current_index: usize,
}
impl DialogueManager {
    fn new(dialogues: Vec<Dialogue>) -> Self {
        DialogueManager {
            dialogues,
            current_index: 0,
        }
    }

    fn display_current_dialogue(&mut self) {
        let current_dialogue = &self.dialogues[self.current_index];
        println!("Character: {}", current_dialogue.character);
        println!("Text: {}", current_dialogue.text);

        if let Some(label) = &current_dialogue.label {
            println!("Label: {}", label);
        }

        if let Some(swf) = &current_dialogue.swf {
            println!("SWF Name: {}", swf.name);
            println!("SWF Visible: {}", swf.visible);
        }

        if let Some(jump) = &current_dialogue.jump {
            println!("Jumping to label: {}", jump);
            if let Some(jump_index) = self.dialogues.iter().position(|d| d.label == Some(jump.to_string())) {
                self.current_index = jump_index;
            } else {
                println!("Error: Cannot find label '{}'", jump);
            }
        } else {
            self.current_index += 1;
        }
    }
}
fn main() {
    // 1. 加载主配置
    let config = load_main_config();
    // let config2 = load_main_config2();
    // println!("标题: {}", config.title);
    // println!("窗口是否可以自由变化: {}", config.window.resize);
    // 2. 加载对话数据（自动应用变量替换）
    let dialogues = load_dialogues(&config);
    // println!("首个对话: {:?}", dialogues[0].label);
    // println!("读取swf结构体{:?}",dialogues[0]);
    // println!("读取swf结构体{:?}",dialogues[0]);
    if let Some(swf) = &dialogues[1].swf {
        println!("SWF name: {}", swf.name);
        println!("SWF visible: {}", swf.visible);
    } else {
        println!("该对话没有 SWF 控件");
    }
    // 3. 创建 DialogueManager 并显示对话
    let mut dialogue_manager = DialogueManager::new(dialogues);
    dialogue_manager.display_current_dialogue();
    dialogue_manager.display_current_dialogue();
    dialogue_manager.display_current_dialogue();
}


// yaml读取主要的程序
// 1. 获取可执行文件所在目录
fn get_exe_dir() -> PathBuf {
    env::current_exe()
        .expect("无法获取可执行文件路径")
        .parent()
        .expect("无法获取父目录")
        .to_path_buf()
}

// 2. 加载主配置文件
fn load_main_config() -> MainConfig {
    // 获取可执行文件目录
    let exe_dir = get_exe_dir();
    
    // 构建配置文件路径 (相对于可执行文件目录)
    let config_path = exe_dir.join("assets/main.yaml");
    println!("配置文件路径: {:?}", config_path);
    
    // 读取文件内容
    let yaml_str = fs::read_to_string(&config_path)
        .unwrap_or_else(|_| panic!("找不到配置文件: {:?}", config_path));
    
    // 解析YAML
    serde_yaml::from_str(&yaml_str)
        .unwrap_or_else(|e| panic!("YAML解析失败: {}\n文件路径: {:?}", e, config_path))
        
}

// 3. 加载对话数据（带变量替换）
fn load_dialogues(config: &MainConfig) -> Vec<Dialogue> {
    // 获取可执行文件目录
    let exe_dir = get_exe_dir();
    
    // 构建对话文件路径
    let dialogues_path = exe_dir.join("assets/dialogues.yaml");
    println!("对话文件路径: {:?}", dialogues_path);
    
    // 读取文件内容
    let yaml_str = fs::read_to_string(&dialogues_path)
        .unwrap_or_else(|_| panic!("找不到对话文件: {:?}", dialogues_path));
    
    // 变量替换处理
    let mut processed_yaml = yaml_str.clone();
    // for (var_name, var_value) in &config.global_variables {
    //     processed_yaml = processed_yaml.replace(&format!("${}", var_name), var_value);
    // }
    
    // 解析YAML
    serde_yaml::from_str(&processed_yaml)
        .unwrap_or_else(|e| panic!("对话YAML解析失败: {}\n文件路径: {:?}", e, dialogues_path))
}
