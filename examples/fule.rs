use Raven::raven::prelude::*;
fn main() {
    let character_name = format!("诡异{}", output_hello());
    let test = Rvn! {
        character  Alice {
            name = character_name;
            sprite = "characters/alice.png";   
        }
        background  School {
            image = "backgrounds/forest.png";
        }
        background Hello{
            image = "backgrounds/hello.png";
        }

        scene 场景1 {
            show character Alice
            show background Hello
        }
        scene 场景2 {
            show character Alice
            show background School
        }
    };

    println!("{:?}", test.characters.get("Alice").unwrap().name);
}


fn output_hello() -> String{
    "hello".to_string()
}