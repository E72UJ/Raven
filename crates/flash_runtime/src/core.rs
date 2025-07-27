use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};

use anyhow::Result;
use error::RuntimeError;
use glam::Mat4;
use swf::{CharacterId, Twips};

use crate::parser::{
    Animation, DepthTimeline, KeyFrame, MovieClip, Transform, parse_shape::matrix::Matrix,
    types::BlendMode,
};

use filter::Filter as RenderFilter;

mod error;
pub mod filter;
mod state_machine;

type CompletionCallback = Box<dyn FnOnce(&mut AnimationPlayer) + Send + Sync + 'static>;
type FrameEventCallback = Box<dyn Fn() + Send + Sync + 'static>;

#[derive(Default)]
pub struct AnimationPlayer {
    // ---------资源-----------
    /// 原flash动画帧率
    frame_rate: f32,
    /// 动画数据
    animations: HashMap<String, Animation>,
    /// 动画子影片资源
    children_clip: HashMap<CharacterId, MovieClip>,
    /// 活动的子影片存在同一个被多次使用的情况
    active_clip: HashMap<String, MovieClip>,
    /// 运行时实例，扁平化结构
    active_instances: Vec<RuntimeInstance>,

    // ----------控制-----------
    /// 播放速度
    speed: f32,
    /// 是否循环
    looping: bool,
    /// 当前时间
    current_time: f32,
    /// 是否播放
    playing: bool,
    /// 当前动画名
    current_animation_name: Option<String>,
    /// 皮肤
    current_skins: HashMap<String, String>,
    /// 播放完成时回调
    on_completion: Option<CompletionCallback>,
    /// 用于帧事件
    frame_event_listeners: HashMap<String, Vec<FrameEventCallback>>,
}

impl AnimationPlayer {
    /// 默认播放速度 x 1.0
    pub fn new(
        animations: HashMap<String, Animation>,
        children_clip: HashMap<CharacterId, MovieClip>,
        frame_rate: f32,
    ) -> Self {
        Self {
            animations,
            children_clip,
            speed: 1.0,
            frame_rate,
            playing: true,
            ..Default::default()
        }
    }

    pub fn update(&mut self, active_instances: &mut Vec<RuntimeInstance>, delta_time: f32) {
        if !self.playing || self.current_animation_name.is_none() {
            return;
        }

        let animation_name = self.current_animation_name.as_ref().unwrap().clone();

        // 1. Time Advancement & Looping
        let previous_time = self.current_time;
        let elapsed_time = delta_time * self.speed;
        self.current_time += elapsed_time;

        let animation = self.animations.get(&animation_name).unwrap();
        let duration = animation.duration;
        let mut on_completion = None;

        if self.current_time >= duration {
            if self.looping {
                self.current_time %= duration;
                // 子动画也需要重置
                self.active_clip.iter_mut().for_each(|(_, v)| {
                    v.current_time = self.current_time;
                });
            } else {
                self.current_time = duration;
                self.playing = false;

                // TODO: 触发动画完成事件
                on_completion = self.on_completion.take();
            }
        }

        let children_clip = &self.children_clip;
        let active_clip = &mut self.active_clip;
        let current_skins = &mut self.current_skins;

        // 2.Instance Lifecycle & Property Updates (Iterate through Depths)
        active_instances.clear();
        let base_transform = Matrix::IDENTITY;
        let base_color_transform = swf::ColorTransform::IDENTITY;
        // 实例标识，用于防止重复生成
        let id = "root";
        collect_current_time_active_shape(
            id,
            &animation.timeline,
            previous_time,
            elapsed_time,
            active_clip,
            children_clip,
            current_skins,
            self.frame_rate,
            active_instances,
            base_transform,
            base_color_transform,
            BlendMode::Normal,
            Vec::new(),
        )
        .unwrap();

        // 3.Frame Event Handle
        // 处理时间值精度问题
        let cmp_current_time = (self.current_time * 1.0e6).trunc();
        let cmp_previous_time = (previous_time * 1.0e6).trunc();

        for event_keyframe in animation.events.iter() {
            let time = (event_keyframe.time * 1.0e6).trunc();
            if (time >= cmp_previous_time) && time < cmp_current_time {
                if let Some(frame_events) = self.frame_event_listeners.get(&event_keyframe.name) {
                    frame_events.iter().for_each(|event| event());
                }
            }
        }

        // 触发完成事件
        if let Some(on_completion) = on_completion {
            on_completion(self);
        }
    }

