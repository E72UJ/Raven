use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, path::PathBuf};

#[derive(Resource, Deserialize, Serialize, Clone)]
pub struct MainConfig {
    pub title: String,
    pub assets: AssetsConfig,
    pub settings: SettingsConfig,
    pub global_variables: GlobalVariables,
    pub variables: HashMap<String, VariableValue>, // 简化为键值对
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AssetsConfig {
    pub characters: HashMap<String, String>,
    pub backgrounds: HashMap<String, String>,
    pub audio: AudioConfig,
    pub videos: HashMap<String, String>,
    pub swf: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AudioConfig {
    pub bgm: HashMap<String, String>,
    pub sfx: HashMap<String, String>,
    pub click_sound: String,
    pub backclick_sound: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SettingsConfig {
    pub initial_scene: String,
    pub text_speed: u32,
    pub auto_save: bool,
    pub resolution: [u32; 2],
    pub font: String,
    pub rewind: bool,
    pub logo_text: String,
    pub resizable: bool,     // 是否允许改变窗口大小
    pub maximizable: bool,   // 是否允许最大化按钮
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GlobalVariables {
    pub player_name: String,
    pub affection_points: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum VariableValue {
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<VariableValue>),
    Null,
}

// 为VariableValue实现Default
impl Default for VariableValue {
    fn default() -> Self {
        VariableValue::Null
    }
}

// 实现一些便利方法
impl VariableValue {
    // 转换为布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            VariableValue::Boolean(b) => Some(*b),
            VariableValue::Number(n) => Some(*n != 0.0),
            VariableValue::String(s) => Some(!s.is_empty()),
            _ => None,
        }
    }
    
    // 转换为数字
    pub fn as_number(&self) -> Option<f64> {
        match self {
            VariableValue::Number(n) => Some(*n),
            VariableValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            VariableValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    // 转换为字符串
    pub fn as_string(&self) -> Option<String> {
        match self {
            VariableValue::String(s) => Some(s.clone()),
            VariableValue::Number(n) => Some(n.to_string()),
            VariableValue::Boolean(b) => Some(b.to_string()),
            VariableValue::Array(a) => Some(format!("{:?}", a)),
            VariableValue::Null => Some("null".to_string()),
        }
    }
    
    // 转换为数组
    pub fn as_array(&self) -> Option<&Vec<VariableValue>> {
        match self {
            VariableValue::Array(a) => Some(a),
            _ => None,
        }
    }
}

impl Default for MainConfig {
    fn default() -> Self {
        // 创建一个包含默认变量的 HashMap
        let mut default_variables = HashMap::new();
        default_variables.insert("curtain_opened".to_string(), VariableValue::Boolean(false));
        default_variables.insert("lamp_on".to_string(), VariableValue::Boolean(false));
        default_variables.insert("books_read".to_string(), VariableValue::Boolean(false));
        default_variables.insert("interactions_count".to_string(), VariableValue::Number(0.0));
        default_variables.insert("visited_locations".to_string(), VariableValue::Array(Vec::new()));
        Self {
            title: "Raven Engine".to_string(),
            assets: AssetsConfig {
                characters: HashMap::new(),
                backgrounds: HashMap::new(),
                audio: AudioConfig {
                    bgm: HashMap::new(),
                    sfx: HashMap::new(),
                    click_sound: "typing2.ogg".to_string(),
                    backclick_sound: "button.ogg".to_string(),
                },
                videos: HashMap::new(),
                swf: HashMap::new(),
            },
            settings: SettingsConfig {
                initial_scene: "intro".to_string(),
                text_speed: 50,
                auto_save: true,
                resolution: [1200, 660],
                logo_text: "Raven logo".to_string(),
                font: "fonts/GenSenMaruGothicTW-Bold.ttf".to_string(),
                rewind: false,
                resizable: true,      // 默认允许调整大小
                maximizable: true,    // 默认允许最大化
            },
            global_variables: GlobalVariables {
                player_name: "主角".to_string(),
                affection_points: 0,
            },
            variables: default_variables
        }
    }
}

impl MainConfig {
    pub fn load() -> Self {
        load_main_config()
    }

    pub fn get_window_size(&self) -> (f32, f32) {
        (self.settings.resolution[0] as f32, self.settings.resolution[1] as f32)
    }
    
    pub fn get_character_path(&self, character: &str) -> Option<&String> {
        self.assets.characters.get(character)
    }
    
    pub fn get_background_path(&self, background: &str) -> Option<&String> {
        self.assets.backgrounds.get(background)
    }
    
    // 获取窗口是否可调整大小
    pub fn is_resizable(&self) -> bool {
        self.settings.resizable
    }
    
    // 获取窗口是否可最大化
    pub fn is_maximizable(&self) -> bool {
        self.settings.maximizable
    }
}

pub fn load_main_config() -> MainConfig {
    let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let yaml_path = exe_dir.join("assets/main.yaml");
    
    println!("配置文件路径: {:?}", yaml_path);
    
    if !yaml_path.exists() {
        println!("配置文件不存在，使用默认配置");
        return MainConfig::default();
    }
    
    let yaml_str = match fs::read_to_string(&yaml_path) {
        Ok(content) => content,
        Err(e) => {
            println!("读取配置文件失败: {}, 使用默认配置", e);
            return MainConfig::default();
        }
    };
    
    match serde_yaml::from_str(&yaml_str) {
        Ok(config) => config,
        Err(e) => {
            println!("YAML解析失败: {}, 使用默认配置", e);
            MainConfig::default()
        }
    }
}
