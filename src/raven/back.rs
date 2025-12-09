mod raven;
use crate::raven::prelude::*;

fn main() {
    let story = Rvn! {
        character ling {
            name = "hytomzzz";
            sprite = "Schematics.png";
        }
        background Classroom {
            image = "Schematics.png";
        }
        scene start {
            show background Classroom2
            ling says "start！"
            show background Classroom
            ling says "我准备开始制作godot-swf第三代了"
            ling says "6.45-90°: POWER DOWN - 但城市电网从未真正熄灭。"
            ling says "7.0-89°: POWER UP - 比预定提前0.55°触发。"
            ling says "8.85-90°: 闪烁信号。旧仓库传来的摩尔斯电码…在重复我的学生编号。"
            ling says "10.0°: 水平阻尼已激活。隔离闸门正在自行移动。"
            ling says "测试链路失败。有人绕过了维护开关。"
            ling says "型号 S-40。接线代码 074007。"
            ling says "……究竟谁在定义我们的‘功能’？"
            ling says "开始启动一个引擎测试项目"
            exit game
        }
        scene start2 {
            show background Classroom
            show character ling
            ling says "第二个段落"
            ling says "第二个段落第一句"
            ling says "第二个段落第二句"

        }
    };
    run_raven_game(Some(story));
}