    pub fn active_instances(&self) -> &Vec<RuntimeInstance> {
        &self.active_instances
    }

    pub fn animation_names(&self) -> Vec<&String> {
        self.animations.keys().collect::<Vec<_>>()
    }

    pub fn set_on_completion(&mut self, on_completion: CompletionCallback) {
        self.on_completion = Some(on_completion);
    }

    /// 设置播放动画
    /// - name 动画名
    /// - looping 是否循环播放
    /// - on_completion 回调事件
    pub fn set_play_animation(&mut self, name: &str, looping: bool) -> Result<()> {
        if !self.animations.contains_key(name) {
            return Err(RuntimeError::AnimationNotFound(name.to_owned()).into());
        }

        // 重置时间
        self.current_time = 0.0;
        self.set_playing(true);
        // 清除活动实例
        self.active_instances.clear();

        self.current_animation_name = Some(name.to_owned());
        self.looping = looping;
        Ok(())
    }

    /// 设置皮肤
    /// - part_name 部位名
    /// - skin_name 皮肤名
    pub fn set_skin(&mut self, part_name: &str, skin_name: &str) -> Result<()> {
        let skin_name = skin_name.to_owned();
        if let Some(target_skip) = self
            .get_skips()
            .iter()
            .find(|part_skips| part_skips.contains_key(part_name))
        {
            if target_skip.get(part_name).unwrap().contains(&&skin_name) {
                self.current_skins.insert(part_name.to_owned(), skin_name);
            } else {
                return Err(RuntimeError::SkinNotFound(skin_name).into());
            }
        } else {
            return Err(RuntimeError::SkinPartNotFound(part_name.to_owned()).into());
        }
        Ok(())
    }

    pub fn get_skips(&self) -> Vec<HashMap<&str, Vec<&String>>> {
        self.children_clip
            .values()
            .filter(|clip| clip.is_skin_frame())
            .map(|clip| {
                let mut skip = HashMap::new();
                skip.insert(
                    clip.name().expect("没有就是有Bug"),
                    clip.skin_frames().keys().collect::<Vec<_>>(),
                );
                skip
            })
            .collect::<Vec<_>>()
    }

    pub fn current_skins(&self) -> &HashMap<String, String> {
        &self.current_skins
    }

    /// 注册一个监听特定名称帧事件的回调函数。
    ///
    /// # Arguments
    /// * `animation_name` - 要监听的动画名。
    /// * `event_name` - 要监听的事件名称 (例如 "footstep", "hit_impact")。
    /// * `callback` - 当事件触发时要调用的函数。
    pub fn register_frame_event<T>(
        &mut self,
        animation_name: &str,
        event_name: String,
        callback: T,
    ) -> Result<()>
    where
        T: Fn() + Send + Sync + 'static,
    {
        // 判断监听的事件是否存在
        if let Some(animation) = self.animations.get(animation_name) {
            if animation
                .events
                .iter()
                .any(|event| event.name == event_name)
            {
                self.frame_event_listeners
                    .entry(event_name)
                    .or_default()
                    .push(Box::new(callback));
            } else {
                return Err(RuntimeError::AnimationEventNotFound(event_name).into());
            }
        } else {
            return Err(RuntimeError::AnimationNotFound(animation_name.to_owned()).into());
        }

        Ok(())
    }

    /// 移除指定事件名称的所有监听器。
    pub fn clear_frame_event_listeners(&mut self, event_name: &str) {
        self.frame_event_listeners.remove(event_name);
    }
    /// 移除所有帧事件监听器。
    pub fn clear_all_frame_event_listeners(&mut self) {
        self.frame_event_listeners.clear();
    }

