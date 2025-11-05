# DSL 设计

最后更新 2025/11/04 作者 furau

## 分层系统
### 默认层级结构
```rust
show alice happy          # 显示在 master 层
show alice happy at left   # 在指定位置显示
show bg school            # 背景层
show overlay ui_element   # UI层
```
### 多对象

每个层可以有多个对象，用标签区分
```rust
show alice happy as alice_main
show alice sad as alice_memory  # 同一角色的不同实例
```

### 替换机制
```rust
show alice happy     # Alice 显示为开心
show alice sad       # Alice 自动从开心切换到伤心（替换，不是叠加）
hide alice          # Alice 完全消失

show bg room        # 显示房间背景  
show bg school      # 背景自动从房间切换到学校
```

### 对象遮蔽
```rust
character Alice {
    name = "爱丽丝";
    sprite = "characters/protagonist/default.png";
}
character Alice {
    name = "摩西";
    sprite = "characters/protagonist/sad.png";
}



```
