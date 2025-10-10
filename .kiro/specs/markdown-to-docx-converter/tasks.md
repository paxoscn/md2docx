# Implementation Plan

- [x] 1. 设置项目结构和核心依赖
  - 创建Rust项目结构，包含lib、bin、web等模块
  - 配置Cargo.toml依赖项（axum、docx-rs、pulldown-cmark、serde等）
  - 设置基础的模块结构和导出
  - _Requirements: 1.1, 3.1, 5.1_

- [x] 2. 实现核心数据模型和配置系统
  - [x] 2.1 创建配置数据结构和序列化
    - 实现ConversionConfig、DocumentConfig、StyleConfig等结构体
    - 添加serde序列化支持和默认值
    - 编写配置验证函数
    - _Requirements: 1.1, 1.2, 1.4_

  - [x] 2.2 实现YAML配置解析器
    - 创建YamlProcessor结构体和解析方法
    - 实现配置文件读取和验证逻辑
    - 编写配置解析的单元测试
    - _Requirements: 1.1, 1.2, 1.4_

  - [x] 2.3 实现Markdown AST数据模型
    - 定义MarkdownElement和InlineElement枚举
    - 创建AST节点的转换和遍历方法
    - 编写AST构建和操作的测试用例
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_

- [x] 3. 实现Markdown解析引擎
  - [x] 3.1 创建Markdown解析器
    - 使用pulldown-cmark实现MarkdownParser结构体
    - 实现从Markdown文本到AST的转换逻辑
    - 处理各种Markdown元素（标题、段落、列表、代码块、表格）
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.6_

  - [x] 3.2 实现图片和链接处理
    - 添加图片URL解析和本地文件处理
    - 实现链接文本和URL的提取逻辑
    - 编写图片和链接处理的测试用例
    - _Requirements: 6.5_

- [x] 4. 实现docx文档生成器
  - [x] 4.1 创建基础docx生成器
    - 使用docx-rs实现DocxGenerator结构体
    - 实现文档基础设置（页面大小、边距、默认字体）
    - 创建样式应用的核心方法
    - _Requirements: 1.1, 1.3, 6.1_

  - [x] 4.2 实现文本格式和样式转换
    - 实现标题样式的应用逻辑（H1-H6到docx标题样式）
    - 添加粗体、斜体、删除线等文本格式转换
    - 实现段落样式和间距设置
    - _Requirements: 6.1, 6.2_

  - [x] 4.3 实现复杂元素转换
    - 添加代码块和行内代码的格式化
    - 实现表格转换逻辑（表头、表格样式、边框）
    - 添加有序和无序列表的转换
    - 实现图片嵌入功能
    - _Requirements: 6.3, 6.4, 6.5, 6.6_

- [x] 5. 实现大模型集成服务
  - [x] 5.1 创建LLM客户端
    - 实现LlmClient结构体和HTTP请求逻辑
    - 添加OpenAI/Claude API的调用接口
    - 实现请求重试和错误处理机制
    - _Requirements: 2.1, 2.4, 2.6_

  - [x] 5.2 实现自然语言配置更新
    - 创建自然语言到配置修改的提示词模板
    - 实现配置更新的解析和应用逻辑
    - 添加配置变更的验证和确认机制
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 6. 实现核心转换服务
  - [x] 6.1 创建转换引擎
    - 实现ConversionEngine结构体
    - 整合Markdown解析器和docx生成器
    - 添加配置应用和转换流程控制
    - _Requirements: 1.1, 1.2, 1.3_

  - [x] 6.2 实现错误处理和日志系统
    - 定义ConversionError和ConfigError错误类型
    - 使用thiserror实现结构化错误处理
    - 集成tracing进行结构化日志记录
    - _Requirements: 7.1, 7.2, 7.4_

- [x] 7. 实现Web API服务器
  - [x] 7.1 创建Axum Web服务器
    - 设置基础的Axum应用和路由结构
    - 实现CORS和日志中间件
    - 添加健康检查端点
    - _Requirements: 3.1, 3.5_

  - [x] 7.2 实现转换API端点
    - 创建POST /api/convert端点处理转换请求
    - 实现请求体解析和响应格式化
    - 添加文件上传和下载功能
    - _Requirements: 3.1, 3.2, 3.4_

  - [x] 7.3 实现配置更新API
    - 创建POST /api/config/update端点
    - 集成自然语言配置更新功能
    - 实现配置预览和确认机制
    - _Requirements: 2.1, 2.2, 2.3, 3.2_

  - [x] 7.4 添加并发处理和资源管理
    - 实现请求限流和超时处理
    - 添加内存使用监控和大文件处理优化
    - 创建异步任务队列处理机制
    - _Requirements: 3.5, 7.3, 7.4_

