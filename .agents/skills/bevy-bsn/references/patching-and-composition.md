# Patching 与 Scene Composition

> 适用日期：2026-08-23。

## 1. Patching：BSN 最重要的特性之一

BSN 不是简单的“重新构造整个 Component”。

例如：

```rust
fn enemy() -> impl Scene {
    bsn! {
        Health {
            current: 100,
            max: 100,
        }
    }
}
```

然后：

```rust
fn boss() -> impl Scene {
    bsn! {
        enemy()
        Health {
            max: 1000,
        }
    }
}
```

最终效果是：

```rust
Health {
    current: 100,
    max: 1000,
}
```

而不是：

```rust
Health {
    current: 0,
    max: 1000,
}
```

也就是说：

```text
base scene
    ↓
patch
    ↓
resolved scene
```

这是 BSN 和传统：

```rust
spawn(Health { ... })
```

思维方式非常不同的地方。

## 2. Scene Composition

Scene 可以像普通 Rust 函数一样组合。

例如：

```rust
fn sword() -> impl Scene {
    bsn! {
        Sword
        Damage(20)
    }
}

fn warrior() -> impl Scene {
    bsn! {
        Character
        sword()
    }
}
```

可以进一步 patch：

```rust
fn elite_warrior() -> impl Scene {
    bsn! {
        warrior()
        Damage(50)
    }
}
```

Agent 应优先考虑：

```text
小 Scene
  ↓
组合
  ↓
更大的 Scene
  ↓
patch
  ↓
最终实例
```

而不是为每一种实体复制一份完整 spawn 代码。

## 3. Scene Caching

BSN 支持 Scene caching，但当前能力存在边界。

缓存语法使用：

```rust
:scene
```

例如概念上：

```rust
bsn! {
    :enemy
    Health {
        max: 200,
    }
}
```

表示缓存/复用 `enemy` 的 resolved scene，再进行 patch。

但要注意：

> 当前缓存并不是所有 Scene 形式都支持。

当前文档明确说明：

- Scene asset caching 已支持。
- function scene caching 尚未完整实现。
- SceneComponent caching 尚未完整实现。
- `:` 只能作为合适的 cached scene include 使用。
- 缓存可能改变某些语义，因此是显式 opt-in。

因此 Agent 不要看到 `:` 就认为所有 Scene 都可以缓存。
