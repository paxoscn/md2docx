# 嵌套加粗标签导致解析卡住的 Bug 修复

## 问题描述

当 Markdown 中包含多个加粗文本且之间没有空格且其中包含特殊字符时，转换会卡住。

### 问题示例

```markdown
**）** a **bbb**
**a****b****c**
**（****）**
```

## 根本原因

pulldown-cmark 解析器在处理连续的加粗标记（如 `**a****b****c**`）时，会生成嵌套的 `Strong` 标签：

```
Start(Strong)
  Text("a")
  Start(Strong)    // 嵌套的 Strong 开始
    Start(Strong)  // 更深层的嵌套
      Text("b")
    End(Strong)
    End(Strong)
  Text("c")
End(Strong)
```

原来的 `collect_text_until_end` 函数没有处理嵌套标签的情况，当遇到第一个 `End(Strong)` 时就会退出，导致索引指针没有正确前进，从而陷入无限循环。

## 解决方案

在 `collect_text_until_end` 函数中添加嵌套层级跟踪：

```rust
fn collect_text_until_end(&self, events: &[Event], index: &mut usize, end_tag_name: &str) -> Result<String, ConversionError> {
    let mut text = String::new();
    let mut nesting_level = 0;  // 新增：跟踪嵌套层级
    
    while *index < events.len() {
        match &events[*index] {
            Event::Start(tag) => {
                // 新增：遇到同类型的开始标签时增加嵌套层级
                if self.tag_matches_name(tag, end_tag_name) {
                    nesting_level += 1;
                }
            },
            Event::End(tag) => {
                if self.tag_matches_name(tag, end_tag_name) {
                    if nesting_level == 0 {
                        // 找到匹配的结束标签
                        *index += 1;
                        break;
                    } else {
                        // 这是嵌套元素的结束标签
                        nesting_level -= 1;
                    }
                }
            },
            Event::Text(t) => text.push_str(t),
            Event::Code(c) => text.push_str(c),
            Event::SoftBreak => text.push(' '),
            Event::HardBreak => text.push('\n'),
            _ => {},
        }
        *index += 1;
    }
    
    Ok(text)
}
```

## 测试用例

添加了以下测试用例来验证修复：

1. `test_parse_bold_with_special_char_and_no_space` - 测试原始问题
2. `test_parse_multiple_bold_no_space_variations` - 测试多种变体
3. `test_nested_emphasis_and_strong` - 测试嵌套的斜体和加粗

所有测试用例都通过，包括：

- `**）** a **bbb**` ✓
- `**）****bbb**` ✓
- `**a****b****c**` ✓
- `**（****）**` ✓
- `**）**a**bbb**` ✓
- `text **）** more **bbb** end` ✓

## 影响范围

此修复影响所有使用 `collect_text_until_end` 函数的地方，包括：

- 加粗文本 (Strong)
- 斜体文本 (Emphasis)
- 删除线文本 (Strikethrough)
- 链接 (Link)
- 图片 (Image)

修复后，所有这些元素都能正确处理嵌套情况，不会再出现卡住的问题。

## 验证

运行以下命令验证修复：

```bash
# 运行所有 parser 测试
cargo test --lib markdown::parser::tests

# 测试实际转换
cargo run --bin md2docx-cli -- convert --input test_nested_bold.md --output test_nested_bold.docx
```

所有测试通过，转换正常完成。
