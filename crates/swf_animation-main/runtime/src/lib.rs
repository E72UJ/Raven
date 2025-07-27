use std::collections::HashMap;

use parser::{
    Animations,
    bitmap::CompressedBitmap,
    parse_flash_animation,
    parse_shape::{Graphic, parse_shape_and_bitmap},
};
use swf::CharacterId;

pub mod core;
pub mod parser;

pub fn parse_animation(
    data: Vec<u8>,
) -> (
    Animations,
    HashMap<CharacterId, Graphic>,
    HashMap<u16, CompressedBitmap>,
) {
    let (animations, shapes, bitmaps) =
        parse_flash_animation(data).expect("Failed to parse SWF tag");
    let graphics = parse_shape_and_bitmap(shapes, &bitmaps);
    (animations, graphics, bitmaps)
}

#[cfg(test)]
mod test {
    use std::io::Read;

    use anyhow::Result;

    use crate::{
        core::AnimationPlayer,
        parser::{output_json, parse_flash_animation, parse_shape::parse_shape_and_bitmap},
    };

    #[test]
    fn test() -> Result<()> {
        // 模拟读取测试文件
        let file_path = "D:\\Code\\Rust\\bevy_flash\\assets\\spirit2619src.swf";
        let file = std::fs::File::open(file_path).expect("Failed to open test file");
        let mut reader = std::io::BufReader::new(file);
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .expect("Failed to read test file");
        // 调用解析函数
        let (animations, shapes, bitmaps) =
            parse_flash_animation(data).expect("Failed to parse SWF tag");
        // 写入输出文件
        output_json(&animations, true, "test", "")?;

        // 运行时
        let mut player = AnimationPlayer::new(
            animations.animations,
            animations.children_clip,
            animations.meta.frame_rate,
        );
        player.set_play_animation(
            "default",
            false,
            Some(Box::new(|| {
                println!("播放完成");
            })),
        )?;

        // player.register_frame_event("default", "attack".to_owned(), || {
        //     println!("触发attack事件");
        // })?;
        // player.register_frame_event("default", "idle".to_owned(), || {
        //     println!("触发idle事件");
        // })?;
        // player.register_frame_event("default", "test".to_owned(), || {
        //     println!("触发test事件");
        // })?;
        // player.register_frame_event("default", "miss".to_owned(), || {
        //     println!("触发miss事件");
        // })?;
        // assert!(
        //     player
        //         .register_frame_event("default", "err".to_owned(), || {
        //             println!("触发err事件");
        //         })
        //         .is_err()
        // );

        // player.set_skin("head", "5")?;
        // player.current_skins().iter().for_each(|(k, v)| {
        //     dbg!(k, v);
        // });

        // player.get_skips().iter().for_each(|skips| {
        //     skips.iter().for_each(|(part, skips_name)| {
        //         println!("部位：{}", part);
        //         skips_name.iter().for_each(|skip_name| {
        //             println!("   皮肤：{}", skip_name);
        //         });
        //     });
        // });
        // let mut active_instances = Vec::new();
        // for i in 0..23 {
        //     println!("第{}帧", i + 1);
        //     player.update(&mut active_instances, 1.0 / 30.0)?;
        // }
        Ok(())
    }
}
