# 最佳实践、常见错误与 Agent 操作规则

> 适用日期：2026-08-23。

## 1. BSN 与传统 ECS Spawn 的关系

不要认为 BSN 取代 ECS。

传统：

```rust
commands.spawn((
    Player,
    Health::default(),
    Transform::default(),
));
```

依然合理。

BSN 更适合：

```text
复杂 Entity
复杂 hierarchy
可复用 prefab
UI
角色
武器
建筑
敌人
场景组合
```

可以采用混合模式：

```text
简单 Entity
    -> spawn()

复杂对象
    -> bsn! / Scene

运行时状态变化
    -> ECS systems

复杂数据处理
    -> ECS systems
```

## 2. BSN 与 glTF 的关系

BSN 是 Bevy 的 Scene composition system。

glTF 是外部标准 3D asset format。

两者不是竞争关系。

可以理解为：

```text
glTF
  ↓
3D asset / model / scene data

BSN
  ↓
Bevy ECS entity composition
```

未来 Bevy 的方向是让 glTF 等资产更自然地接入新的 Scene 系统。

因此不要说：

> “BSN 是 Bevy 自己的 3D 模型格式。”

这是错误的。

## 3. 推荐的代码组织方式

对于中大型游戏，建议：

```text
src/
├── game/
│   ├── player/
│   │   ├── mod.rs
│   │   ├── component.rs
│   │   ├── systems.rs
│   │   └── scene.rs
│   │
│   ├── enemy/
│   │   ├── mod.rs
│   │   ├── component.rs
│   │   ├── systems.rs
│   │   └── scene.rs
│   │
│   └── weapon/
│       ├── mod.rs
│       ├── component.rs
│       ├── systems.rs
│       └── scene.rs
```

其中：

```text
component.rs
    ECS 数据

systems.rs
    ECS systems

scene.rs
    BSN Scene / SceneComponent
```

例如：

```rust
// scene.rs

pub fn player() -> impl Scene {
    bsn! {
        @Player
        Children [
            body(),
            weapon_mount(),
        ]
    }
}
```

## 4. 最重要的设计原则

### 原则 1：Scene 描述“结构”，System 描述“行为”

不要：

```rust
bsn! {
    // 大量 gameplay logic
}
```

应该：

```text
BSN
    -> Entity / Component / hierarchy

ECS systems
    -> AI / combat / movement / inventory / economy
```

### 原则 2：优先组合小 Scene

不要复制：

```rust
fn player()
fn player_with_sword()
fn player_with_sword_and_shield()
fn elite_player_with_sword_and_shield()
```

应该：

```text
player()
 + sword()
 + shield()
 + elite patch
```

### 原则 3：利用 patch，不要复制默认值

推荐：

```rust
fn enemy() -> impl Scene {
    bsn! {
        Health {
            current: 100,
            max: 100,
        }
    }
}

fn boss() -> impl Scene {
    bsn! {
        enemy()
        Health {
            max: 1000,
        }
    }
}
```

而不是复制整个 `Health`。

### 原则 4：完整 gameplay object 考虑 SceneComponent

如果：

```text
Player Component
```

隐含：

```text
Body
Weapon
Hands
HealthBar
Collider
...
```

考虑：

```rust
#[derive(SceneComponent)]
struct Player;
```

这样可以表达：

```text
Player Component
+
Player Scene
```

之间的强关系。

### 原则 5：不要滥用 SceneComponent

不要：

```rust
#[derive(SceneComponent)]
struct Health;

#[derive(SceneComponent)]
struct Strength;

#[derive(SceneComponent)]
struct Faction;
```

如果这些只是数据 Component，没有对应的实体结构，就没必要。

## 5. 常见错误

### 错误 1：把 `bsn!` 当作普通 struct initializer

错误心智模型：

```text
bsn! = 更短的 spawn
```

正确：

```text
bsn! = Scene / Patch / Composition DSL
```

### 错误 2：认为 Component field 必须完整填写

BSN 支持 patch。

