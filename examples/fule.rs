use Raven::raven::prelude::*;

fn main() {
    let test = Rvn! {
        character Alice {
            name = "你好";
            sprite = "characters/alice.png";
            
        }
    };
    println!("Example 运行完成！");
    println!("{:?}", test.characters);
}