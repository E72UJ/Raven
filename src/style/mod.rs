use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Resource, Deserialize, Debug)]
pub struct UiStyleSheet {
    #[serde(flatten)]
    pub groups: HashMap<String, HashMap<String, UiStyle>>,
    
    // 媒体查询配置
    #[serde(default)]
    pub media_queries: HashMap<String, MediaQuery>,
    
    // 当前活跃的媒体查询（运行时状态，不序列化）
    #[serde(skip, default)]
    pub active_media_queries: Vec<String>,
    
    // 调试模式开关
    #[serde(skip, default)]
    pub debug_mode: bool,
}

impl Default for UiStyleSheet {
    fn default() -> Self {
        Self {
            groups: HashMap::new(),
            media_queries: HashMap::new(),
            active_media_queries: Vec::new(),
            debug_mode: true, // 默认开启调试模式
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct UiStyle {
    background_color: Option<[f32; 4]>,
    text_color: Option<[f32; 4]>,
    font_size: Option<f32>,
    padding: Option<[f32; 4]>,
    pub position: Option<[Option<f32>; 4]>,
    margin: Option<f32>,
    border_radius: Option<f32>,
    size: Option<[f32; 2]>,
    border_color: Option<[f32; 4]>,
    border_width: Option<f32>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MediaQuery {
    pub condition: String,
    pub styles: HashMap<String, ElementStyle>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ElementStyle {
    pub font_size: Option<f32>,
    pub position: Option<[Option<f32>; 4]>, // [top, right, bottom, left]
    pub size: Option<[f32; 2]>,
    pub padding: Option<[f32; 4]>,
    pub background_color: Option<[f32; 4]>,
    pub text_color: Option<[f32; 4]>,
    pub border_color: Option<[f32; 4]>,
    pub border_width: Option<f32>,
    pub margin: Option<f32>,
    pub border_radius: Option<f32>,
}

// 响应式组件
#[derive(Component)]
pub struct ElementId(pub String);

#[derive(Component)]
pub struct ResponsiveFontSize {
    pub current_size: f32,
}

#[derive(Resource)]
pub struct WindowState {
    pub resolution: Vec2,
    pub is_fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            resolution: Vec2::new(1920.0, 1080.0),
            is_fullscreen: false,
        }
    }
}

// 样式更新触发器
#[derive(Resource, Default)]
pub struct StyleUpdateTrigger {
    pub force_update: bool,
    pub last_frame_entity_count: usize,
}

// 样式更新原因（用于调试）
#[derive(Debug)]
pub enum StyleUpdateReason {
    WindowChanged,
    NewEntities,
    ForceUpdate,
    GameStateChanged,
    InitialLoad,
}

// 媒体查询缓存
#[derive(Resource)]
pub struct MediaQueryCache {
    last_window_size: Vec2,
    last_fullscreen: bool,
    active_queries: Vec<String>,
    cache_valid: bool,
}

impl Default for MediaQueryCache {
    fn default() -> Self {
        Self {
            last_window_size: Vec2::ZERO,
            last_fullscreen: false,
            active_queries: Vec::new(),
            cache_valid: false,
        }
    }
}

impl UiStyleSheet {
    pub fn debug_print_groups(&self) {
        if !self.debug_mode { return; }
        println!("样式表中的所有分组:");
        for (group_name, _) in &self.groups {
            println!("  - {}", group_name);
        }
        println!("总共 {} 个分组", self.groups.len());
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut stylesheet: UiStyleSheet = serde_yaml::from_str(&content)?;
        stylesheet.active_media_queries = Vec::new();
        stylesheet.debug_mode = true; // 手动设置为 true
        Ok(stylesheet)
    }

    pub fn get_style(&self, group: &str, style_name: &str) -> Option<&UiStyle> {
        self.groups.get(group)?.get(style_name)
    }

    pub fn get_background_color(&self, group: &str, style_name: &str) -> Color {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(color) = style.background_color {
                return Color::srgba(color[0], color[1], color[2], color[3]);
            }
        }
        Color::NONE
    }

    pub fn get_text_color(&self, group: &str, style_name: &str) -> Color {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(color) = style.text_color {
                return Color::srgba(color[0], color[1], color[2], color[3]);
            }
        }
        Color::WHITE
    }

    pub fn get_font_size(&self, group: &str, style_name: &str) -> f32 {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(size) = style.font_size {
                return size;
            }
        }
        14.0
    }