```rust
Health {
    max: 200,
}
```

可能是在已有：

```rust
Health {
    current: 100,
    max: 100,
}
```

上 patch。

### 错误 3：混淆 `Player` 和 `@Player`

```rust
Player
```

和：

```rust
@Player
```

语义不同。后者表示 SceneComponent include。

### 错误 4：认为 `.bsn` 已经是稳定内置资产格式

截至 2026-08-23，不应这么描述。

### 错误 5：把 BSN 当成替代 ECS

BSN 是 ECS 之上的 Scene composition layer。

### 错误 6：把所有逻辑都塞进 Scene

Scene 负责：

```text
what exists
```

System 负责：

```text
what happens
```

## 6. 对 Kenshi 类游戏的推荐用法

如果做类似 Kenshi 的游戏，不建议把所有角色数据都做成 SceneComponent。

推荐：

```text
Character Scene
│
├── Character
├── Transform
├── Visibility
├── Body
│   ├── Head
│   ├── Torso
│   ├── LeftArm
│   ├── RightArm
│   ├── LeftLeg
│   └── RightLeg
│
├── Equipment
│   ├── Weapon
│   └── Backpack
│
└── Presentation
    └── ...
```

而角色的 gameplay 数据：

```text
Health
Strength
Toughness
Dexterity
AttackSkill
DefenseSkill
Athletics
...
```

仍然应该是普通 ECS Components。

例如：

```rust
#[derive(Component)]
struct Strength(pub f32);

#[derive(Component)]
struct Toughness(pub f32);

#[derive(Component)]
struct CombatSkill(pub f32);
```

Scene：

```rust
fn character() -> impl Scene {
    bsn! {
        @Character
        Strength(10.0)
        Toughness(10.0)
        Children [
            body(),
            equipment(),
        ]
    }
}
```

核心思想：

```text
BSN
    -> 角色“由哪些 Entity 组成”

ECS Components
    -> 角色“有哪些状态”

ECS Systems
    -> 角色“如何行动”
```

这三层不要混在一起。

## 7. 给 Coding Agent 的完整操作规则

当 Agent 修改 Bevy 项目时，应遵守：

1. **先检查项目当前 Bevy 版本。**
2. 不要根据旧版 Bevy Scene API 猜测 BSN API。
3. 如果项目使用 Bevy 0.19.x，优先参考当前 `bevy::scene` / `bevy_scene` 文档。
4. 优先使用官方 `bsn!` / `bsn_list!`。
5. 创建复杂对象时优先考虑 `impl Scene` function。
6. 需要复用时优先 Scene composition，而不是复制 spawn 代码。
7. 需要修改已有 Scene 的少量 Component 字段时，优先使用 patch。
8. `@Foo` 只用于 SceneComponent。
9. 普通数据 Component 不要为了“看起来统一”而全部转换成 SceneComponent。
10. 不要假设 `.bsn` 官方 asset loader 已经稳定可用。
11. 不要把 BSN 当成替代 ECS。
12. gameplay 行为优先放 ECS systems。
13. 需要性能优化时，不要默认 BSN 就等于运行时高成本；首先区分 Scene resolution、spawn 和运行时 ECS processing。
14. 需要极端 spawn performance 时，对比 Required Components 与 SceneComponent。
15. 对 BSN 的具体 API 有疑问时，优先查当前 Bevy 官方文档和源码，而不是依赖旧博客/旧版本答案。

## 8. 官方资料

建议 Agent 优先使用这些资料：

- Bevy Scene / BSN 官方 API 文档：`bevy::scene`
- `bsn!` 官方 API 文档
- Bevy 0.19 发布说明
- Bevy 官方 BSN 示例
- Bevy 官方最新源码

如果官方文档和第三方文章冲突：

> **以当前 Bevy 官方文档和当前源码为准。**

尤其要警惕：

```text
Bevy 0.14
Bevy 0.15
Bevy 0.16
Bevy 0.17
Bevy 0.18
```

时期的旧 Scene API 与当前 BSN 架构混用。
