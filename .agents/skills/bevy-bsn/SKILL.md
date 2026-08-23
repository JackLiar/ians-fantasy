---
name: bevy-bsn
description: Bevy BSN（Bevy Scene Notation）Scene / Prefab / Entity Composition 系统知识，适用于 Bevy 0.19+。当编写或修改 bsn! / bsn_list! 场景代码、spawn_scene、SceneComponent / @Component、Scene patching 与组合、Entity hierarchy，或需要在 BSN 与传统 ECS spawn 之间做选择时使用。
---

# Bevy BSN（Bevy Scene Notation）

> **适用日期：2026-08-23**
> 操作前先检查项目当前 Bevy 版本；BSN 具体 API 有疑问时，优先查当前 Bevy 官方文档（`bevy::scene`）和源码，不要依赖旧版本博客或旧 Scene API。

## 一句话总结

> **把 BSN 理解成 Bevy ECS 上的“可组合、可 patch、可参数化、可形成 hierarchy 的 Scene/Prefab 描述系统”。**

BSN 不是“把 ECS 换掉”，而是：

> **用 Scene 描述一组 ECS Entity/Component 的构造和组合方式。**

## 核心心智模型

```text
BSN
 ├── Scene              -> 描述一个 Entity
 ├── SceneList          -> 描述多个 Entity
 ├── Scene composition  -> Scene 可以组合
 ├── patching           -> 后面的 Scene 可以覆盖前面的组件字段
 ├── hierarchy          -> Children / RelationshipTarget 可以嵌套 Scene
 ├── SceneComponent     -> Component + 与之绑定的 Scene
 ├── Template           -> 支持组件级/字段级 patch
 └── asset integration  -> Scene 中可以使用 asset path
```

即 `Prefab + Component Composition + Hierarchy + Template/Patch`，而不是单纯的“spawn 语法糖”。

```text
                  BSN
                   │
        Entity / hierarchy
        Component composition
        prefab / scene
                   │
                   ▼
                  ECS
                   │
       Components = state/data
       Systems    = behavior
                   │
                   ▼
             Game Logic
```

## 最小速查

```rust
// 单个 Entity
fn sword() -> impl Scene {
    bsn! {
        Sword
        Damage(20)
    }
}

// 多个 Entity
fn equipment() -> impl SceneList {
    bsn_list![
        Sword,
        Shield,
        Helmet,
    ]
}

// spawn
commands.spawn_scene(sword());

// 组合 + patch：boss 复用 enemy，只改 max
fn boss() -> impl Scene {
    bsn! {
        enemy()
        Health {
            max: 1000,   // 只写要改的字段，其余保留 base 的值
        }
    }
}
```

核心符号速查：

| 写法 | 含义 |
| --- | --- |
| `Foo` | 普通 Component（unit） |
| `Foo(1)` / `Foo { a: 1 }` | tuple / struct 组件；struct 只写要 patch 的字段即可 |
| `#Name` | 给 Entity 加 `Name("...")`，用于调试/引用，**不是** Entity ID |
| `@Foo` | 把 `Foo` 当作 SceneComponent 使用（展开其绑定 Scene），**不可**与普通 `Foo` 互换 |
| `@prop: v` | SceneComponent 的 prop，参数化 Scene 生成过程（不是组件字段，不可继续 patch） |
| `Children [...]` | Entity hierarchy；逗号分隔多个 child，无逗号的多行属于同一个 child |
| `{ expr }` | 动态 Rust 表达式 |
| `:name` | 显式 opt-in 的 scene caching include（支持范围有限，见 references） |
| `on(\|e\| { ... })` | 在 Scene 中注册 observer / event handler |

## 决策树

```text
创建游戏对象时：
这个对象只有一个 Entity？
    ├── 是 -> 普通 spawn / bsn! 都可以
    └── 否 -> 有复杂 hierarchy？
                ├── 是 -> Scene（bsn!）
                └── 否 -> 普通 ECS spawn

Component 是否代表一个完整对象（Component + 一整个 hierarchy）？
    ├── 是 -> SceneComponent
    └── 否 -> Component A 需要 Component B/C？
                ├── 是 -> Required Components（spawn 性能极端优先时也选它）
                └── 否 -> 普通 Component
```

## 硬性规则（违反即出错）

1. **先检查项目当前 Bevy 版本**，不要根据旧版 Bevy Scene API（0.14–0.18）猜测 BSN API。
2. **不要告诉用户 `.bsn` 官方 asset format 已经稳定可用**——截至 2026-08-23 尚未正式提供，`bsn! { ... }` 才是当前最可靠的方式。
3. **不要把 BSN 当成替代 ECS**：Scene 描述 `what exists`，System 负责 `what happens`，复杂 gameplay 逻辑放 ECS systems。
4. **`@Foo` 只用于 SceneComponent**，不要随意把 `@Player` 改成 `Player`（语义不同）。
5. **不要为了“统一”把普通数据 Component（Health、Velocity、Faction 等）改成 SceneComponent**。
6. 复用优先 Scene composition + patch，**不要**为每种变体复制完整 spawn 代码。
7. 官方文档与第三方文章冲突时，**以当前 Bevy 官方文档和源码为准**。

## 参考文件（按需加载，不要一次性全部读入）

| 文件 | 内容 | 何时阅读 |
| --- | --- | --- |
| `references/syntax.md` | Scene / SceneList、unit/tuple/struct 组件、动态 `{ expr }`、普通 Rust 值、Scene 函数写法、最小完整示例 | 编写新的 `bsn!` 场景代码时 |
| `references/patching-and-composition.md` | Patching 语义、Scene 组合模式、Scene caching（`:name`）的支持边界 | 复用/修改已有 Scene、写派生变体、使用 caching 时 |
| `references/hierarchy-and-names.md` | `Children [...]` 嵌套规则（逗号/括号）、`#Name`、Relationship 通用语法 | 写 Entity 层级结构、非 Children 的关系时 |
| `references/scene-components.md` | SceneComponent 设计、`@Component`、Props（`@prop`）、vs Required Components 的取舍 | 设计新的 gameplay object 组件、纠结用哪种机制时 |
| `references/assets-ui-and-observers.md` | `.bsn` 文件现状、Asset path 集成、UI 场景写法、Observer / event handler | 加载资产、写 UI、在 Scene 里注册事件时 |
| `references/best-practices.md` | 与 ECS spawn 的关系、代码组织（scene.rs 等）、5 条设计原则、常见错误、Kenshi 类游戏用法、Agent 完整操作规则、官方资料 | 做架构决策、review 代码、大型对象设计时 |