    pub fn current_animation_name(&self) -> Option<&str> {
        self.current_animation_name.as_deref()
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = if speed <= 0.0 { 0.0 } else { speed };
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    pub fn looping(&self) -> bool {
        self.looping
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }
}

fn collect_current_time_active_shape(
    instance_id: &str,
    timeline: &BTreeMap<u16, DepthTimeline>,
    current_time: f32,
    elapsed_time: f32,
    active_clip: &mut HashMap<String, MovieClip>,
    children_clip: &HashMap<CharacterId, MovieClip>,
    current_skins: &mut HashMap<String, String>,
    frame_rate: f32,
    active_instances: &mut Vec<RuntimeInstance>,
    base_transform: Matrix,
    base_color_transform: swf::ColorTransform,
    base_blend_mode: BlendMode,
    base_filters: Vec<RenderFilter>,
) -> Result<()> {
    for (depth, depth_timeline) in timeline {
        let placements = &depth_timeline.placement;
        let (Some(start_placement), _end_placement) = find_key_frame(current_time, placements)
        else {
            continue;
        };

        let start_keyframe = placements.get(start_placement).unwrap();
        if let Some(id) = start_keyframe.resource_id() {
            // 唯一标识
            let instance_id = format!("{}_{}", instance_id, depth);

            let transforms = &depth_timeline.transforms;
            // 既然start存在那么transform一定存在
            let (start, _end) = find_key_frame(current_time, transforms);
            // TODO: 补帧出现变换异常
            // let transform = match end_placement {
            //     // 如果后一帧是空针，不需要插值计算，所在帧切换了资源图也不插帧
            //     Some(index)
            //         if placements.get(index).unwrap().resource_id().is_none()
            //             || placements.get(index).unwrap().resource_id().unwrap() != id =>
            //     {
            //         transforms.get(start.unwrap()).unwrap().matrix
            //     }
            //     _ => lerp_transform(
            //         start.expect("transform 必须存在、否则这会是一个Bug"),
            //         end,
            //         transforms,
            //         current_time,
            //     ),
            // };
            let transform: Matrix = transforms.get(start.unwrap()).unwrap().matrix;
            let current_transform = base_transform * transform;

            // 颜色变换
            let color_transform = start_keyframe.color_transform().color_transform;
            let current_color_transform = base_color_transform * color_transform;

            if let Some(child_clip) = children_clip.get(&id) {
                // 混合模式
                let blend_mode = start_keyframe.blend_mode();

                // 滤镜
                let mut filters: Vec<RenderFilter> = start_keyframe
                    .filters()
                    .iter()
                    .map(RenderFilter::from)
                    .collect();

                filters.append(&mut base_filters.clone());

                let clip_instance_id = format!("{}_{}", instance_id, id);
                let mut child_clip = if let Some(child_clip) = active_clip.remove(&clip_instance_id)
                {
                    child_clip
                } else {
                    child_clip.clone()
                };

                // 判断是否是皮肤clip
                let child_current_time = if child_clip.is_skin_frame() {
                    // 是皮肤clip
                    let name = child_clip.name().expect("替换皮肤的影片剪辑必须命名！");
                    // 是否设置了皮肤
                    let skip_frame = if let Some(skip_name) = current_skins.get(name) {
                        if let Some(frame) = child_clip.skin_frame(skip_name) {
                            *frame
                        } else {
                            return Err(RuntimeError::SkinNotFound(skip_name.to_owned()).into());
                        }
                    } else {
                        // 没有设置皮肤，使用默认皮肤
                        child_clip.default_skin_frame()
                    };
                    // 计算对应帧对应的事件
                    skip_frame as f32 / frame_rate
                } else {
                    child_clip.current_time
                };

                collect_current_time_active_shape(
                    &instance_id,
                    child_clip.timeline(),
                    child_current_time,
                    elapsed_time,
                    active_clip,
                    children_clip,
                    current_skins,
                    frame_rate,
                    active_instances,
                    current_transform,
                    current_color_transform,
                    blend_mode,
                    filters,
                )?;
                child_clip.current_time += elapsed_time;
                if child_clip.current_time >= child_clip.duration() {
                    child_clip.current_time %= child_clip.duration();
                }
                active_clip.insert(clip_instance_id, child_clip);
            } else {
                // 记录这个child_movie找到的shape为当前活动实例，将每一帧的实例Shape扁平化输出，游戏引擎中迭代实在不方便
                active_instances.push(RuntimeInstance::new(
                    id,
                    current_transform,
                    current_color_transform,
                    base_blend_mode,
                    base_filters.clone(),
                ));
            }
        }
    }
    Ok(())
}

fn find_key_frame<T: KeyFrame>(time: f32, key_frames: &[T]) -> (Option<usize>, Option<usize>) {
    match key_frames.binary_search_by(|k| k.time().partial_cmp(&time).unwrap_or(Ordering::Less)) {
        // 刚好相等
        Ok(index) => (Some(index), None),
        Err(index) => {
            if index == 0 {
                // 时间在第一个关键帧之前，当前深度的keyframe不显示
                (None, None)
            } else if index >= key_frames.len() {
                // 时间在最后一个关键帧之后，使用最后一帧值
                (Some(key_frames.len() - 1), None)
            } else {
                // 时间在 index-1 到 index 之间
                (Some(index - 1), Some(index))
            }
        }
    }
}

/// 实例只需要存储用于引擎渲染的Shape就行吗？
/// 在多个Shape合成的MovieClip上应用滤镜，需要一起渲染，
#[derive(Debug, Default)]
pub struct RuntimeInstance {
    id: CharacterId,
    transform: Matrix,
    color_transform: swf::ColorTransform,
    blend: BlendMode,
    filters: Vec<RenderFilter>,
}

impl RuntimeInstance {
    fn new(
        id: CharacterId,
        transform: Matrix,
        color_transform: swf::ColorTransform,
        blend: BlendMode,
        filters: Vec<RenderFilter>,
    ) -> Self {
        Self {
            id,
            transform,
            color_transform,
            blend,
            filters,
        }
    }

