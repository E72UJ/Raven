# To Make Better Use Of SWF Resources

## 目标

  该库将实现重构flash纯动画运行时

## swf_to_json

一个将`Adobe Animate` / `Flash`动画转成JSON数据的工具。

1. 用法:

- 自己编译：

    ```bash
          cargo run --release --package=swf_to_json  D:\Code\Rust\swf_animation\core\tests\swfs\spirit2159src.swf
          // 或者指定放大倍数
          cargo run --release --package=swf_to_json  D:\Code\Rust\swf_animation\core\tests\swfs\spirit2159src.swf --scale 2.0
    ```

- 编译完成
  
    ```bash
        swf_to_json "D:\Code\Rust\swf_animation\core\tests\swfs\spirit2159src.swf"
        // 或者指定放大倍数
        swf_to_json "D:\Code\Rust\swf_animation\core\tests\swfs\spirit2159src.swf" --scale 2.0
    ```

> 可用第三方工具，如`Texture Packer`等工具打包成`atlas`，然后使用`SpriteSheet`加载。

1. 动画文件 animation.json 的格式**暂定**如下:

    ```json
    {
      // 动画名称
      "name": "head",
      // 动画帧率
      "frame_rate": 30,
      // 子动画，Key为子动画ID(对一个swf文件，子动画ID是唯一的),Value为子动画数据
      "base_animations": {
        "2": {
          // 表示一个时间轴，Key为时间轴深度，深度按照从小到大渲染，即深度值越小的在底层，深度值越大的在顶层，深度值大的重叠时会遮住小的。
          "time_lines": {
            // key为深度值，frames为帧数据
            "2": [
              {
                // 资源ID，子动画或者图形资源中的一个
                "id": 1,
                // 第几帧才开始渲染
                "place_frame": 0,
                // 帧持续时间，这里持续一帧
                "duration": 1,
                // 矩阵变换，用于缩放、旋转、平移等操作。
                "matrix": {
                  "a": 5.4772186, // 缩放X
                  "b": 0.0, // 旋转倾斜0（RotateSkew0）
                  "c": 0.0, // 旋转倾斜1（RotateSkew1）
                  "d": 5.4772186, // 缩放Y
                  "tx": -203.5, // 平移X
                  "ty": -223.0 // 平移Y
                },
                // 颜色变换，用于改变颜色。
                "color_transform": {
                  "mult_color": [0.0, 0.0, 0.0, 0.0],
                  "add_color": [0.0, 0.0, 0.0, 0.0]
                },
                // 混合模式，用于控制两个图像之间的混合方式。默认为Normal。TODO: 混合模式 改为数值判断？还是枚举？
                "blend_mode": "Normal",
                // 滤镜
                "filters": []
              }
            ]
          },
          // 动画总帧数
          "total_frames": 1
        }
      },
      // 目标动画，key为动画名称，value为动画数据，这里的名称默认是`default`，因为没有在adobe animation中设置动画标签，所以默认为`default`。当设置了动画标签后，这里会根据标签名称进行分类，一个标签代表一个动画。
      "animations": {
        "default": {
          "time_lines": {
            "1": {
              "frames": [
                {
                  "id": 2,
                  "place_frame": 0,
                  "duration": 1,
                  "matrix": {
                    "a": 1.0,
                    "b": 0.0,
                    "c": 0.0,
                    "d": 1.0,
                    "tx": 608.2,
                    "ty": 534.15
                  },
                  "color_transform": {
                    "mult_color": [0.0, 0.0, 0.0, 0.0],
                    "add_color": [0.0, 0.0, 0.0, 0.0]
                  },
                  "blend_mode": "Normal"
                }
              ]
            }
          }
        },
        // 动画总帧数
        "total_frames": 1
      }
    }
    ```

2. 注意事项（规则）
    - 只有根影片剪辑的动画才会解析为`animations`项下的动画，其余的影片都作为子动画。若根影片无动画则`animations`下不会有可播放动画。
    - 使用多动画时要保证所有需要的动画都在根影片时间轴，使用影片中的`Label`分割，并且`Label`将作为动画的名称，若根影片无任何标签将会生成默认名（名称为`default`）。
    - 如果发现导入godot中素材堆叠在一起，可能是需要启用 `useRootTransform`。

3. 由于本人不熟悉`Godot`引擎，所有诚邀伙伴帮助完善`Godot`插件。
