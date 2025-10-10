# Design Document

## Overview

标题自动编号功能将为Markdown到docx转换工具添加对标题编号的支持。该功能通过在YAML配置的标题样式中添加`numbering`字段来启用，系统会自动维护各级标题的编号状态，并在生成docx文档时将编号前缀添加到标题文本中。

编号格式支持占位符模式，如`%1.%2.`表示两级编号（1.1.、1.2.等），`%1.%2.%3`表示三级编号（1.1.1、1.1.2等）。系统会根据标题层级自动递增相应的编号计数器。

## Architecture

### 核心组件

1. **NumberingState** - 编号状态管理器
   - 维护各级标题的当前编号
   - 处理编号的递增和重置逻辑
   - 支持跳级标题的编号处理

2. **NumberingFormatter** - 编号格式化器
   - 解析numbering配置字符串
   - 将编号状态格式化为最终的编号前缀
   - 验证编号格式的有效性

3. **HeadingProcessor** - 标题处理器
   - 在AST处理阶段为标题添加编号
   - 集成到现有的docx生成流程
   - 保持与现有标题样式的兼容性

### 集成点

- **配置模型扩展**: 在`HeadingStyle`结构体中添加`numbering`字段
- **AST处理**: 在markdown解析后、docx生成前的阶段处理编号
- **docx生成**: 修改标题生成逻辑，将编号前缀与标题文本合并

## Components and Interfaces

### 1. 配置模型扩展

```rust
// 在 src/config/models.rs 中扩展 HeadingStyle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingStyle {
    pub font: FontConfig,
    pub spacing_before: f32,
    pub spacing_after: f32,
    pub alignment: Option<String>,
    pub numbering: Option<String>, // 新增字段
}
```

### 2. 编号状态管理

```rust
// 新增 src/numbering/mod.rs
pub struct NumberingState {
    counters: [u32; 6], // 支持H1-H6的编号计数器
}

impl NumberingState {
    pub fn new() -> Self;
    pub fn increment_level(&mut self, level: u8);
    pub fn reset_lower_levels(&mut self, level: u8);
    pub fn get_counter(&self, level: u8) -> u32;
}
```

### 3. 编号格式化器

```rust
// 在 src/numbering/formatter.rs
pub struct NumberingFormatter;

impl NumberingFormatter {
    pub fn parse_format(format: &str) -> Result<Vec<u8>, NumberingError>;
    pub fn format_number(format: &str, state: &NumberingState) -> Result<String, NumberingError>;
    pub fn validate_format(format: &str) -> Result<(), NumberingError>;
}
```

### 4. 标题处理器

```rust
// 在 src/numbering/processor.rs
pub struct HeadingProcessor {
    state: NumberingState,
    config: Arc<ConversionConfig>,
}

impl HeadingProcessor {
    pub fn new(config: Arc<ConversionConfig>) -> Self;
    pub fn process_heading(&mut self, level: u8, text: &str) -> Result<String, NumberingError>;
    pub fn should_number_level(&self, level: u8) -> bool;
}
```

## Data Models

### 编号配置格式

支持的编号格式模式：
- `%1.` - 单级编号：1., 2., 3.
- `%1.%2.` - 两级编号：1.1., 1.2., 2.1.
- `%1.%2.%3` - 三级编号：1.1.1, 1.1.2, 1.2.1
- `%1-%2-%3` - 自定义分隔符：1-1-1, 1-1-2

### 编号状态数据结构

```rust
#[derive(Debug, Clone)]
pub struct NumberingState {
    counters: [u32; 6], // H1到H6的计数器
}

#[derive(Debug, Clone)]
pub struct NumberingFormat {
    levels: Vec<u8>,     // 包含的级别 [1, 2] 表示 %1.%2.
    template: String,    // 格式模板
}
```

### 错误处理

```rust
#[derive(Debug, Error)]
pub enum NumberingError {
    #[error("Invalid numbering format: {0}")]
    InvalidFormat(String),
    #[error("Invalid heading level: {0}")]
    InvalidLevel(u8),
    #[error("Numbering format parsing error: {0}")]
    ParseError(String),
}
```

## Error Handling

### 错误类型和处理策略

1. **配置错误**
   - 无效的numbering格式字符串
   - 策略：记录警告，跳过该级别的编号

2. **级别错误**
   - 超出H1-H6范围的标题级别
   - 策略：使用最接近的有效级别

3. **格式解析错误**
   - 无法解析的占位符模式
   - 策略：降级为无编号模式

4. **状态管理错误**
   - 编号计数器溢出
   - 策略：重置计数器并记录警告

### 错误恢复机制

- **优雅降级**: 编号功能出错时，保持原有的标题处理逻辑
- **部分功能**: 某个级别编号出错时，其他级别继续正常工作
- **详细日志**: 记录所有编号相关的错误和警告信息

## Testing Strategy

### 单元测试

1. **NumberingState测试**
   - 编号递增逻辑
   - 级别重置逻辑
   - 跳级处理逻辑

2. **NumberingFormatter测试**
   - 各种格式模式的解析
   - 格式化输出的正确性
   - 错误格式的处理

3. **HeadingProcessor测试**
   - 标题编号的完整流程
   - 配置变更的响应
   - 错误情况的处理

### 集成测试

1. **配置集成测试**
   - YAML配置的解析和应用
   - 自然语言配置的转换
   - 配置验证逻辑

2. **docx生成测试**
   - 带编号标题的docx输出
   - 编号格式在docx中的正确显示
   - 与其他功能的兼容性

3. **端到端测试**
   - 完整的markdown到docx转换流程
   - Web界面的编号预览功能
   - API接口的编号功能

### 性能测试

1. **大文档测试**
   - 包含大量标题的文档处理
   - 编号状态管理的性能
   - 内存使用情况

2. **复杂编号测试**
   - 多级嵌套标题的处理
   - 频繁级别跳转的性能
   - 编号格式化的效率

### 测试数据

```yaml
# 测试用的配置示例
styles:
  headings:
    1:
      font:
        family: "Times New Roman"
        size: 18
        bold: true
      numbering: "%1."
    2:
      font:
        family: "Times New Roman"
        size: 16
        bold: true
      numbering: "%1.%2."
    3:
      font:
        family: "Times New Roman"
        size: 14
        bold: true
      numbering: "%1.%2.%3"
```

### 边界条件测试

1. **极端情况**
   - 空文档
   - 只有一个级别的标题
   - 深度嵌套的标题结构

2. **错误输入**
   - 无效的编号格式
   - 缺失的配置字段
   - 不支持的标题级别

3. **兼容性**
   - 与现有功能的兼容性
   - 向后兼容性（无numbering配置时的行为）
   - 不同docx查看器的兼容性