    pub fn id(&self) -> CharacterId {
        self.id
    }

    pub fn blend(&self) -> BlendMode {
        self.blend
    }

    pub fn transform(&self) -> Mat4 {
        self.transform.into()
    }

    pub fn transform_matrix(&self) -> Matrix {
        self.transform
    }

    pub fn color_transform(&self) -> swf::ColorTransform {
        self.color_transform
    }

    pub fn filters_mut(&mut self) -> &mut Vec<RenderFilter> {
        &mut self.filters
    }
}

#[allow(dead_code)]
fn lerp_transform(
    start_index: usize,
    end_index: Option<usize>,
    transform: &[Transform],
    current_time: f32,
) -> Matrix {
    let start = transform.get(start_index).unwrap();

    let matrix = if let Some(end_index) = end_index {
        let end = transform.get(end_index).unwrap();
        lerp_matrix(
            &start.matrix,
            &end.matrix,
            calc_lerp_factor(start.time, end.time, current_time),
        )
    } else {
        start.matrix
    };
    matrix
}

/// 对两个 Matrix 进行线性插值
///
/// - `start`: 起始矩阵
/// - `end`: 结束矩阵
/// - `t`: 插值因子 (0.0 to 1.0)
///
/// 返回插值后的新矩阵
fn lerp_matrix(start: &Matrix, end: &Matrix, t: f32) -> Matrix {
    // 确保 t 在 [0, 1] 范围内
    let t = t.clamp(0.0, 1.0);

    Matrix {
        a: start.a + (end.a - start.a) * t,
        b: start.b + (end.b - start.b) * t,
        c: start.c + (end.c - start.c) * t,
        d: start.d + (end.d - start.d) * t,
        tx: Twips::from_pixels(
            start.tx.to_pixels() + (end.tx.to_pixels() - start.tx.to_pixels()) * t as f64,
        ),
        ty: Twips::from_pixels(
            start.ty.to_pixels() + (end.ty.to_pixels() - start.ty.to_pixels()) * t as f64,
        ),
    }
}

fn calc_lerp_factor(start_time: f32, end_time: f32, current_time: f32) -> f32 {
    let segment_duration = end_time - start_time;
    let raw_t = if segment_duration <= 0.0 {
        0.0
    } else {
        (current_time - start_time) / segment_duration
    };

    raw_t.clamp(0.0, 1.0)
}
