// examples/test_raven.rs
use Raven::{raven::*, Rvn};
// 或者
// use your_engine::raven_prelude::*;  // 如果你只想使用 raven 相关功能

fn main() {
    // 调用你的函数
    let test = Rvn! {
        character Alice {
            name = "你好";
            sprite = "characters/alice.png";
            
        }
    };
    println!("Example 运行完成！");
    println!("{:?}", test.characters);
}