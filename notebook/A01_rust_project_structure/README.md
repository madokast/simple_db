# Rust 项目模板

学习 Rust 项目的结构，考虑工作空间（workspace）功能。

有一个大项目 basic_operations，根目录创建 `Cargo.toml` 文件，内容如下：

```toml
[workspace]
resolver = "2"
```

## adder 二进制子项目

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

继续创建 `operate` 子项目，执行 `cargo new operate`。

修改这个项目的 `Cargo.toml`，引入 `adder` 子项目。

```toml
[package]
name = "operate"
version = "0.1.0"
edition = "2021"

[dependencies]
adder = { path = "../adder"}
```

在 `src/main.rs` 中，调用 `adder` 子项目的 `add` 方法。

```rust
use adder;

fn main() {
    println!("{}", adder::add(1, 2));
}
```

在 `operate` 子项目中执行 `cargo run`，输出 `3`。可见 `operate` 子项目可以调用 `adder` 子项目的 `add` 方法。

## subtractor 二进制子项目

继续在根目录执行 `cargo new subtractor --bin`，创建了 `subtractor` 子项目。

修改 `Cargo.toml`，引入 `adder` 子项目。

```toml
[package]
name = "subtractor"
version = "0.1.0"
edition = "2021"

[dependencies]
adder = { path = "../adder"}
```

我们这里定义一个 `negate` 方法，便于用 `add` 实现 `subtract`。

`subtractor/src` 下新建 `utils` 文件夹，新增两个文件 `mod.rs` 和 `negator.rs`。 

```rust
// subtractor/src/utils/mod.rs
pub mod negator;

// subtractor/src/utils/negator.rs
pub fn negate(i: i32) -> i32 {
    -i
}
```

在 `subtractor/src/lib.rs` 中，调用 `adder` 子项目的 `add` 方法和 `utils` 模块的 `negator::negate`，实现 `subtract`。

```rust
use adder;
mod utils;
pub fn subtract(left: i32, right: i32) -> i32 {
    adder::add(left, utils::negator::negate(right))
}
```

最后，在 `operate` 子项目中，修改 `Cargo.toml`，引入 `subtractor` 子项目。即新增 `subtractor = { path = "../subtractor"}`。

在 `main` 方法成功调用 `subtract`。

```rust
use adder;
use subtractor;

fn main() {
    println!("{}", adder::add(1, 2));
    println!("{}", subtractor::subtract(1, 2));
}
```

## 测试代码

详见 `adder` 和 `subtractor` 子项目。