    pub fn get_padding(&self, group: &str, style_name: &str) -> UiRect {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(padding) = style.padding {
                return UiRect {
                    left: Val::Px(padding[0]),
                    right: Val::Px(padding[1]),
                    top: Val::Px(padding[2]),
                    bottom: Val::Px(padding[3]),
                };
            }
        }
        UiRect::all(Val::Px(0.0))
    }

    pub fn get_position(&self, group: &str, style_name: &str) -> UiRect {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(position) = style.position {
                return UiRect {
                    left: position[3].map_or(Val::Auto, |v| Val::Px(v)),
                    right: position[1].map_or(Val::Auto, |v| Val::Px(v)),
                    top: position[0].map_or(Val::Auto, |v| Val::Px(v)),
                    bottom: position[2].map_or(Val::Auto, |v| Val::Px(v)),
                };
            }
        }
        UiRect::all(Val::Auto)
    }

    pub fn get_size(&self, group: &str, style_name: &str) -> Option<(Val, Val)> {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(size) = style.size {
                return Some((Val::Px(size[0]), Val::Px(size[1])));
            }
        }
        None
    }

    pub fn get_border_color(&self, group: &str, style_name: &str) -> Color {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(color) = style.border_color {
                return Color::srgba(color[0], color[1], color[2], color[3]);
            }
        }
        Color::NONE
    }

    pub fn get_border_width(&self, group: &str, style_name: &str) -> UiRect {
        if let Some(style) = self.get_style(group, style_name) {
            if let Some(width) = style.border_width {
                return UiRect::all(Val::Px(width));
            }
        }
        UiRect::all(Val::Px(0.0))
    }

    pub fn get_matching_media_queries(&self, window_size: Vec2, is_fullscreen: bool) -> Vec<(String, MediaQuery)> {
        self.media_queries
            .iter()
            .filter(|(_, query)| {
                ConditionEvaluator::evaluate(&query.condition, window_size, is_fullscreen)
            })
            .map(|(name, query)| (name.clone(), query.clone()))
            .collect()
    }
    
    pub fn debug_print_media_queries(&self) {
        if !self.debug_mode { return; }
        println!("=== 媒体查询配置 ===");
        for (name, query) in &self.media_queries {
            println!("媒体查询: {}", name);
            println!("  条件: {}", query.condition);
            println!("  样式规则数量: {}", query.styles.len());
            for (element_id, _) in &query.styles {
                println!("    - 元素ID: {}", element_id);
            }
        }
        println!("===================");
    }

    pub fn debug_print(&self) {
        if !self.debug_mode { return; }
        println!("=== UI样式表内容 ===");
        for (group_name, group_styles) in &self.groups {
            println!("分组: {}", group_name);
            for (style_name, style) in group_styles {
                println!("  样式名称: {}", style_name);
                
                if let Some(bg_color) = style.background_color {
                    println!("    背景色: [{}, {}, {}, {}]", bg_color[0], bg_color[1], bg_color[2], bg_color[3]);
                }
                
                if let Some(text_color) = style.text_color {
                    println!("    文字色: [{}, {}, {}, {}]", text_color[0], text_color[1], text_color[2], text_color[3]);
                }
                
                if let Some(font_size) = style.font_size {
                    println!("    字体大小: {}", font_size);
                }
                
                if let Some(size) = style.size {
                    println!("    尺寸: [宽:{}, 高:{}]", size[0], size[1]);
                }
                
                println!();
            }
        }
        
        self.debug_print_media_queries();
        
        println!("=== 当前活跃的媒体查询 ===");
        if self.active_media_queries.is_empty() {
            println!("无活跃的媒体查询");
        } else {
            for query_name in &self.active_media_queries {
                println!("  - {}", query_name);
            }
        }
        println!("===================");
    }

    // 手动触发样式更新
    pub fn trigger_update(&self, trigger: &mut StyleUpdateTrigger) {
        trigger.force_update = true;
    }
}

