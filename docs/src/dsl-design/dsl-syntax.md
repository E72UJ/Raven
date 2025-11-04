# 执行策略
核心分段策略
```rust
  scene 场景1 {
            // 连续执行的指令
            show character Alice
            show background Hello
            play_music "intro.mp3"
            
            // 自动暂停点 - 对话
            narrator "欢迎来到这个神奇的世界！"
            
            // 自动暂停点 - 角色对话  
            dialog Alice "你好，我是Alice！"
            dialog Alice "很高兴见到你！"
            
            // 连续执行的场景切换
            hide background Hello
            show background School
            set Alice emotion happy
            
            // 又一个暂停点
            dialog Alice "我们现在在学校了！"
            
            // 选择菜单 - 暂停点
            choice {
                option "探索学校" -> {
                    dialog Alice "好的，让我们一起探索！"
                    jump scene 探索场景
                }
                option "回家" -> {
                    dialog Alice "那么再见了！"
                    end_scene
                }
            }
        }
    };
```