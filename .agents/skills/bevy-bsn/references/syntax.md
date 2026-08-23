# BSN 基础语法

> 适用日期：2026-08-23。对应官方 `bsn!` / `bsn_list!` 宏（Bevy 0.19+）。

## 1. `Scene` 与 `SceneList`

### 1.1 `Scene`

一个 `Scene` 对应一个 Entity。

```rust
fn sword() -> impl Scene {
    bsn! {
        Sword
        Damage(20)
    }
}
```

可以理解为：

```text
Entity
 ├── Sword
 └── Damage(20)
```

然后：

```rust
commands.spawn_scene(sword());
```

### 1.2 `SceneList`

`SceneList` 对应多个 Entity。

```rust
fn equipment() -> impl SceneList {
    bsn_list![
        Sword,
        Shield,
        Helmet,
    ]
}
```

得到三个 Entity。`Children [...]` 内部使用的也是 SceneList 语义。

## 2. 最基础的 BSN 语法

### 2.1 Unit Component

```rust
bsn! {
    Player
    Health
    Alive
}
```

相当于向同一个 Entity 添加：

```rust
Player::default()
Health::default()
Alive::default()
```

前提是这些类型实现了相应的 Component/Template 支持。

### 2.2 Tuple Component

```rust
bsn! {
    Health(100)
}
```

类似：

```rust
Health(100)
```

### 2.3 Struct Component

```rust
bsn! {
    Health {
        current: 100,
        max: 100,
    }
}
```

注意：

> BSN 中写 Struct Component 时，不应该机械地认为它必须提供所有字段。

BSN 的重要能力之一是 **patching**（详见 `patching-and-composition.md`）。

## 3. 动态 Rust Expression

BSN 并不是完全静态 DSL。需要动态 Rust 表达式时使用 `{ ... }`：

```rust
fn enemy(hp: u32) -> impl Scene {
    bsn! {
        Health {
            current: { hp },
            max: { hp },
        }
    }
}
```

也可以：

```rust
fn sprite(path: String) -> impl Scene {
    bsn! {
        Sprite {
            image: { path },
        }
    }
}
```

复杂表达式应该优先使用 `{}`：

```rust
Health {
    max: { level * 100 + bonus },
}
```

## 4. BSN 中的普通 Rust 值

BSN 的 value 可以使用很多普通 Rust 表达式，例如：

```rust
100
-10
1.5
true
"hello"
SOME_CONSTANT
some_function()
```

复杂 Rust 表达式：

```rust
{ some_rust_expression() }
```

因此不要把 BSN 当成完全独立于 Rust 的脚本语言。

## 5. Scene Function

BSN 很适合把 Scene 写成普通 Rust function：

```rust
fn player(name: &str, hp: u32) -> impl Scene {
    bsn! {
        #Player
        Name(name)
        Health {
            current: hp,
            max: hp,
        }
    }
}
```

调用：

```rust
commands.spawn_scene(player("Jack", 100));
```

这是一种非常推荐的组织方式。建议：

```text
fn player(...)
fn enemy(...)
fn weapon(...)
fn health_bar(...)
fn inventory(...)
```

每个函数返回 `impl Scene`；大型列表返回 `impl SceneList`。

## 6. 最小完整示例

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component, Default, Clone)]
struct Player;

#[derive(Component, Default, Clone)]
struct Body;

#[derive(Component, Default, Clone)]
struct Sword;

fn player() -> impl Scene {
    bsn! {
        @Player
        Children [
            Body,
            Sword,
        ]
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_scene(player());
}
```

如果不使用 SceneComponent，也可以：

```rust
fn player() -> impl Scene {
    bsn! {
        Player
        Children [
            Body,
            Sword,
        ]
    }
}
```