- [x] 8. 实现命令行工具
  - [x] 8.1 创建CLI参数解析
    - 使用clap实现命令行参数定义
    - 添加输入文件、输出文件、配置文件等参数
    - 实现帮助信息和参数验证
    - _Requirements: 5.1, 5.5_

  - [x] 8.2 实现单文件转换功能
    - 创建单个Markdown文件的转换逻辑
    - 实现配置文件加载和应用
    - 添加转换进度显示和结果输出
    - _Requirements: 5.1, 5.4_

  - [x] 8.3 实现批量转换功能
    - 添加目录遍历和批量文件处理
    - 实现并行转换和进度统计
    - 添加错误处理和日志记录
    - _Requirements: 5.3, 5.6_

  - [x] 8.4 集成自然语言配置修改
    - 实现--config-prompt参数处理
    - 集成LLM服务进行配置更新
    - 添加配置预览和用户确认功能
    - _Requirements: 2.2, 5.2_

- [-] 9. 创建Web前端界面
  - [x] 9.1 设置React项目结构
    - 创建React + TypeScript + Vite项目
    - 配置Tailwind CSS和基础组件
    - 设置路由和状态管理
    - _Requirements: 4.1_

  - [x] 9.2 实现文件上传界面
    - 创建拖拽上传组件
    - 实现文件格式验证和预览
    - 添加上传进度显示
    - _Requirements: 4.1, 4.2_

  - [x] 9.3 实现配置编辑界面
    - 创建YAML配置编辑器
    - 实现自然语言配置修改输入框
    - 添加配置预览和实时更新功能
    - _Requirements: 2.3, 4.1, 4.5_

  - [x] 9.4 实现转换和下载功能
    - 创建转换按钮和进度显示
    - 实现转换结果处理和文件下载
    - 添加错误信息显示和用户反馈
    - _Requirements: 4.3, 4.4_

- [x] 10. 编写综合测试
  - [x] 10.1 创建单元测试套件
    - 为所有核心模块编写单元测试
    - 实现测试数据和模拟对象
    - 添加测试覆盖率报告
    - _Requirements: 7.1, 7.2_

  - [x] 10.2 实现集成测试
    - 创建API端点的集成测试
    - 实现完整转换流程的测试
    - 添加并发请求和性能测试
    - _Requirements: 3.5, 7.3_

  - [x] 10.3 创建端到端测试
    - 实现CLI工具的功能测试
    - 创建Web界面的自动化测试
    - 添加真实文件转换的验证测试
    - _Requirements: 5.1, 5.3, 4.1, 4.3_

- [x] 11. 完善文档和部署配置
  - [x] 11.1 编写用户文档
    - 创建README文件和使用说明
    - 编写API文档和配置示例
    - 添加故障排除和FAQ
    - _Requirements: 5.5_

  - [x] 11.2 创建部署配置
    - 编写Dockerfile和docker-compose配置
    - 创建CI/CD流水线配置
    - 添加生产环境配置和监控设置
    - _Requirements: 7.1, 7.2_

- [x] 12. 实现代码块换行符保留功能
  - [x] 12.1 扩展代码块配置模型
    - 在CodeBlockStyle结构体中添加preserve_line_breaks、line_spacing、paragraph_spacing字段
    - 更新配置序列化和默认值设置
    - 编写配置验证逻辑确保新字段的有效性
    - _Requirements: 6.7_

  - [x] 12.2 重构代码块生成逻辑
    - 修改DocxGenerator的add_code_block方法，按行分割代码内容
    - 为每行代码创建独立的段落和运行对象
    - 实现代码块内部行间距和段落间距的配置应用
    - _Requirements: 6.7_

  - [x] 12.3 处理代码块边缘情况
    - 实现空行保留逻辑，在docx中表示为空段落
    - 添加制表符到空格的转换处理
    - 确保长代码行保持原始格式不自动换行
    - _Requirements: 6.7_

  - [x] 12.4 更新配置文件和示例
    - 在默认配置模板中添加新的代码块配置选项
    - 更新配置示例文件展示换行符保留功能
    - 修改自然语言配置提示词支持新的代码块选项
    - _Requirements: 6.7_

  - [x] 12.5 编写换行符保留测试
    - 创建包含多行代码块的测试用例
    - 验证空行、制表符、长行的正确处理
    - 添加配置开关的功能测试（开启/关闭换行符保留）
    - _Requirements: 6.7_