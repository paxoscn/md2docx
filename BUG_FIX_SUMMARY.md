# Bug 修复总结：嵌套加粗标签导致解析卡住

## 问题
当 Markdown 包含连续的加粗标记（如 `**a****b****c**`）时，解析器会卡住无法完成转换。

## 原因
pulldown-cmark 会为连续的加粗标记生成嵌套的 `Strong` 标签，但原来的 `collect_text_until_end` 函数没有处理嵌套情况，导致在第一个 `End` 标签处就退出，索引指针没有正确前进。

## 修复
在 `src/markdown/parser.rs` 的 `collect_text_until_end` 函数中添加嵌套层级跟踪：

- 遇到同类型的 `Start` 标签时增加嵌套计数
- 遇到 `End` 标签时检查嵌套层级
- 只有在嵌套层级为 0 时才真正退出

## 测试
添加了两个新测试：
- `test_parse_bold_with_special_char_and_no_space` - 测试原始问题
- `test_parse_multiple_bold_no_space_variations` - 测试多种变体
- `test_nested_emphasis_and_strong` - 测试嵌套格式

所有测试通过 ✓

## 验证
```bash
# 原始问题
echo "**）** a **bbb**" > test.md
cargo run --bin md2docx-cli -- convert --input test.md --output test.docx
# ✓ 转换成功
```
