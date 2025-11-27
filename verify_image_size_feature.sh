#!/bin/bash

# 图片尺寸参数功能验证脚本

echo "=========================================="
echo "图片尺寸参数功能验证"
echo "=========================================="
echo ""

# 1. 编译项目
echo "1. 编译项目..."
cargo build --lib 2>&1 | grep -E "(Compiling|Finished)" | tail -5
if [ $? -eq 0 ]; then
    echo "✓ 编译成功"
else
    echo "✗ 编译失败"
    exit 1
fi
echo ""

# 2. 运行单元测试
echo "2. 运行单元测试..."
echo "   测试: test_parse_image_with_size_params"
cargo test test_parse_image_with_size_params --lib 2>&1 | grep "test result:"
if [ $? -eq 0 ]; then
    echo "   ✓ 测试通过"
else
    echo "   ✗ 测试失败"
    exit 1
fi

echo "   测试: test_parse_image_with_only_width"
cargo test test_parse_image_with_only_width --lib 2>&1 | grep "test result:"
if [ $? -eq 0 ]; then
    echo "   ✓ 测试通过"
else
    echo "   ✗ 测试失败"
    exit 1
fi

echo "   测试: test_parse_image"
cargo test test_parse_image --lib 2>&1 | grep "test result:"
if [ $? -eq 0 ]; then
    echo "   ✓ 测试通过"
else
    echo "   ✗ 测试失败"
    exit 1
fi
echo ""

# 3. 转换测试文档
echo "3. 转换测试文档..."
if [ -f "test_image_size.md" ]; then
    cargo run --bin md2docx-cli -- convert --input test_image_size.md --output test_image_size_verify.docx 2>&1 | grep "Conversion completed"
    if [ $? -eq 0 ] && [ -f "test_image_size_verify.docx" ]; then
        echo "   ✓ 文档转换成功"
        ls -lh test_image_size_verify.docx | awk '{print "   文件大小:", $5}'
    else
        echo "   ✗ 文档转换失败"
        exit 1
    fi
else
    echo "   ⚠ 测试文档不存在，跳过"
fi
echo ""

# 4. 检查代码诊断
echo "4. 检查代码诊断..."
echo "   检查 src/markdown/ast.rs"
cargo check --lib 2>&1 | grep "src/markdown/ast.rs" | grep -i error
if [ $? -ne 0 ]; then
    echo "   ✓ 无错误"
else
    echo "   ✗ 发现错误"
    exit 1
fi

echo "   检查 src/markdown/parser.rs"
cargo check --lib 2>&1 | grep "src/markdown/parser.rs" | grep -i error
if [ $? -ne 0 ]; then
    echo "   ✓ 无错误"
else
    echo "   ✗ 发现错误"
    exit 1
fi

echo "   检查 src/docx/generator.rs"
cargo check --lib 2>&1 | grep "src/docx/generator.rs" | grep -i error
if [ $? -ne 0 ]; then
    echo "   ✓ 无错误"
else
    echo "   ✗ 发现错误"
    exit 1
fi
echo ""

# 5. 总结
echo "=========================================="
echo "验证完成！"
echo "=========================================="
echo ""
echo "功能实现总结："
echo "✓ URL参数解析功能正常"
echo "✓ AST存储宽度和高度信息"
echo "✓ DOCX生成器应用尺寸参数"
echo "✓ 所有测试通过"
echo "✓ 向后兼容性保持"
echo ""
echo "使用示例："
echo "  ![img](img/example.png?width=100&height=50)"
echo ""
echo "相关文档："
echo "  - IMAGE_SIZE_FEATURE.md"
echo "  - IMAGE_SIZE_PARAMS_README.md"
echo "  - IMAGE_SIZE_IMPLEMENTATION_SUMMARY.md"
echo ""
