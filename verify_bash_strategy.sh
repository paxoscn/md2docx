#!/bin/bash

echo "=========================================="
echo "Bash 策略验证脚本"
echo "=========================================="
echo

# 1. 运行单元测试
echo "1. 运行 Bash 策略单元测试..."
cargo test --lib bash_strategy --quiet
if [ $? -eq 0 ]; then
    echo "✓ 单元测试通过"
else
    echo "✗ 单元测试失败"
    exit 1
fi
echo

# 2. 运行示例程序
echo "2. 运行示例程序..."
cargo run --release --example bash_italic_comments_example --quiet 2>&1 | grep -q "格式化后的代码"
if [ $? -eq 0 ]; then
    echo "✓ 示例程序运行成功"
else
    echo "✗ 示例程序运行失败"
    exit 1
fi
echo

# 3. 测试文档转换
echo "3. 测试文档转换..."
if [ -f "test_bash_italic_comments.docx" ]; then
    SIZE=$(stat -f%z test_bash_italic_comments.docx 2>/dev/null || stat -c%s test_bash_italic_comments.docx 2>/dev/null)
    if [ "$SIZE" -gt 0 ]; then
        echo "✓ DOCX 文件生成成功 (大小: $SIZE 字节)"
    else
        echo "✗ DOCX 文件为空"
        exit 1
    fi
else
    echo "✗ DOCX 文件不存在"
    exit 1
fi
echo

# 4. 检查文件是否存在
echo "4. 检查实现文件..."
FILES=(
    "src/markdown/code_block/strategies/bash_strategy.rs"
    "examples/bash_italic_comments_example.rs"
    "test_bash_italic_comments.md"
    "BASH_ITALIC_COMMENTS_README.md"
    "BASH_STRATEGY_IMPLEMENTATION_SUMMARY.md"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✓ $file"
    else
        echo "✗ $file 不存在"
        exit 1
    fi
done
echo

# 5. 验证策略注册
echo "5. 验证策略注册..."
grep -q "BashStrategy" src/markdown/code_block/integration.rs
if [ $? -eq 0 ]; then
    echo "✓ BashStrategy 已在 integration.rs 中注册"
else
    echo "✗ BashStrategy 未注册"
    exit 1
fi

grep -q "bash_strategy" src/markdown/code_block/strategies/mod.rs
if [ $? -eq 0 ]; then
    echo "✓ bash_strategy 模块已导出"
else
    echo "✗ bash_strategy 模块未导出"
    exit 1
fi
echo

echo "=========================================="
echo "✓ 所有验证通过！"
echo "=========================================="
echo
echo "Bash 策略已成功实现并集成到系统中。"
echo "支持的语言: bash, sh, shell, zsh"
echo "功能: 注释斜体渲染"