// 条件评估器
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    pub fn evaluate(condition: &str, window_size: Vec2, is_fullscreen: bool) -> bool {
        let width = window_size.x;
        let height = window_size.y;
        
        let condition = condition
            .replace("width", &width.to_string())
            .replace("height", &height.to_string())
            .replace("fullscreen", &is_fullscreen.to_string());
        
        Self::parse_condition(&condition)
    }
    
    fn parse_condition(condition: &str) -> bool {
        let and_parts: Vec<&str> = condition.split(" and ").collect();
        
        for part in and_parts {
            if !Self::evaluate_single_condition(part.trim()) {
                return false;
            }
        }
        
        true
    }
    
    fn evaluate_single_condition(condition: &str) -> bool {
        if condition.contains(">=") {
            let parts: Vec<&str> = condition.split(">=").collect();
            if parts.len() == 2 {
                let left: f32 = parts[0].trim().parse().unwrap_or(0.0);
                let right: f32 = parts[1].trim().parse().unwrap_or(0.0);
                return left >= right;
            }
        } else if condition.contains("<=") {
            let parts: Vec<&str> = condition.split("<=").collect();
            if parts.len() == 2 {
                let left: f32 = parts[0].trim().parse().unwrap_or(0.0);
                let right: f32 = parts[1].trim().parse().unwrap_or(0.0);
                return left <= right;
            }
        } else if condition.contains("<") {
            let parts: Vec<&str> = condition.split("<").collect();
            if parts.len() == 2 {
                let left: f32 = parts[0].trim().parse().unwrap_or(0.0);
                let right: f32 = parts[1].trim().parse().unwrap_or(0.0);
                return left < right;
            }
        } else if condition.contains(">") {
            let parts: Vec<&str> = condition.split(">").collect();
            if parts.len() == 2 {
                let left: f32 = parts[0].trim().parse().unwrap_or(0.0);
                let right: f32 = parts[1].trim().parse().unwrap_or(0.0);
                return left > right;
            }
        } else if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let left_str = parts[0].trim();
                let right_str = parts[1].trim();
                
                if let (Ok(left), Ok(right)) = (left_str.parse::<f32>(), right_str.parse::<f32>()) {
                    return (left - right).abs() < f32::EPSILON;
                }
                
                if let (Ok(left), Ok(right)) = (left_str.parse::<bool>(), right_str.parse::<bool>()) {
                    return left == right;
                }
                
                return left_str == right_str;
            }
        }
        
        false
    }
}

impl ElementStyle {
    pub fn to_node(&self, _window_size: Vec2) -> Node {
        let mut node = Node::default();
        
        if let Some(position) = self.position {
            node.position_type = PositionType::Absolute;
            node.top = position[0].map_or(Val::Auto, |v| Val::Px(v));
            node.right = position[1].map_or(Val::Auto, |v| Val::Px(v));
            node.bottom = position[2].map_or(Val::Auto, |v| Val::Px(v));
            node.left = position[3].map_or(Val::Auto, |v| Val::Px(v));
        }
        
        if let Some(size) = self.size {
            node.width = Val::Px(size[0]);
            node.height = Val::Px(size[1]);
        }
        
        if let Some(padding) = self.padding {
            node.padding = UiRect {
                left: Val::Px(padding[0]),
                right: Val::Px(padding[1]),
                top: Val::Px(padding[2]),
                bottom: Val::Px(padding[3]),
            };
        }
        
        node
    }

    pub fn to_background_color(&self) -> Option<Color> {
        self.background_color.map(|color| Color::srgba(color[0], color[1], color[2], color[3]))
    }

    pub fn to_text_color(&self) -> Option<Color> {
        self.text_color.map(|color| Color::srgba(color[0], color[1], color[2], color[3]))
    }

    #[allow(dead_code)]
    pub fn to_border_color(&self) -> Option<Color> {
        self.border_color.map(|color| Color::srgba(color[0], color[1], color[2], color[3]))
    }
}

// 窗口状态更新系统
pub fn update_window_state(
    mut window_state: ResMut<WindowState>,
    q_windows: Query<&Window>,
) {
    for window in q_windows.iter() {
        let new_resolution = Vec2::new(window.resolution.width(), window.resolution.height());
        let new_fullscreen = matches!(window.mode, bevy::window::WindowMode::Fullscreen(..));
        
        if window_state.resolution != new_resolution || window_state.is_fullscreen != new_fullscreen {
            window_state.resolution = new_resolution;
            window_state.is_fullscreen = new_fullscreen;
        }
    }
}

