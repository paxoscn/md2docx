use md2docx_converter::markdown::code_block::{
    strategies::BashStrategy,
    CodeBlockStrategy,
    ProcessingConfig,
};

fn main() {
    // 创建 Bash 策略实例
    let strategy = BashStrategy::new();
    
    // 示例 Bash 代码
    let bash_code = r#"#!/bin/bash
# 这是一个备份脚本
# 作者：测试用户

SOURCE_DIR="/home/user/documents"
BACKUP_DIR="/backup" # 备份目录

# 创建备份目录
mkdir -p "$BACKUP_DIR"

# 执行备份
tar -czf "$BACKUP_DIR/backup-$(date +%Y%m%d).tar.gz" "$SOURCE_DIR" # 压缩备份

echo "备份完成" # 输出消息
"#;

    // 配置处理选项（启用格式化）
    let config = ProcessingConfig::default()
        .with_formatting(true);

    // 处理代码块
    match strategy.process(bash_code, &config) {
        Ok(processed) => {
            println!("=== 原始代码 ===");
            println!("{}", processed.original_code);
            println!();
            
            if let Some(formatted) = &processed.processed_code {
                println!("=== 格式化后的代码（注释为斜体）===");
                println!("{}", formatted);
                println!();
            }
            
            println!("=== 处理信息 ===");
            println!("语言: {:?}", processed.language);
            println!("是否格式化: {}", processed.metadata.is_formatted);
            println!("处理时间: {:?}", processed.metadata.processing_time);
            
            if processed.has_warnings() {
                println!("\n=== 警告 ===");
                for warning in &processed.warnings {
                    println!("- [{}] {}", warning.warning_type, warning.message);
                }
            }
            
            if processed.error_count() > 0 {
                println!("\n=== 错误 ===");
                for error in &processed.errors {
                    println!("- [{}] {}", error.error_type, error.message);
                }
            }
        }
        Err(e) => {
            eprintln!("处理失败: {}", e.message);
        }
    }
}
