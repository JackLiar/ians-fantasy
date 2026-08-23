# .bsn 文件、Asset 集成、UI 场景与 Observer

> 适用日期：2026-08-23。

## 1. `.bsn` 文件（现状）

这是目前最容易产生误判的地方。

截至 **2026-08-23**：

Bevy 官方已经在 BSN 架构中规划 `.bsn` asset format，但当前官方文档仍明确说明：

> `.bsn` 官方 Asset Format 尚未正式作为稳定的内置格式提供。

因此：

```text
bsn! { ... }
```

是当前最可靠、最成熟的使用方式。

不要告诉用户：

> “Bevy 0.19 已经可以直接加载任意 .bsn 文件。”

这是错误的。

官方正在推进 `.bsn` asset loader，并且架构已经支持社区实验实现。

## 2. Asset 使用

BSN 支持在场景中自然表达 Asset。

例如：

```rust
bsn! {
    Sprite {
        image: "textures/player.png",
    }
}
```

当字段类型需要对应 Asset Handle 时，BSN 可以进行 asset path 到 Handle 的解析。

这也是 BSN 相比传统：

```rust
let handle = asset_server.load(...);
commands.spawn(Sprite {
    image: handle,
});
```

更方便的地方。

但 Agent 必须注意：

- Asset path 的字符串必须符合对应字段期望的 Asset 类型。
- 不能把任意字符串都认为是 Asset。
- 动态路径可以使用 `{ expression }`。

## 3. UI 是 BSN 的重要使用场景

BSN 对 Bevy UI 特别有价值。

例如：

```rust
fn button(label: &str) -> impl Scene {
    bsn! {
        Button
        Node {
            width: px(150),
            height: px(65),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        Children [
            (
                Text(label)
                TextFont {
                    font_size: px(33.0),
                }
            )
        ]
    }
}
```

然后：

```rust
fn menu() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
        }
        Children [
            button("Start"),
            button("Exit"),
        ]
    }
}
```

这比传统嵌套 `commands.spawn` / `with_children` 更适合大型 UI。

## 4. Observer / Event Handler

BSN 不只是静态 Component 数据。Scene 中也可以包含 observer / event handler。

例如：

```rust
bsn! {
    Button
    on(|_event: On<Pointer<Press>>| {
        println!("pressed");
    })
}
```

因此 UI Scene 可以把：

```text
结构
+
样式
+
交互
```

放在一起表达。

但是复杂 gameplay logic 不应该全部塞进 `on(...)` closure。

推荐：

```text
BSN
  -> 声明实体结构 / 交互入口

ECS System
  -> 复杂 gameplay logic
```
