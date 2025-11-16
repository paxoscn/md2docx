有个bug, 当 Markdown 中包含多个加粗文本且之间没有空格且其中包含特殊字符"）"时转换会卡住,例如:
**）** a **bbb**

以下代码似乎没能加粗:
```rust
pub mod bajie;
let y = 5;
```