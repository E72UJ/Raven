use filter::Filter;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use swf::{CharacterId, Depth, Encoding, Tag};
mod filter;

#[derive(Serialize, Deserialize, Debug)]
pub struct VectorAnimation {
    name: String,
    frame_rate: u16,
    shape_transform: BTreeMap<CharacterId, (f32, f32)>,
    base_animations: HashMap<CharacterId, Animation>,
    animations: BTreeMap<String, Animation>,
}

impl VectorAnimation {
    pub fn new(name: String, frame_rate: u16) -> Self {
        Self {
            name,
            frame_rate,
            shape_transform: BTreeMap::new(),
            base_animations: HashMap::new(),
            animations: BTreeMap::new(),
        }
    }
    pub fn add_animation(&mut self, label: &String, animation: Animation) {
        self.animations.insert(label.to_owned(), animation);
    }
    pub fn animation(&mut self, label: &String) -> &mut Animation {
        self.animations.entry(label.to_owned()).or_default()
    }
    fn register_base_animation(&mut self, id: CharacterId, animation: Animation) {
        self.base_animations.insert(id, animation);
    }
}
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Animation {
    timelines: BTreeMap<Depth, Vec<Frame>>,
    total_frames: u16,
}
impl Animation {
    fn timelines(&mut self) -> &mut BTreeMap<Depth, Vec<Frame>> {
        &mut self.timelines
    }
    fn timeline(&mut self, depth: Depth) -> &mut Vec<Frame> {
        self.timelines.get_mut(&depth).unwrap()
    }
    fn insert_or_get_timeline(&mut self, depth: Depth) -> &mut Vec<Frame> {
        self.timelines.entry(depth).or_default()
    }
    fn set_total_frame(&mut self, total_frame: u16) {
        self.total_frames = total_frame;
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Frame {
    id: CharacterId,
    place_frame: u16,
    duration: u16,
    transform: Transform,
    blend_mode: BlendMode,
    filters: Vec<Filter>,
}

impl Frame {
    pub fn new(id: CharacterId, place_frame: u16) -> Self {
        Self {
            id,
            place_frame,
            ..Default::default()
        }
    }
    // 用于补齐空帧
    fn space_frame(duration: u16) -> Self {
        Self {
            duration,
            ..Default::default()
        }
    }
    pub fn auto_add(&mut self) {
        self.duration += 1;
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Transform {
    matrix: Matrix,
    color_transform: ColorTransform,
}
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f32,
    pub ty: f32,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ColorTransform {
    mult_color: [f32; 4],
    add_color: [f32; 4],
}

impl Default for ColorTransform {
    fn default() -> Self {
        Self {
            mult_color: [1.0, 1.0, 1.0, 1.0],
            add_color: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub enum BlendMode {
    #[default]
    Normal,
    Add,
    Multiply,
    Layer,
    Screen,
    Lighten,
    Darken,
    Difference,
    Subtract,
    Invert,
    Alpha,
    Erase,
    Overlay,
    HardLight,
}

impl BlendMode {
    fn from_swf_blend_mode(blend_mode: swf::BlendMode) -> Self {
        match blend_mode {
            swf::BlendMode::Normal => Self::Normal,
            swf::BlendMode::Add => Self::Add,
            swf::BlendMode::Multiply => Self::Multiply,
            swf::BlendMode::Layer => Self::Layer,
            swf::BlendMode::Screen => Self::Screen,
            swf::BlendMode::Lighten => Self::Lighten,
            swf::BlendMode::Darken => Self::Darken,
            swf::BlendMode::Difference => Self::Difference,
            swf::BlendMode::Subtract => Self::Subtract,
            swf::BlendMode::Invert => Self::Invert,
            swf::BlendMode::Alpha => Self::Alpha,
            swf::BlendMode::Erase => Self::Erase,
            swf::BlendMode::Overlay => Self::Overlay,
            swf::BlendMode::HardLight => Self::HardLight,
        }
    }
}

// 生成动画文件
pub fn generation_animation(
    tags: Vec<Tag>,
    shape_transform: BTreeMap<CharacterId, (f32, f32)>,
    file_name: &str,
    output: &Path,
    frame_rate: u16,
    encoding_for_version: &'static Encoding,
) -> anyhow::Result<()> {
    let file_name: Vec<&str> = file_name.split(".").collect();
    let mut vector_animation =
        VectorAnimation::new(file_name.first().unwrap().to_string(), frame_rate);
    vector_animation.shape_transform = shape_transform;
    tags.iter().for_each(|tag| {
        if let Tag::DefineSprite(sprite) = tag {
            parse_sprite_animation_as_base_animation(&mut vector_animation, sprite);
        }
    });

    parse_animation(&mut vector_animation, tags, encoding_for_version);

    // 清除空白帧
    clear_blank_frame(&mut vector_animation);
    // 写入animation.json文件
    let writer = BufWriter::new(File::create(
        output.join(format!("{}.json", file_name.first().unwrap())),
    )?);
    // 二进制格式写入animation.an文件
    serde_json::to_writer(writer, &vector_animation)?;
    let mut buf = Vec::new();
    // 1.
    // vector_animation.serialize(&mut Serializer::new(&mut buf))?;
    // 2. 更高效？
    let mut serializer =
        rmp_serde::Serializer::new(&mut buf).with_bytes(rmp_serde::config::BytesMode::ForceAll);
    // 写入animation.an文件
    vector_animation.serialize(&mut serializer)?;
    File::create(output.join("animation.an"))?.write_all(&buf)?;
    Ok(())
}

fn parse_animation(
    vector_animation: &mut VectorAnimation,
    tags: Vec<Tag>,
    encoding_for_version: &'static Encoding,
) {
    let mut animation_name = String::from("");
    let mut current_frame = 0;
    for tag in tags {
        match tag {
            Tag::PlaceObject(place_object) => match place_object.action {
                swf::PlaceObjectAction::Place(id) => {
                    //  保证第一帧有标签区分不同的动画，如果没有设置则视为只有一个动画，将其命名为default
                    if animation_name.is_empty() {
                        animation_name = String::from("default");
                        vector_animation.add_animation(&animation_name, Animation::default());
                    }
                    add_timeline(
                        vector_animation.animation(&animation_name),
                        &place_object,
                        id,
                        current_frame,
                    );
                }
                swf::PlaceObjectAction::Modify => {
                    modify_at_depth(
                        vector_animation.animation(&animation_name),
                        &place_object,
                        current_frame,
                    );
                }
                swf::PlaceObjectAction::Replace(id) => {
                    replace_at_depth(
                        vector_animation
                            .animation(&animation_name)
                            .timeline(place_object.depth),
                        &place_object,
                        id,
                        current_frame,
                    );
                }
            },
            Tag::RemoveObject(remove_object) => {
                remove_at_depth(
                    vector_animation
                        .animation(&animation_name)
                        .timeline(remove_object.depth),
                );
            }
            Tag::ShowFrame => {
                current_frame += 1;
                vector_animation
                    .animation(&animation_name)
                    .timelines()
                    .iter_mut()
                    .for_each(|(_, timeline)| {
                        let late_frame = timeline.last_mut().unwrap();
                        late_frame.auto_add();
                    });
            }
            Tag::FrameLabel(frame_label) => {
                if !vector_animation.animations.is_empty() {
                    // 此时当前动画结束，开始下一个动画，记录总帧数
                    let animation = vector_animation.animation(&animation_name);
                    animation.set_total_frame(current_frame);
                    // 下一个标签定义的动画可能是在当前动画的基础上修改的，所以需要复制当前动画的最后一帧，这种情况出现在动画设计时没有做子动画，所有动画都在根时间轴上
                    let mut new_animation = animation.clone();
                    new_animation
                        .timelines
                        .iter_mut()
                        .for_each(|(_, timeline)| {
                            let mut last_frame = timeline.last().unwrap().clone();
                            timeline.clear();
                            last_frame.duration = 0;
                            last_frame.place_frame = 0;
                            timeline.push(last_frame);
                        });
                    current_frame = 0;
                    animation_name = frame_label.label.to_string_lossy(encoding_for_version);
                    vector_animation.add_animation(&animation_name, new_animation);
                } else {
                    current_frame = 0;
                    animation_name = frame_label.label.to_string_lossy(encoding_for_version);
                    vector_animation.add_animation(&animation_name, Animation::default());
                }
            }
            _ => {
                continue;
            }
        }
    }
    // 最后一个动画结束，记录总帧数
    vector_animation
        .animation(&animation_name)
        .set_total_frame(current_frame);
}

/// 解析DefineSprite标签动画，这个动画被解析为基础动画
fn parse_sprite_animation_as_base_animation(
    vector_animation: &mut VectorAnimation,
    sprite: &swf::Sprite,
) {
    let mut current_frame = 0;
    let mut animation = Animation::default();
    animation.set_total_frame(sprite.num_frames);
    for tag in sprite.tags.iter() {
        match tag {
            Tag::PlaceObject(place_object) => match place_object.action {
                swf::PlaceObjectAction::Place(id) => {
                    add_timeline(&mut animation, place_object, id, current_frame);
                }
                swf::PlaceObjectAction::Modify => {
                    modify_at_depth(&mut animation, place_object, current_frame);
                }
                swf::PlaceObjectAction::Replace(id) => {
                    replace_at_depth(
                        animation.timeline(place_object.depth),
                        place_object,
                        id,
                        current_frame,
                    );
                }
            },
            Tag::RemoveObject(remove_object) => {
                remove_at_depth(animation.timeline(remove_object.depth));
            }
            Tag::ShowFrame => {
                current_frame += 1;
                animation.timelines().iter_mut().for_each(|(_, timeline)| {
                    let late_frame = timeline.last_mut().unwrap();
                    late_frame.auto_add();
                });
            }
            _ => {
                continue;
            }
        }
    }
    vector_animation.register_base_animation(sprite.id, animation);
}

fn add_timeline(
    animation: &mut Animation,
    place_object: &swf::PlaceObject,
    id: CharacterId,
    current_frame: u16,
) {
    let timeline = animation.insert_or_get_timeline(place_object.depth);
    // 补齐空白帧
    let frame_delta = current_frame - timeline.iter().map(|frame| frame.duration).sum::<u16>();
    if frame_delta > 0 {
        timeline.push(Frame::space_frame(current_frame - timeline.len() as u16));
    }
    let mut frame = Frame::new(id, current_frame);
    apply_place_object(&mut frame, place_object);
    timeline.push(frame);
}

fn apply_place_object(frame: &mut Frame, place_object: &swf::PlaceObject) {
    if let Some(matrix) = place_object.matrix {
        frame.transform.matrix = Matrix {
            a: matrix.a.to_f32(),
            b: matrix.b.to_f32(),
            c: matrix.c.to_f32(),
            d: matrix.d.to_f32(),
            tx: matrix.tx.to_pixels() as f32,
            ty: matrix.ty.to_pixels() as f32,
        };
    }

    if let Some(color_transform) = place_object.color_transform {
        frame.transform.color_transform = ColorTransform {
            mult_color: color_transform.mult_rgba_normalized(),
            add_color: color_transform.add_rgba_normalized(),
        };
    }

    if let Some(blend_mode) = place_object.blend_mode {
        frame.blend_mode = BlendMode::from_swf_blend_mode(blend_mode);
    }

    if let Some(filters) = &place_object.filters {
        frame.filters = filters.iter().map(Filter::from).collect();
    }
}

fn replace_at_depth(
    timeline: &mut Vec<Frame>,
    place_object: &swf::PlaceObject,
    id: CharacterId,
    current_frame: u16,
) {
    let mut frame = timeline.last_mut().unwrap().clone();
    frame.id = id;
    frame.place_frame = current_frame;
    frame.duration = 0;
    apply_place_object(&mut frame, place_object);
    timeline.push(frame);
}

fn remove_at_depth(timeline: &mut Vec<Frame>) {
    // 设置为空白帧 （即不显示任何内容）默认Id为0是空白帧
    timeline.push(Frame::new(CharacterId::default(), 0));
}

fn clear_blank_frame(vector_animation: &mut VectorAnimation) {
    let clear = |animation: &mut Animation| {
        animation
            .timelines()
            .iter_mut()
            .for_each(|(_, timeline)| timeline.retain(|frame| frame.id != 0));
    };
    vector_animation.animations.values_mut().for_each(clear);

    vector_animation
        .base_animations
        .values_mut()
        .for_each(clear)
}

fn modify_at_depth(animation: &mut Animation, place_object: &swf::PlaceObject, current_frame: u16) {
    let timeline = animation.insert_or_get_timeline(place_object.depth);
    let mut frame = timeline.last_mut().unwrap().clone();
    frame.place_frame = current_frame;
    frame.duration = 0;
    apply_place_object(&mut frame, place_object);
    timeline.push(frame);
}
