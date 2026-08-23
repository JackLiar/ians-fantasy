# Entity Hierarchy、#Name 与 Relationship 语法

> 适用日期：2026-08-23。

## 1. Entity Hierarchy

BSN 可以直接表达 Entity hierarchy。

例如：

```rust
bsn! {
    #Player
    Children [
        #Body Body,
        #Weapon Weapon,
        #Shield Shield,
    ]
}
```

大致表示：

```text
Player
├── Body
├── Weapon
└── Shield
```

还可以继续嵌套：

```rust
bsn! {
    #Player
    Children [
        (
            #Body
            Body
            Children [
                Head,
                Torso,
                LeftArm,
                RightArm,
            ]
        ),
        Weapon,
    ]
}
```

### 逗号语义（非常重要）

这里的逗号非常重要：

```rust
Children [
    A B
]
```

表示：

```text
一个 child entity
A + B
```

而：

```rust
Children [
    A,
    B,
]
```

表示：

```text
两个 child entities
A
B
```

推荐 Agent 使用括号提高可读性：

```rust
Children [
    (
        #Body
        Body
    ),
    (
        #Weapon
        Weapon
    ),
]
```

## 2. `#Name`

```rust
#Player
```

会给该 Entity 添加：

```rust
Name("Player")
```

因此：

```rust
bsn! {
    #Player
    Character
}
```

可以理解为：

```text
Entity
 ├── Name("Player")
 └── Character
```

它主要用于：

- 调试
- Entity 引用
- hierarchy 可读性
- Scene 内部命名

不要把 `#Name` 理解成 ECS Entity ID。

## 3. Relationship Syntax

BSN 不仅限于 `Children`。Relationship Target 类型也可以使用类似 SceneList 的写法。

最常见的是：

```rust
Children [
    ChildA,
    ChildB,
]
```

因此 Agent 应理解：

```text
Children [...]
```

是“把多个 Scene 通过 relationship 放进去”的一种典型用法，而不是 BSN 的唯一 hierarchy 机制。
