use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    env,
    fs::File,
    io::{BufWriter, Cursor},
};

use anyhow::Result;
use bitmap::CompressedBitmap;
use decode::decode_define_bits_jpeg_dimensions;
use parse_shape::matrix::Matrix;
use serde::{Deserialize, Serialize};
use swf::{CharacterId, DefineBitsLossless, Depth, Encoding, PlaceObject, Shape, SwfStr, Tag};
use swf_derive::KeyFrame;
use types::{BlendMode, Filter};

pub mod bitmap;
mod decode;
pub mod parse_shape;
pub mod types;

/// 动画版本号
/// 这里的版本号是从Cargo.toml中获取的，表示当前动画解析器的版本
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 新格式动画原信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub frame_rate: f32,
    pub frames: u16,
    pub version: String,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            frame_rate: 0.0,
            frames: 0,
            version: String::from(VERSION),
        }
    }
}

pub trait KeyFrame {
    fn time(&self) -> f32;
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, KeyFrame)]
pub struct Transform {
    pub time: f32,
    pub matrix: Matrix,
}

impl Transform {
    fn new(
        time: f32,
        a: swf::Fixed16,
        b: swf::Fixed16,
        c: swf::Fixed16,
        d: swf::Fixed16,
        tx: swf::Twips,
        ty: swf::Twips,
    ) -> Self {
        Self {
            time,
            matrix: Matrix {
                a: a.to_f32(),
                b: b.to_f32(),
                c: c.to_f32(),
                d: d.to_f32(),
                tx,
                ty,
            },
        }
    }
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ColorTransform {
    pub mult_color: [f32; 4],
    pub add_color: [i16; 4],
    #[serde(skip)]
    pub color_transform: swf::ColorTransform,
}

// TODO: 手动实现序列化和反序列化
// impl<'de> Deserialize<'de> for ColorTransform {
//     fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         str
//     }
// }

impl ColorTransform {
    fn new(
        color_transform: swf::ColorTransform,
        mult_color: [f32; 4],
        add_color: [i16; 4],
    ) -> Self {
        Self {
            mult_color,
            add_color,
            color_transform,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, KeyFrame)]
pub struct Placement {
    time: f32,
    resource_id: Option<CharacterId>,
    blend_mode: BlendMode,
    color_transform: ColorTransform,
    filters: Vec<Filter>,
}

impl Placement {
    pub fn resource_id(&self) -> Option<CharacterId> {
        self.resource_id
    }

    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub fn color_transform(&self) -> ColorTransform {
        self.color_transform
    }

