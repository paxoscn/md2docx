# Rust 关键词粗体渲染功能

## 概述

`RustStrategy` 现在支持自动将 Rust 代码中的关键词用粗体渲染。当启用格式化选项时，所有 Rust 关键词会被自动包裹在 `**` 标记中，以便在 Markdown 或 Word 文档中以粗体显示。

## 支持的关键词

该功能支持以下类型的 Rust 关键词：

### 基本关键词
- 控制流: `if`, `else`, `match`, `for`, `while`, `loop`, `break`, `continue`, `return`
- 声明: `fn`, `let`, `const`, `static`, `struct`, `enum`, `trait`, `impl`, `type`, `mod`, `use`
- 访问控制: `pub`, `crate`, `super`, `self`, `Self`
- 其他: `as`, `in`, `mut`, `ref`, `move`, `where`, `extern`, `unsafe`

### 异步关键词
- `async`, `await`

### 保留关键词
- `abstract`, `become`, `box`, `do`, `final`, `macro`, `override`, `priv`, `typeof`, `unsized`, `virtual`, `yield`

### 常用类型
- 整数类型: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- 浮点类型: `f32`, `f64`
- 基本类型: `bool`, `char`, `str`
- 标准库类型: `String`, `Vec`, `Option`, `Result`, `Box`, `Rc`, `Arc`

## 使用方法

### 基本用法

```rust
use md2docx_converter::markdown::code_block::{
    RustStrategy, CodeBlockStrategy, ProcessingConfig,
};

// 创建策略实例
let strategy = RustStrategy::new();

// 配置处理选项（启用格式化）
let config = ProcessingConfig::default()
    .with_syntax_validation(true)
    .with_formatting(true);  // 必须启用格式化才能应用粗体

// 处理代码
let rust_code = r#"
fn main() {
    let x: i32 = 42;
    if x > 0 {
        println!("positive");
    }
}
"#;

let processed = strategy.process(rust_code, &config).unwrap();

// 获取格式化后的代码
if let Some(formatted) = processed.processed_code {
    println!("{}", formatted);
    // 输出:
    // **fn** main() {
    //     **let** x: **i32** = 42;
    //     **if** x > 0 {
    //         println!("positive");
    //     }
    // }
}
```

### 配置选项

关键词粗体功能会在以下条件下自动应用：

1. 必须启用格式化选项: `config.with_formatting(true)`
2. 代码必须通过语法验证（如果启用了语法验证）

```rust
// 仅格式化，不验证语法
let config = ProcessingConfig::default()
    .with_formatting(true);

// 格式化并验证语法
let config = ProcessingConfig::default()
    .with_syntax_validation(true)
    .with_formatting(true);
```

## 示例

### 示例 1: 简单函数

**输入:**
```rust
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

**输出:**
```rust
**fn** add(a: **i32**, b: **i32**) -> **i32** {
    **return** a + b;
}
```

### 示例 2: 结构体和实现

**输入:**
```rust
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
```

**输出:**
```rust
**pub** **struct** Point {
    **pub** x: **f64**,
    **pub** y: **f64**,
}

**impl** Point {
    **pub** **fn** new(x: **f64**, y: **f64**) -> **Self** {
        **Self** { x, y }
    }
}
```

### 示例 3: 控制流

**输入:**
```rust
fn check_value(x: i32) {
    if x > 0 {
        println!("positive");
    } else if x < 0 {
        println!("negative");
    } else {
        println!("zero");
    }
    
    match x {
        0 => println!("zero"),
        _ => println!("non-zero"),
    }
}
```

**输出:**
```rust
**fn** check_value(x: **i32**) {
    **if** x > 0 {
        println!("positive");
    } **else** **if** x < 0 {
        println!("negative");
    } **else** {
        println!("zero");
    }
    
    **match** x {
        0 => println!("zero"),
        _ => println!("non-zero"),
    }
}
```

## 运行示例

项目包含一个完整的示例程序，展示关键词粗体功能：

```bash
cargo run --example rust_keyword_bold_example
```

## 技术实现

关键词粗体功能使用正则表达式实现：

1. 维护一个 Rust 关键词列表
2. 对每个关键词使用词边界匹配 (`\b关键词\b`)
3. 将匹配的关键词替换为 `**关键词**`
4. 保持代码结构和缩进不变

这种方法确保：
- 只匹配完整的关键词（不会匹配关键词的一部分）
- 不影响字符串、注释或标识符中的文本
- 保持原始代码的格式和结构

## 注意事项

1. **格式化必须启用**: 关键词粗体功能只在启用格式化选项时才会应用
2. **支持代码片段**: 即使代码语法无效（如代码片段 `pub mod bajie;`），关键词粗体仍然会应用
3. **性能**: 关键词替换使用正则表达式，对于大型代码块可能会有轻微的性能影响
4. **Markdown 兼容**: 输出使用 Markdown 粗体语法 (`**text**`)，可以直接在 Markdown 文档中使用

## 测试

运行相关测试：

```bash
# 运行所有 Rust 策略测试
cargo test --lib rust_strategy::tests

# 运行特定的关键词粗体测试
cargo test --lib rust_strategy::tests::test_keyword_bold
```

## 未来改进

可能的改进方向：

1. 支持自定义关键词列表
2. 支持不同的格式化样式（如颜色、下划线等）
3. 支持其他编程语言的关键词高亮
4. 优化正则表达式性能
5. 支持更细粒度的格式化控制
