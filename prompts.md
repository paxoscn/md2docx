有个bug, 当 Markdown 中包含多个加粗文本且之间没有空格且其中包含特殊字符"）"时转换会卡住,例如:
**）** a **bbb**

以下代码似乎没能加粗:
```rust
pub mod bajie;
let y = 5;
```

---

为 code block 实现一个新的策略: 对 note 类型的 code block ,将第一行文字的字体放大加粗倾斜, 并在表格右上角插入一个小提示图片

---

在结果 docx 上显示了 HTML 标签而不是格式化后的内容

---

能否始终将单个#开头的标题下移到下一页的开头?

---

代码块中的#看不到了. 如:

```bash
# 调试构建
cargo build
# 生产构建
cargo build --release
```

---

默认不要在code block中支持Markdown语法

---

若图片URL包含"?"则将后面的参数用在图片格式控制上, 例如对图片:

![img](img/llm-no-agent.png?width=100&height=50)

进行渲染时需要将图片的宽高设为100和50