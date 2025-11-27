# 图片尺寸参数功能实现

## 功能概述

实现了对Markdown图片URL中查询参数的支持，允许通过URL参数控制图片的宽度和高度。

## 语法

```markdown
![alt_text](image_url?width=100&height=50)
```

## 支持的参数

- `width`: 图片宽度（像素）
- `height`: 图片高度（像素）

## 使用示例

### 1. 同时指定宽度和高度

```markdown
![img](img/llm-no-agent.png?width=100&height=50)
```

这将把图片渲染为宽100像素、高50像素。

### 2. 只指定宽度

```markdown
![img](img/test.png?width=200)
```

这将把图片宽度设为200像素，高度使用配置文件中的默认值。

### 3. 只指定高度

```markdown
![img](img/test.png?height=150)
```

这将把图片高度设为150像素，宽度使用配置文件中的默认值。

### 4. 不指定参数

```markdown
![img](img/normal.png)
```

这将使用配置文件中的默认宽度和高度。

## 实现细节

### 1. AST修改

在 `src/markdown/ast.rs` 中，为 `MarkdownElement::Image` 添加了 `width` 和 `height` 字段：

```rust
Image {
    alt_text: String,
    url: String,
    title: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}
```

### 2. Parser修改

在 `src/markdown/parser.rs` 中：

- 添加了 `parse_image_url_params` 方法来解析URL查询参数
- 修改了图片解析逻辑，在创建 `Image` 元素时提取并保存宽度和高度参数
- URL会被清理，移除查询参数部分

```rust
fn parse_image_url_params(url: &str) -> (String, Option<u32>, Option<u32>) {
    // 解析 ?width=100&height=50 格式的参数
    // 返回 (clean_url, width, height)
}
```

### 3. Generator修改

在 `src/docx/generator.rs` 中：

- 修改了 `add_image` 方法签名，接受 `width` 和 `height` 参数
- 当提供了自定义尺寸时，使用 `embed_local_image_sized` 方法
- 如果没有提供尺寸，则使用配置文件中的默认值

## 测试

添加了以下测试用例：

1. `test_parse_image_with_size_params`: 测试同时包含宽度和高度参数的图片
2. `test_parse_image_with_only_width`: 测试只包含宽度参数的图片

所有现有测试都已更新以适配新的数据结构。

## 向后兼容性

此功能完全向后兼容：

- 不带参数的图片URL继续正常工作
- 使用配置文件中的默认尺寸
- 不影响现有的Markdown文档

## 注意事项

1. 参数只对本地图片有效（远程URL图片目前显示为占位符）
2. 如果只指定一个维度，另一个维度将使用配置文件中的默认值
3. 参数值必须是有效的正整数
4. 无效的参数会被忽略，使用默认值

## 示例输出

运行以下命令测试功能：

```bash
cargo run --bin md2docx-cli -- convert --input test_image_size.md --output test_image_size.docx
```

生成的DOCX文档将包含按指定尺寸渲染的图片。