    pub fn filters(&self) -> &Vec<Filter> {
        &self.filters
    }
}

impl Placement {
    fn new(time: f32, resource_id: Option<CharacterId>) -> Self {
        Self {
            time,
            resource_id,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct DepthTimeline {
    pub placement: Vec<Placement>, // 资源变化
    pub transforms: Vec<Transform>,
}

impl DepthTimeline {
    pub fn new(time: f32, resource_id: CharacterId) -> Self {
        Self {
            placement: vec![Placement::new(time, Some(resource_id))],
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub time: f32,
    pub name: String,
}

impl Event {
    fn new(time: f32, name: String) -> Self {
        Self { time, name }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub timeline: BTreeMap<Depth, DepthTimeline>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Event>,
}
impl Animation {
    fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct MovieClip {
    name: Option<String>,
    id: CharacterId,
    duration: f32,
    timeline: BTreeMap<Depth, DepthTimeline>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    skin_frames: HashMap<String, u32>,
    #[serde(skip_serializing_if = "String::is_empty")]
    default_skin: String,
    #[serde(skip_serializing)]
    pub current_time: f32,
}
impl MovieClip {
    fn new(id: CharacterId, duration: f32) -> Self {
        Self {
            id,
            duration,
            ..Default::default()
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn timeline(&self) -> &BTreeMap<Depth, DepthTimeline> {
        &self.timeline
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn is_skin_frame(&self) -> bool {
        !self.skin_frames.is_empty()
    }

    pub fn default_skin_frame(&self) -> u32 {
        *self
            .skin_frames
            .get(&self.default_skin)
            .expect("找不到就是有鬼")
    }

    pub fn skin_frame(&self, name: &str) -> Option<&u32> {
        self.skin_frames.get(name)
    }

    pub fn skin_frames(&self) -> &HashMap<String, u32> {
        &self.skin_frames
    }
}

/// 新格式动画数据
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Animations {
    /// 动画元数据
    pub meta: Meta,
    /// 资源定义
    pub children_clip: HashMap<CharacterId, MovieClip>,
    /// Key为动画名称，Value为动画数据
    pub animations: HashMap<String, Animation>,
}

impl Animations {
    pub fn new(meta: Meta) -> Self {
        Self {
            meta,
            ..Default::default()
        }
    }
}

/// 解析flash动画为新格式，方便集成到游戏引擎中
/// 接收`swf`文件二进制数据
pub fn parse_flash_animation(
    data: Vec<u8>,
) -> Result<(
    Animations,
    HashMap<CharacterId, Shape>,
    HashMap<CharacterId, CompressedBitmap>,
)> {
    // 将二进制数据转换为字节流
    let cursor = Cursor::new(data);
    let swf_buf = swf::decompress_swf(cursor)?;
    let swf = swf::parse_swf(&swf_buf)?;
    let tags = swf.tags;

    // 获取flash动画的帧率和总帧数
    let header = &swf.header;
    let frame_rate = header.frame_rate().to_f32();
    let frames = header.num_frames();
    let swf_encoding = SwfStr::encoding_for_version(header.version());

    let meta = Meta {
        frame_rate,
        frames,
        ..Default::default()
    };

    let mut shapes = HashMap::new();
    let mut bitmaps = HashMap::new();
    // 解析动画数据
    let mut animations = Animations::new(meta);
    parse_animation_data(
        &mut animations,
        &mut shapes,
        &mut bitmaps,
        tags,
        frame_rate,
        swf_encoding,
    );

    Ok((animations, shapes, bitmaps))
}

/// 输出动画数据到json文件
pub fn output_json(
    animations: &Animations,
    is_pretty: bool,
    file_name: &str,
    output: &str,
) -> Result<()> {
    // 将动画数据写入文件
    if output.is_empty() {
        let path = env::current_dir()?.join(format!("{}.json", file_name));
        let writer = BufWriter::new(File::create(path)?);
        // 是否格式化输出
        if is_pretty {
            serde_json::to_writer_pretty(writer, animations)?;
        } else {
            serde_json::to_writer(writer, animations)?;
        }
    }
    Ok(())
}

/// 解析子动画
fn parse_sprite_animation(
    sprite: swf::Sprite<'_>,
    frame_rate: f32,
    children_clip: &mut HashMap<CharacterId, MovieClip>,
    swf_encoding: &'static Encoding,
) {
    let mut movie_clip = MovieClip::new(sprite.id, sprite.num_frames as f32 / frame_rate);
    let mut current_frame: u32 = 0;
    for tag in sprite.tags {
        let time = current_frame as f32 / frame_rate;
        match tag {
            Tag::PlaceObject(place_object) => {
                parse_place_object(
                    &mut movie_clip.timeline,
                    &place_object,
                    time,
                    children_clip,
                    swf_encoding,
                );
            }
            Tag::RemoveObject(remove_object) => {
                remove_at_depth(&mut movie_clip.timeline, remove_object.depth, time);
            }
            Tag::ShowFrame => {
                current_frame += 1;
            }
            Tag::FrameLabel(frame_label) => {
                let label = frame_label.label.to_string_lossy(swf_encoding);
                parse_sprite_label(label, current_frame, &mut movie_clip);
            }
            _ => {}
        }
    }
    children_clip.insert(sprite.id, movie_clip);
}

// 皮肤定义clip，将每一帧作为一个皮肤资源处理
fn parse_sprite_label(label: String, current_frame: u32, movie_clip: &mut MovieClip) {
    if label.starts_with("skin_") {
        let label = label.trim_start_matches("skin_").to_owned();
        if current_frame == 0 {
            movie_clip.default_skin = label.clone();
        }
        movie_clip.skin_frames.insert(label, current_frame);
    }
}

/// 解析动画数据
fn parse_animation_data(
    animations: &mut Animations,
    shapes: &mut HashMap<CharacterId, Shape>,
    bitmaps: &mut HashMap<CharacterId, CompressedBitmap>,
    tags: Vec<Tag<'_>>,
    frame_rate: f32,
    swf_encoding: &'static Encoding,
) {
    let mut current_frame: u32 = 0;
    let mut current_animation_name = String::from("default"); // 默认动画名称
    let mut time: f32;
    for tag in tags {
        // 将当前帧数转换为时间，单位为秒
        time = current_frame as f32 / frame_rate;
        match tag {
            Tag::DefineShape(shape) => {
                shapes.insert(shape.id, shape);
            }
            Tag::DefineBitsJpeg3(jpeg_data) => {
                let (width, height) = decode_define_bits_jpeg_dimensions(jpeg_data.data).unwrap();
                bitmaps.insert(
                    jpeg_data.id,
                    CompressedBitmap::Jpeg {
                        data: jpeg_data.data.to_vec(),
                        alpha: Some(jpeg_data.alpha_data.to_vec()),
                        width,
                        height,
                    },
                );
            }
            Tag::DefineBitsLossless(bit_loss_less) => {
                bitmaps.insert(
                    bit_loss_less.id,
                    CompressedBitmap::Lossless(DefineBitsLossless {
                        version: bit_loss_less.version,
                        id: bit_loss_less.id,
                        format: bit_loss_less.format,
                        width: bit_loss_less.width,
                        height: bit_loss_less.height,
                        data: Cow::Owned(bit_loss_less.data.clone().into_owned()),
                    }),
                );
            }
            Tag::DefineSprite(sprite) => {
                // 解析子动画为引用资源
                parse_sprite_animation(
                    sprite,
                    frame_rate,
                    &mut animations.children_clip,
                    swf_encoding,
                );
            }
            Tag::PlaceObject(place_object) => {
                // 获取当前动画
                let animation = animations
                    .animations
                    .entry(current_animation_name.clone())
                    .or_insert(Animation::new(current_animation_name.clone()));
                let children_clip = &mut animations.children_clip;
                parse_place_object(
                    &mut animation.timeline,
                    &place_object,
                    time,
                    children_clip,
                    swf_encoding,
                );
            }
            Tag::RemoveObject(remove_object) => {
                if let Some(animation) = animations.animations.get_mut(&current_animation_name) {
                    remove_at_depth(&mut animation.timeline, remove_object.depth, time);
                }
            }
            Tag::ShowFrame => {
                current_frame += 1;
            }
            Tag::FrameLabel(frame_label) => {
                let label = frame_label.label.to_string_lossy(swf_encoding);
                parse_label(
                    &mut animations.animations,
                    &label,
                    &mut current_animation_name,
                    time,
                    &mut current_frame,
                );
            }
            // 其余的都是非动画数据
            _ => {}
        }
    }

    time = current_frame as f32 / frame_rate;
    // 最后一个动画解析完成，为最后一个动画加上duration
    if let Some(animation) = animations.animations.get_mut(&current_animation_name) {
        animation.duration = time
    }
}

fn parse_place_object(
    timeline: &mut BTreeMap<u16, DepthTimeline>,
    place_object: &PlaceObject,
    time: f32,
    children_clip: &mut HashMap<CharacterId, MovieClip>,
    swf_encoding: &'static Encoding,
) {
    match place_object.action {
        swf::PlaceObjectAction::Place(id) => {
            // 这里的id是会指向同一个definition资源对象的，所以对多次引用的childClip，会出现名称覆盖的情况，
            // 我想这种复用的childClip 并不需要指定实例名
            // 记录子影片实例名，
            if let Some(clip_name) = place_object.name {
                let name = clip_name.to_string_lossy(swf_encoding);
                if let Some(child_clip) = children_clip.get_mut(&id) {
                    child_clip.name = Some(name);
                }
            }
            let depth_timeline = timeline.entry(place_object.depth).or_default();

            if let Some(last) = depth_timeline.placement.pop() {
                if last.resource_id().is_some() || last.time != time {
                    depth_timeline.placement.push(last);
                }
            }
            let mut placement = Placement::new(time, Some(id));
            apply_place_object(depth_timeline, &mut placement, place_object, time);
            depth_timeline.placement.push(placement);
        }
        swf::PlaceObjectAction::Modify => {
            // 修改对象
            if let Some(depth_timeline) = timeline.get_mut(&place_object.depth) {
                let mut placement = depth_timeline.placement.last_mut().unwrap().clone();
                placement.time = time;
                apply_place_object(depth_timeline, &mut placement, place_object, time);
                depth_timeline.placement.push(placement);
            }
        }
        swf::PlaceObjectAction::Replace(id) => {
            if let Some(depth_timeline) = timeline.get_mut(&place_object.depth) {
                let mut placement = depth_timeline.placement.last_mut().unwrap().clone();
                placement.time = time;
                placement.resource_id = Some(id);
                apply_place_object(depth_timeline, &mut placement, place_object, time);
                depth_timeline.placement.push(placement);
            }
        }
    }
}

/// flash的标签将在转换中发挥重要作用，自定义多动画、标记事件等
fn parse_label(
    animations: &mut HashMap<String, Animation>,
    label: &str,
    current_animation_name: &mut String,
    time: f32,
    current_frame: &mut u32,
) {
    if label.starts_with("anim_") {
        // if label.is_ascii() {
        // 计算出当前动画的时长
        if let Some(animation) = animations.get_mut(current_animation_name) {
            animation.duration = time
        }

        // 这是一个新动画标签，当前帧置为0
        *current_frame = 0;
        // 去掉定义前缀
        let animation_name = label.trim_start_matches("anim_");
        // 如果动画名称已经存在，则提示错误，定义了相同名字的动画
        if animations.contains_key(animation_name) {
            eprintln!("Error: Duplicate animation name: {}", animation_name);
            return;
        }
        // 记录当前正在运行的动画名
        *current_animation_name = animation_name.to_owned();
        // 创建新的动画数据
        animations.insert(
            current_animation_name.clone(),
            Animation::new(current_animation_name.clone()),
        );
    }
    if label.starts_with("event_") {
        // 读取到事件标签时，当前动画可能还没有初始化
        let animation = animations
            .entry(current_animation_name.clone())
            .or_insert(Animation::new(current_animation_name.to_owned()));
        let event_name = label.trim_start_matches("event_");
        animation
            .events
            .push(Event::new(time, event_name.to_owned()));
    }
    // 这里可以添加更多的解析逻辑
}

fn remove_at_depth(timeline: &mut BTreeMap<Depth, DepthTimeline>, depth: Depth, time: f32) {
    // 删除指定深度的对象
    if let Some(depth_timeline) = timeline.get_mut(&depth) {
        depth_timeline.placement.push(Placement::new(time, None));
    }
}

fn apply_place_object(
    depth_timeline: &mut DepthTimeline,
    placement: &mut Placement,
    place_object: &PlaceObject,
    current_time: f32,
) {
    if let Some(matrix) = place_object.matrix {
        depth_timeline.transforms.push(Transform::new(
            current_time,
            matrix.a,
            matrix.b,
            matrix.c,
            matrix.d,
            matrix.tx,
            matrix.ty,
        ));
    }
    if let Some(color_transform) = place_object.color_transform {
        // 处理颜色变换
        placement.color_transform = ColorTransform::new(
            color_transform,
            [
                color_transform.r_multiply.to_f32(),
                color_transform.g_multiply.to_f32(),
                color_transform.b_multiply.to_f32(),
                color_transform.a_multiply.to_f32(),
            ],
            [
                color_transform.r_add,
                color_transform.g_add,
                color_transform.b_add,
                color_transform.a_add,
            ],
        );
    }
    // TODO: 需要合并到transform 否则会会无法判断是否由混合模式
    if let Some(blend_mode) = place_object.blend_mode {
        // 处理混合模式
        placement.blend_mode = blend_mode.into();
    }

    if let Some(filters) = &place_object.filters {
        // 处理滤镜变换
        placement.filters = filters.iter().map(Filter::from).collect();
    }
}
