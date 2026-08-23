# SceneComponent 与 @Component

> 适用日期：2026-08-23。

## 1. SceneComponent 是什么

这是 BSN 非常关键的设计。

传统 ECS 中可能存在：

```rust
#[derive(Component)]
struct Player;
```

但是：

```text
Player Component
```

实际上依赖：

```text
Player Entity
 ├── Transform
 ├── Visual
 ├── Weapon
 ├── Inventory
 └── ...
```

传统 ECS 很难保证：

> “只要存在 Player Component，就一定存在整个 Player Scene。”

BSN 使用：

```rust
#[derive(SceneComponent, Default, Clone)]
struct Player {
    score: usize,
}
```

然后：

```rust
impl Player {
    fn scene() -> impl Scene {
        bsn! {
            #Player
            Children [
                LeftHand,
                RightHand,
            ]
        }
    }
}
```

这样：

```rust
commands.spawn_scene(bsn! {
    @Player {
        score: 10,
    }
});
```

会同时：

1. 添加 `Player` Component
2. 生成 Player Scene
3. 生成相关 children

因此：

> `SceneComponent` 可以被理解为“Component + 它对应的实体结构”。

## 2. `@Component`

普通：

```rust
Player
```

表示普通 Component / Scene entry。

而：

```rust
@Player
```

表示：

> 把 `Player` 当作 SceneComponent 使用。

例如：

```rust
bsn! {
    @Player {
        score: 100,
    }
}
```

这和：

```rust
bsn! {
    Player {
        score: 100,
    }
}
```

语义不同。Agent 不应该随意把 `@Player` 改成 `Player`。

## 3. SceneComponent 的设计原则

如果一个 Component 本身代表一个完整的 gameplay object，适合考虑：

```rust
#[derive(SceneComponent, ...)]
struct Player;
```

典型例子：

```text
Player
Enemy
Weapon
Vehicle
Building
NPC
InventoryUI
HealthBar
Character
```

但不要把所有 Component 都做成 SceneComponent。

例如：

```text
Health
Damage
Armor
Velocity
Faction
```

如果它们只是普通数据组件，而不是一个完整的实体结构，就通常应该保持普通 Component。

## 4. SceneComponent Props

SceneComponent 可以有 props。

例如：

```rust
#[derive(SceneComponent, Default, Clone)]
#[scene(PlayerProps)]
struct Player {
    score: usize,
}

#[derive(Default)]
struct PlayerProps {
    scale: f32,
}
```

Scene：

```rust
impl Player {
    fn scene(props: PlayerProps) -> impl Scene {
        bsn! {
            Transform {
                scale: Vec3::splat(props.scale),
            }
        }
    }
}
```

调用：

```rust
bsn! {
    @Player {
        @scale: 2.0,
        score: 100,
    }
}
```

这里：

```rust
@scale
```

是 prop。

而：

```rust
score
```

是 Component 自己的字段。

核心区别：

```text
field
    -> 修改 Component

@prop
    -> 参数化 Scene 生成过程
```

Props 是在 Scene 被 include 时立即求值，因此不能像普通 Component field 那样继续 patch。

## 5. `SceneComponent` vs Required Components

这是 Agent 很容易搞错的地方。

如果只是：

> “Component A 存在时，自动要求 Component B/C 也存在”

优先考虑 Bevy ECS 的 Required Components。

例如概念上：

```text
Player
  requires Transform
  requires Visibility
```

而如果需求是：

> “Player 不仅需要几个 Component，还对应一整个 hierarchy / Scene。”

才更适合：

```rust
SceneComponent
```

简单判断：

```text
只是自动添加 Components
        ↓
Required Components

代表一个完整的 Entity/Hierarchy
        ↓
SceneComponent
```

另外，官方文档明确指出，如果 spawn performance 是极端优先级，Required Components 更适合。
