#!/bin/bash

# 验证代码块中 # 注释符号修复

echo "=== 验证代码块中 # 注释符号修复 ==="
echo ""

# 1. 编译项目
echo "1. 编译项目..."
cargo build --release --bin md2docx-cli 2>&1 | tail -3
echo ""

# 2. 转换测试文件
echo "2. 转换测试文件..."
cargo run --release --bin md2docx-cli -- convert --input test_hash_fix.md --output test_hash_fix_verify.docx 2>&1 | tail -3
echo ""

# 3. 验证 # 符号是否存在
echo "3. 验证 # 符号是否存在于生成的 docx 文件中..."
hash_count=$(unzip -p test_hash_fix_verify.docx word/document.xml 2>/dev/null | grep -o "# " | wc -l | tr -d ' ')
echo "   找到 $hash_count 个 # 符号"

if [ "$hash_count" -gt 0 ]; then
    echo "   ✓ # 符号正确显示"
else
    echo "   ✗ # 符号未找到"
    exit 1
fi
echo ""

# 4. 显示部分 # 注释内容
echo "4. 显示部分 # 注释内容："
unzip -p test_hash_fix_verify.docx word/document.xml 2>/dev/null | grep -o "# [^<]*" | head -5
echo ""

# 5. 运行单元测试
echo "5. 运行单元测试..."
cargo test --lib test_parse_inline_formatting --release 2>&1 | grep "test result:"
cargo test --lib test_code_block_with_hash_comments --release 2>&1 | grep "test result:"
echo ""

# 6. 运行所有代码块相关测试
echo "6. 运行所有代码块相关测试..."
cargo test --lib docx::generator::tests --release 2>&1 | grep "test result:"
echo ""

echo "=== 验证完成 ==="
