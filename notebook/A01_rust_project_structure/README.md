# Rust 项目模板

学习 Rust 项目的结构，考虑工作空间（workspace）功能。

有一个大项目 basic_operations，根目录创建 `Cargo.toml` 文件，内容如下：

```toml
[workspace]
resolver = "2"
```

## adder 子项目

随后在根目录执行 `cargo new adder --bin`，创建了 `adder` 子项目，而且 `Cargo.toml` 内容变化如下

```toml
[workspace]

resolver = "2"
members = ["adder"]
```

子项目 src 目录，创建 `lib.rs` 暴露 `add` 方法。

```rust
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}
```

## operate 子项目