// 改进的媒体查询样式更新系统
pub fn update_styles_from_media_queries(
    mut ui_query: Query<(&mut Node, &ElementId, Option<&mut BackgroundColor>), Without<TextFont>>,
    mut text_query: Query<(&mut TextFont, &mut ResponsiveFontSize, &ElementId, Option<&mut Node>, Option<&mut TextColor>)>,
    window_state: Res<WindowState>,
    mut cache: ResMut<MediaQueryCache>,
    mut trigger: ResMut<StyleUpdateTrigger>,
    mut stylesheet: ResMut<UiStyleSheet>,
) {
    let mut update_reason = None;
    
    // 检查各种触发条件
    let window_changed = window_state.is_changed();
    let force_update = trigger.force_update;
    
    // 检查是否有新的UI元素
    let current_entity_count = ui_query.iter().len() + text_query.iter().len();
    let new_entities = current_entity_count != trigger.last_frame_entity_count;
    
    // 检查缓存有效性
    let cache_invalid = !cache.cache_valid;
    
    // 决定是否需要更新
    let should_update = window_changed || force_update || new_entities || cache_invalid;
    
    if !should_update {
        return;
    }
    
    // 确定更新原因（用于调试）
    if window_changed {
        update_reason = Some(StyleUpdateReason::WindowChanged);
    } else if force_update {
        update_reason = Some(StyleUpdateReason::ForceUpdate);
    } else if new_entities {
        update_reason = Some(StyleUpdateReason::NewEntities);
    } else if cache_invalid {
        update_reason = Some(StyleUpdateReason::InitialLoad);
    }
    
    if stylesheet.debug_mode {
        if let Some(reason) = update_reason {
            println!("🔄 样式更新触发: {:?}", reason);
        }
    }
    
    // 重置触发器
    trigger.force_update = false;
    trigger.last_frame_entity_count = current_entity_count;
    
    // 更新缓存
    if window_changed {
        cache.last_window_size = window_state.resolution;
        cache.last_fullscreen = window_state.is_fullscreen;
    }
    cache.cache_valid = true;
    
    // 获取匹配的媒体查询
    let matching_queries = stylesheet.get_matching_media_queries(
        window_state.resolution, 
        window_state.is_fullscreen
    );
    
    // 检查活跃的媒体查询是否有变化
    let current_active_queries: Vec<String> = matching_queries.iter()
        .map(|(name, _)| name.clone())
        .collect();
    
    if current_active_queries != stylesheet.active_media_queries {
        if stylesheet.debug_mode {
            println!("🔄 媒体查询变更:");
            println!("  之前: {:?}", stylesheet.active_media_queries);
            println!("  现在: {:?}", current_active_queries);
            println!("  窗口大小: {}x{}, 全屏: {}", 
                window_state.resolution.x, window_state.resolution.y, window_state.is_fullscreen);
        }
        stylesheet.active_media_queries = current_active_queries;
    }
    
    if matching_queries.is_empty() {
        if stylesheet.debug_mode {
            println!("⚠️  没有匹配的媒体查询 (窗口: {}x{})", 
                window_state.resolution.x, window_state.resolution.y);
        }
        return;
    }
    
    // 收集所有匹配的样式
    let mut matched_styles: HashMap<String, ElementStyle> = HashMap::new();
    let mut applied_elements: Vec<String> = Vec::new();
    
    for (query_name, media_query) in &matching_queries {
        if stylesheet.debug_mode {
            println!("✅ 应用媒体查询: {} (条件: {})", query_name, media_query.condition);
        }
        
        for (element_id, style) in &media_query.styles {
            matched_styles.insert(element_id.clone(), style.clone());
            applied_elements.push(element_id.clone());
        }
    }
    
    if !applied_elements.is_empty() && stylesheet.debug_mode {
        println!("📝 准备应用样式到 {} 个元素: {:?}", applied_elements.len(), applied_elements);
    }
    
    // 应用样式到纯 UI 节点
    let mut ui_elements_updated = 0;
    for (mut node, element_id, mut bg_color_option) in ui_query.iter_mut() {
        if let Some(style) = matched_styles.get(&element_id.0) {
            if stylesheet.debug_mode {
                println!("🎨 应用 UI 样式到元素: {}", element_id.0);
            }
            apply_node_style(&mut node, style, &window_state.resolution, &element_id.0, stylesheet.debug_mode);
            
            // 应用背景色
            if let (Some(ref mut bg_color), Some(color)) = (bg_color_option.as_mut(), style.to_background_color()) {
                bg_color.0 = color;
                if stylesheet.debug_mode {
                    println!("  🌈 应用背景色: {:?}", color);
                }
            }
            ui_elements_updated += 1;
        }
    }
    
    // 应用样式到文本元素
    let mut text_elements_updated = 0;
    for (mut text_font, mut responsive_font, element_id, mut node_option, mut text_color_option) in text_query.iter_mut() {
        if let Some(style) = matched_styles.get(&element_id.0) {
            if stylesheet.debug_mode {
                println!("📝 应用文本样式到元素: {}", element_id.0);
            }
            
            // 应用字体大小
            if let Some(font_size) = style.font_size {
                text_font.font_size = font_size;
                responsive_font.current_size = font_size;
                if stylesheet.debug_mode {
                    println!("  📏 字体大小: {}", font_size);
                }
            }
            
            // 应用文本颜色
            if let (Some(ref mut text_color), Some(color)) = (text_color_option.as_mut(), style.to_text_color()) {
                text_color.0 = color;
                if stylesheet.debug_mode {
                    println!("  🌈 文本颜色: {:?}", color);
                }
            }
            
            // 如果文本元素也有 Node，应用位置样式
            if let Some(ref mut node) = node_option {
                apply_node_style(node, style, &window_state.resolution, &element_id.0, stylesheet.debug_mode);
            }
            text_elements_updated += 1;
        }
    }
    
    if stylesheet.debug_mode && (ui_elements_updated > 0 || text_elements_updated > 0) {
        println!("✨ 样式更新完成: {} 个UI元素, {} 个文本元素", ui_elements_updated, text_elements_updated);
    }
}

