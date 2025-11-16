# 代码块缩进测试

## 测试1：标准缩进（4个空格）

```python
    def function_with_indent():
        print("This line has 8 spaces")
        if True:
            print("This line has 12 spaces")
```

## 测试2：不同级别的缩进

```javascript
    function test() {
        console.log("4 spaces");
        if (true) {
            console.log("8 spaces");
            for (let i = 0; i < 10; i++) {
                console.log("12 spaces");
            }
        }
    }
```

## 测试3：混合空格和制表符

```java
    public class Test {
	public void method() {
	    System.out.println("Mixed tabs and spaces");
	}
    }
```

## 测试4：保留空行

```rust
    fn main() {
        println!("Line 1");
        
        println!("Line 3 with empty line above");
        
        
        println!("Line 6 with two empty lines above");
    }
```
