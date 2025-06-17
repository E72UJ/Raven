use bevy::{
    color::palettes::basic::*, // 导入基础颜色调色板
    ecs::relationship::RelatedSpawnerCommands, // 导入ECS关系相关的命令
    prelude::*, // 导入Bevy的常用类型和函数
    winit::WinitSettings, // 导入窗口设置
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // 添加Bevy默认插件集合(窗口、渲染、输入等)
        // 只在有用户输入时运行应用，以减少CPU/GPU使用
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup) // 添加启动时执行的系统
        .add_systems(Update, button_system) // 添加每帧更新时执行的系统
        .run(); // 运行应用
}

// 定义按钮的三种状态颜色
const NORMAL_BUTTON: Color = Color::srgb(0.75, 0.15, 0.15); // 正常状态：深红色
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25); // 悬停状态：深灰色
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35); // 按下状态：绿色

// 按钮交互系统：处理按钮的点击、悬停等交互
fn button_system(
    // 查询所有交互状态发生变化的按钮
    mut interaction_query: Query<
        (
            &Interaction,           // 交互状态组件
            &mut BackgroundColor,   // 可变的背景颜色组件
            &mut BorderColor,       // 可变的边框颜色组件
            &Children,              // 子实体组件
            &Name,                  // 名称组件
        ),
        (Changed<Interaction>, With<Button>), // 过滤器：只查询交互状态变化且有Button组件的实体
    >,
    // 查询文本组件
    mut text_query: Query<&mut Text>,
) {
    // 遍历所有满足条件的按钮
    for (interaction, mut color, mut border_color, children, name) in &mut interaction_query {
        // 获取按钮的文本子实体（这里没有实际使用，所以加下划线前缀）
        let _text = text_query.get_mut(children[0]).unwrap();
        
        // 根据交互状态更新按钮外观
        match *interaction {
            Interaction::Pressed => {
                // 按钮被按下时
                *color = PRESSED_BUTTON.into(); // 设置背景为绿色
                border_color.0 = RED.into();    // 设置边框为红色
                println!("按下了: {}", name.as_str()); // 在控制台输出按钮名称
            }
            Interaction::Hovered => {
                // 鼠标悬停在按钮上时
                *color = HOVERED_BUTTON.into(); // 设置背景为深灰色
                border_color.0 = Color::WHITE;  // 设置边框为白色
            }
            Interaction::None => {
                // 无交互状态（正常状态）
                *color = NORMAL_BUTTON.into();  // 设置背景为深红色
                border_color.0 = Color::BLACK;  // 设置边框为黑色
            }
        }
    }
}

// 设置系统：创建UI界面
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 生成2D UI相机
    commands.spawn(Camera2d);

    // 创建主容器：占满整个屏幕的垂直布局容器
    commands
        .spawn(Node {
            width: Val::Percent(100.0),                           // 宽度100%
            height: Val::Percent(100.0),                          // 高度100%
            flex_direction: FlexDirection::Column,                // 垂直布局
            justify_content: JustifyContent::SpaceBetween,        // 子元素之间分布
            ..default()
        })
        .with_children(|parent| {
            // 主内容区域：占用90%的高度，目前为空
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),  // 宽度100%
                    height: Val::Percent(90.0),  // 高度90%
                    ..default()
                },
                BackgroundColor(Color::NONE), // 透明背景
            ));

            // 底部导航栏：占用10%的高度
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),                      // 宽度100%
                        height: Val::Percent(10.0),                      // 高度10%
                        flex_direction: FlexDirection::Row,              // 水平布局
                        justify_content: JustifyContent::SpaceEvenly,    // 子元素均匀分布
                        align_items: AlignItems::Center,                 // 垂直居中对齐
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)), // 半透明深色背景
                ))
                .with_children(|parent| {
                    // 创建所有导航按钮
                    let nav_items = vec!["主菜单", "保存", "读取", "设置", "历史", "跳过", "自动"];

                    // 遍历按钮标签，为每个标签创建一个按钮
                    for item in nav_items {
                        create_nav_button(parent, &asset_server, item);
                    }
                });
        });
}

// 创建导航按钮的辅助函数
fn create_nav_button(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>, // 父容器的命令
    asset_server: &Res<AssetServer>,                  // 资源服务器，用于加载字体
    label: &str,                                      // 按钮标签文本
) {
    parent
        .spawn((
            Button,                                    // 按钮组件
            Node {
                width: Val::Px(80.0),                 // 按钮宽度80像素
                height: Val::Px(40.0),                // 按钮高度40像素
                border: UiRect::all(Val::Px(2.0)),    // 四周边框2像素
                justify_content: JustifyContent::Center, // 内容水平居中
                align_items: AlignItems::Center,       // 内容垂直居中
                ..default()
            },
            BorderColor(Color::BLACK),                 // 边框颜色为黑色
            BorderRadius::all(Val::Px(5.0)),          // 圆角半径5像素
            BackgroundColor(NORMAL_BUTTON),           // 背景颜色为正常状态颜色
            Name::new(label.to_string()),             // 给按钮命名，用于识别
        ))
        .with_child((
            // 按钮内的文本
            Text::new(label),                         // 文本内容
            TextFont {
                font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"), // 加载自定义字体
                font_size: 16.0,                      // 字体大小16像素
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),   // 文本颜色为浅灰色
        ));
}