fn apply_node_style(node: &mut Node, style: &ElementStyle, window_size: &Vec2, element_id: &str, debug_mode: bool) {
    let new_node = style.to_node(*window_size);
    
    if style.size.is_some() {
        node.width = new_node.width;
        node.height = new_node.height;
        if debug_mode {
            println!("  📐 应用尺寸到 {}: {}x{}", element_id, 
                if let Val::Px(w) = new_node.width { w } else { 0.0 },
                if let Val::Px(h) = new_node.height { h } else { 0.0 });
        }
    }
    if style.position.is_some() {
        node.position_type = new_node.position_type;
        node.left = new_node.left;
        node.right = new_node.right;
        node.top = new_node.top;
        node.bottom = new_node.bottom;
        
        if debug_mode {
            println!("  📍 应用位置到 {}: left={:?}, top={:?}, right={:?}, bottom={:?}", 
                element_id, new_node.left, new_node.top, new_node.right, new_node.bottom);
        }
    }
    if style.padding.is_some() {
        node.padding = new_node.padding;
        if debug_mode {
            println!("  📦 应用内边距到 {}: {:?}", element_id, new_node.padding);
        }
    }
}

// 样式加载系统
pub fn load_styles(mut stylesheet: ResMut<UiStyleSheet>) {
    let program_dir = match std::env::current_exe() {
        Ok(exe_path) => {
            let mut dir = exe_path;
            dir.pop();
            dir
        }
        Err(e) => {
            println!("⚠️  无法获取程序路径: {}", e);
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        }
    };
    
    println!("📁 程序目录: {:?}", program_dir);
    let style_file_path = program_dir.join("assets").join("style.yaml");
    println!("📄 样式文件路径: {:?}", style_file_path);
    println!("🔍 文件是否存在: {}", style_file_path.exists());
    
    match UiStyleSheet::load_from_file(style_file_path.to_str().unwrap()) {
        Ok(loaded_stylesheet) => {
            println!("✅ 样式表加载成功！");
            *stylesheet = loaded_stylesheet;
            stylesheet.debug_print();
        }
        Err(e) => {
            println!("❌ 加载样式表失败: {}", e);
            *stylesheet = UiStyleSheet::default();
        }
    }
}

// 手动触发样式更新的系统
pub fn force_style_update(mut trigger: ResMut<StyleUpdateTrigger>) {
    trigger.force_update = true;
}

// 通用的状态变化触发器
pub fn on_state_changed(mut trigger: ResMut<StyleUpdateTrigger>) {
    trigger.force_update = true;
}

pub struct StylePlugin;

impl Plugin for StylePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UiStyleSheet>()
            .init_resource::<WindowState>()
            .init_resource::<MediaQueryCache>()
            .init_resource::<StyleUpdateTrigger>()
            .add_systems(Startup, (load_styles, force_style_update).chain())
            .add_systems(Update, update_window_state)
            .add_systems(Update, update_styles_from_media_queries.after(update_window_state));
    }
}
