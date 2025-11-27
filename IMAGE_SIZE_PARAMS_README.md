# 图片尺寸参数功能

## 快速开始

在Markdown图片语法中，可以通过URL查询参数来控制图片的宽度和高度：

```markdown
![图片描述](图片路径?width=宽度&height=高度)
```

## 使用示例

### 示例 1：设置宽度和高度

```markdown
![示例](img/example.png?width=100&height=50)
```

这将把图片渲染为宽100像素、高50像素。

### 示例 2：只设置宽度

```markdown
![示例](img/example.png?width=200)
```

这将把图片宽度设为200像素，高度使用默认值。

### 示例 3：只设置高度

```markdown
![示例](img/example.png?height=150)
```

这将把图片高度设为150像素，宽度使用默认值。

## 参数说明

- `width`: 图片宽度（像素），必须是正整数
- `height`: 图片高度（像素），必须是正整数

## 测试

运行以下命令测试功能：

```bash
# 编译项目
cargo build

# 运行测试
cargo test test_parse_image_with_size_params --lib
cargo test test_parse_image_with_only_width --lib

# 转换示例文档
cargo run --bin md2docx-cli -- convert --input test_image_size.md --output test_image_size.docx
```

## 实现细节

1. **Parser层**：解析URL中的查询参数，提取width和height值
2. **AST层**：在Image节点中存储width和height信息
3. **Generator层**：根据参数值调整生成的DOCX文档中的图片尺寸

## 向后兼容性

- 不带参数的图片URL继续正常工作
- 使用配置文件中的默认尺寸
- 不影响现有的Markdown文档

## 相关文件

- `src/markdown/ast.rs`: Image节点定义
- `src/markdown/parser.rs`: URL参数解析逻辑
- `src/docx/generator.rs`: 图片尺寸应用逻辑
- `test_image_size.md`: 测试示例文档
- `IMAGE_SIZE_FEATURE.md`: 详细功能说明
