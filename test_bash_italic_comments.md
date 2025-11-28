# Bash 注释斜体测试

这个文档用于测试 Bash 代码块中注释的斜体渲染功能。

## 基本示例

```bash
#!/bin/bash
# 这是一个注释
echo "Hello World" # 行内注释
ls -la
```

## 复杂示例

```bash
#!/bin/bash

# 脚本功能：备份文件
# 作者：测试
# 日期：2024-01-01

SOURCE_DIR="/home/user/documents"
BACKUP_DIR="/backup" # 备份目录

# 创建备份目录
mkdir -p "$BACKUP_DIR"

# 执行备份
tar -czf "$BACKUP_DIR/backup-$(date +%Y%m%d).tar.gz" "$SOURCE_DIR" # 压缩备份

echo "备份完成" # 输出消息
```

## Shell 脚本示例

```sh
# 这是一个 shell 脚本
for i in {1..5}; do
    echo "Number: $i" # 打印数字
done
```

## Zsh 示例

```zsh
# Zsh 配置
export PATH="$HOME/bin:$PATH" # 添加到 PATH
alias ll='ls -la' # 别名定义
```
