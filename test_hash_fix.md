# 测试代码块中的 # 符号修复

## Bash 脚本示例

```bash
# 这是注释
echo "Hello"

# 调试构建
cargo build

# 生产构建
cargo build --release

# 多个 # 符号
### 这不是标题，是注释
```

## Python 示例

```python
# 导入模块
import os

def hello():
    # 函数内注释
    print("Hello")
    
# 主程序
if __name__ == "__main__":
    # 调用函数
    hello()
```

## Shell 脚本

```sh
#!/bin/bash
# 脚本开头的 shebang 和注释

# 设置变量
NAME="World"

# 输出
echo "Hello, $NAME"
```

## 混合格式测试

```bash
# 这是注释
echo "Normal text"

# **粗体注释**
echo "Bold comment above"

# *斜体注释*
echo "Italic comment above"
```
