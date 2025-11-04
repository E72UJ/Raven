use Raven::raven::prelude::*;
fn main() {
    let story = Rvn! {
        character Alice {
            name = "爱丽丝";
            sprite = "characters/protagonist/default.png";
        }

        character Alice {
            name = "Hytomz";
            sprite = "characters/protagonist/default.png";
        }
        
        background Room {
            image = "background/one.png";
        }

        scene start {
            show background Room
            // show character Alice
            Alice says "欢迎使用Raven引擎"
            Alice says "它的作者是Furau，今天是2025年11/04"
            Alice says "感谢你的使用"
            Alice says "目前开发模式实现了窗口编译"
            Alice says "同时也实现了基础语句跳转"
            exit game
        }
    };
    
    run_raven_game(Some(story));
}