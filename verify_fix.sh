#!/bin/bash

echo "验证代码块空格保留修复"
echo "========================"
echo ""

# 创建测试文件
cat > test_spaces.md << 'EOF'
# 空格保留测试

```python
    def hello():
        print("这行前面有8个空格")
        if True:
            print("这行前面有12个空格")
```
EOF

echo "1. 创建测试文件: test_spaces.md"
echo ""

# 转换文件
echo "2. 转换为 DOCX..."
./target/release/md2docx-cli convert --input test_spaces.md --output test_spaces.docx

echo ""
echo "3. 转换完成！"
echo ""
echo "请打开 test_spaces.docx 文件查看代码块中的空格是否被正确保留。"
echo "每行前面的空格应该完整保留，不会被压缩成单个空格。"
