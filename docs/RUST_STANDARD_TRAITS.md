## Rust 标准库常用 Trait 系统讲解

本文档系统地讲解 Rust 标准库中的常用 Trait，包括其底层原理、使用场景和代码示例。

### 目录

1. [基础 Trait](#基础-trait)
2. [自动 Trait](#自动-trait)
3. [Blanket Trait](#blanket-trait)
4. [标记结构体](#标记结构体)

---

### 基础 Trait

#### 1. Clone

**底层原理：**
`Clone` trait 提供了显式复制值的能力。与 `Copy` 不同，`Clone` 可以执行深拷贝，允许自定义复制逻辑。

```rust
pub trait Clone {
    fn clone(&self) -> Self;
    fn clone_from(&mut self, source: &Self) { ... }
}
```

**使用场景：**
- 需要显式复制堆上数据（如 `String`、`Vec<T>`）
- 复制操作可能代价较高，需要明确表示
- 实现自定义的深拷贝逻辑

**代码示例：**

```rust
#[derive(Clone)]
struct Person {
    name: String,
    age: u32,
}

fn main() {
    let person1 = Person {
        name: String::from("Alice"),
        age: 30,
    };
    
    // 显式调用 clone
    let person2 = person1.clone();
    
    println!("Person1: {}, {}", person1.name, person1.age);
    println!("Person2: {}, {}", person2.name, person2.age);
}
```

**手动实现 Clone：**

```rust
struct CustomData {
    data: Vec<i32>,
}

impl Clone for CustomData {
    fn clone(&self) -> Self {
        println!("Cloning CustomData...");
        CustomData {
            data: self.data.clone(),
        }
    }
}
```

---

#### 2. Copy

**底层原理：**
`Copy` 是一个标记 trait（marker trait），表示类型的值可以通过简单的位复制（bitwise copy）来复制。实现 `Copy` 的类型在赋值时会自动复制，而不是移动所有权。

```rust
pub trait Copy: Clone { }
```

**使用场景：**
- 简单的栈上数据类型（如整数、浮点数、布尔值）
- 所有字段都实现了 `Copy` 的结构体
- 不包含堆分配或其他资源的类型

**代码示例：**

```rust
#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p1 = Point { x: 10, y: 20 };
    let p2 = p1; // 自动复制，p1 仍然有效
    
    println!("p1: ({}, {})", p1.x, p1.y);
    println!("p2: ({}, {})", p2.x, p2.y);
}
```

**Clone vs Copy 对比：**

| 特性 | Clone | Copy |
|------|-------|------|
| 复制方式 | 显式调用 `.clone()` | 隐式自动复制 |
| 性能开销 | 可能较高（深拷贝） | 低（位复制） |
| 适用类型 | 任何类型 | 仅简单类型 |
| 堆分配 | 可以包含 | 不能包含 |
| 实现要求 | 独立实现 | 必须先实现 Clone |

---

#### 3. Debug

**底层原理：**
`Debug` trait 用于格式化输出，主要用于调试目的。使用 `{:?}` 或 `{:#?}` 格式化符号。

```rust
pub trait Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result;
}
```

**使用场景：**
- 调试输出
- 日志记录
- 错误信息展示

**代码示例：**

```rust
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

fn main() {
    let rect = Rectangle { width: 30, height: 50 };
    
    // 普通 Debug 输出
    println!("{:?}", rect);
    
    // 美化 Debug 输出
    println!("{:#?}", rect);
}
```

**手动实现 Debug：**

```rust
use std::fmt;

struct Person {
    name: String,
    age: u32,
}

impl fmt::Debug for Person {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Person")
            .field("name", &self.name)
            .field("age", &self.age)
            .finish()
    }
}
```

---

#### 4. Default

**底层原理：**
`Default` trait 提供创建类型默认值的方法。

```rust
pub trait Default {
    fn default() -> Self;
}
```

**使用场景：**
- 创建类型的默认实例
- 结构体部分初始化（使用 `..Default::default()`）
- 泛型编程中需要默认值

**代码示例：**

```rust
#[derive(Default, Debug)]
struct Config {
    host: String,      // 默认为空字符串
    port: u16,         // 默认为 0
    timeout: u32,      // 默认为 0
}

fn main() {
    // 使用默认值
    let config1 = Config::default();
    println!("{:?}", config1);
    
    // 部分字段使用默认值
    let config2 = Config {
        host: String::from("localhost"),
        port: 8080,
        ..Default::default()
    };
    println!("{:?}", config2);
}
```

**手动实现 Default：**

```rust
struct Connection {
    url: String,
    timeout: u32,
}

impl Default for Connection {
    fn default() -> Self {
        Connection {
            url: String::from("http://localhost:8080"),
            timeout: 30,
        }
    }
}
```

---

#### 5. PartialEq

**底层原理：**
`PartialEq` 定义了部分等价关系，允许使用 `==` 和 `!=` 运算符。"部分"意味着不是所有值都可以比较（如 `NaN`）。

```rust
pub trait PartialEq<Rhs = Self> {
    fn eq(&self, other: &Rhs) -> bool;
    fn ne(&self, other: &Rhs) -> bool { !self.eq(other) }
}
```

**使用场景：**
- 比较两个值是否相等
- 实现自定义相等逻辑
- 浮点数比较（不满足完全等价关系）

**代码示例：**

```rust
#[derive(PartialEq, Debug)]
struct Book {
    title: String,
    isbn: String,
}

fn main() {
    let book1 = Book {
        title: String::from("Rust Programming"),
        isbn: String::from("123-456"),
    };
    
    let book2 = Book {
        title: String::from("Rust Programming"),
        isbn: String::from("123-456"),
    };
    
    println!("Books equal: {}", book1 == book2); // true
}
```

**手动实现 PartialEq（仅比较 ISBN）：**

```rust
struct Book {
    title: String,
    isbn: String,
}

impl PartialEq for Book {
    fn eq(&self, other: &Self) -> bool {
        self.isbn == other.isbn
    }
}
```

---

#### 6. Eq

**底层原理：**
`Eq` 是 `PartialEq` 的子 trait，表示完全等价关系（自反性、对称性、传递性）。它是一个标记 trait，没有额外方法。

```rust
pub trait Eq: PartialEq<Self> { }
```

**使用场景：**
- 作为 `HashMap` 的键
- 需要完全等价关系的场景
- 不包含浮点数的类型

**代码示例：**

```rust
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Debug)]
struct UserId(u32);

fn main() {
    let mut users = HashMap::new();
    users.insert(UserId(1), "Alice");
    users.insert(UserId(2), "Bob");
    
    println!("{:?}", users.get(&UserId(1)));
}
```

**PartialEq vs Eq 对比：**

| 特性 | PartialEq | Eq |
|------|-----------|-----|
| 等价关系 | 部分等价 | 完全等价 |
| 自反性 | 不保证 `x == x` | 保证 `x == x` |
| 浮点数 | 支持（NaN != NaN） | 不支持 |
| HashMap 键 | 不可用 | 可用 |

---

#### 7. PartialOrd

**底层原理：**
`PartialOrd` 定义了部分排序关系，允许使用 `<`、`<=`、`>`、`>=` 运算符。

```rust
pub trait PartialOrd<Rhs = Self>: PartialEq<Rhs> {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;
    fn lt(&self, other: &Rhs) -> bool { ... }
    fn le(&self, other: &Rhs) -> bool { ... }
    fn gt(&self, other: &Rhs) -> bool { ... }
    fn ge(&self, other: &Rhs) -> bool { ... }
}
```

**使用场景：**
- 比较大小
- 排序操作
- 包含浮点数的类型

**代码示例：**

```rust
#[derive(PartialEq, PartialOrd, Debug)]
struct Temperature {
    celsius: f64,
}

fn main() {
    let t1 = Temperature { celsius: 20.0 };
    let t2 = Temperature { celsius: 25.0 };
    
    println!("t1 < t2: {}", t1 < t2); // true
    println!("t1 >= t2: {}", t1 >= t2); // false
}
```

**手动实现 PartialOrd：**

```rust
use std::cmp::Ordering;

struct Person {
    name: String,
    age: u32,
}

impl PartialEq for Person {
    fn eq(&self, other: &Self) -> bool {
        self.age == other.age
    }
}

impl PartialOrd for Person {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.age.partial_cmp(&other.age)
    }
}
```

---

#### 8. Ord

**底层原理：**
`Ord` 是 `PartialOrd` 的子 trait，表示完全排序关系。所有值都可以比较。

```rust
pub trait Ord: Eq + PartialOrd<Self> {
    fn cmp(&self, other: &Self) -> Ordering;
    fn max(self, other: Self) -> Self { ... }
    fn min(self, other: Self) -> Self { ... }
    fn clamp(self, min: Self, max: Self) -> Self { ... }
}
```

**使用场景：**
- 作为 `BTreeMap` 的键
- 需要完全排序的场景
- 不包含浮点数的类型

**代码示例：**

```rust
use std::collections::BTreeMap;
use std::cmp::Ordering;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Priority(u32);

fn main() {
    let mut tasks = BTreeMap::new();
    tasks.insert(Priority(3), "Low priority");
    tasks.insert(Priority(1), "High priority");
    tasks.insert(Priority(2), "Medium priority");
    
    for (priority, task) in &tasks {
        println!("{:?}: {}", priority, task);
    }
}
```

**PartialOrd vs Ord 对比：**

| 特性 | PartialOrd | Ord |
|------|------------|-----|
| 排序关系 | 部分排序 | 完全排序 |
| 返回值 | `Option<Ordering>` | `Ordering` |
| 浮点数 | 支持 | 不支持 |
| BTreeMap 键 | 不可用 | 可用 |

---

#### 9. Hash

**底层原理：**
`Hash` trait 用于计算类型的哈希值，主要用于哈希表（如 `HashMap`、`HashSet`）。

```rust
pub trait Hash {
    fn hash<H: Hasher>(&self, state: &mut H);
    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H) { ... }
}
```

**使用场景：**
- 作为 `HashMap` 或 `HashSet` 的键
- 需要快速查找的数据结构
- 实现自定义哈希逻辑

**代码示例：**

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Debug)]
struct Product {
    id: u32,
    name: String,
}

impl Hash for Product {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // 只根据 id 计算哈希值
        self.id.hash(state);
    }
}

fn main() {
    let mut inventory = HashMap::new();
    inventory.insert(
        Product { id: 1, name: String::from("Laptop") },
        10
    );
    
    let key = Product { id: 1, name: String::from("Different Name") };
    println!("Stock: {:?}", inventory.get(&key)); // Some(10)
}
```

---

#### 10. Variance（型变）

**底层原理：**
Variance 不是一个 trait，而是 Rust 类型系统的一个概念，描述泛型类型参数的子类型关系如何影响整个类型的子类型关系。

**三种 Variance：**

| Variance | 定义 | 示例 |
|----------|------|------|
| Covariant（协变） | 如果 `T` 是 `U` 的子类型，则 `F<T>` 是 `F<U>` 的子类型 | `&'a T`、`Box<T>` |
| Contravariant（逆变） | 如果 `T` 是 `U` 的子类型，则 `F<U>` 是 `F<T>` 的子类型 | `fn(T)` |
| Invariant（不变） | `F<T>` 和 `F<U>` 没有子类型关系 | `&mut T`、`Cell<T>` |

**使用场景：**
- 理解生命周期的子类型关系
- 设计安全的 API
- 避免类型系统漏洞

**代码示例：**

```rust
// 协变示例：&'a T 对 'a 和 T 都是协变的
fn covariant_example() {
    let s = String::from("hello");
    let r: &'static str = "world";
    
    // 'static 是所有生命周期的子类型
    // &'static str 可以转换为 &'a str
    let _: &str = r;
}

// 不变示例：&mut T 对 T 是不变的
fn invariant_example() {
    let mut s = String::from("hello");
    let r: &mut String = &mut s;
    
    // 不能将 &mut String 转换为 &mut &str
    // let _: &mut &str = r; // 编译错误
}

// 逆变示例：函数参数
fn contravariant_example() {
    // fn(&'static str) 可以用在需要 fn(&'a str) 的地方
    fn print_static(s: &'static str) {
        println!("{}", s);
    }
    
    fn use_fn<'a>(f: fn(&'a str), s: &'a str) {
        f(s);
    }
    
    let s = String::from("hello");
    use_fn(print_static, &s);
}
```

---

### 自动 Trait

自动 trait（Auto Traits）是编译器自动为类型实现的 trait，无需手动实现。

#### 1. Sized

**底层原理：**
`Sized` 表示类型在编译时具有已知的固定大小。大多数类型都自动实现 `Sized`。

```rust
pub trait Sized { }
```

**使用场景：**
- 默认情况下，泛型参数隐式要求 `Sized`
- 使用 `?Sized` 放宽限制，允许动态大小类型（DST）

**代码示例：**

```rust
// 默认要求 T: Sized
fn generic_function<T>(value: T) {
    // T 必须是固定大小的
}

// 放宽 Sized 限制
fn flexible_function<T: ?Sized>(value: &T) {
    // T 可以是动态大小类型，如 str 或 [i32]
}

fn main() {
    let s: &str = "hello";
    flexible_function(s); // str 是 DST
    
    let arr: &[i32] = &[1, 2, 3];
    flexible_function(arr); // [i32] 是 DST
}
```

**Sized vs ?Sized 对比：**

| 特性 | Sized | ?Sized |
|------|-------|--------|
| 大小 | 编译时已知 | 可能运行时确定 |
| 默认行为 | 泛型默认要求 | 需要显式指定 |
| 示例类型 | `i32`、`String`、`Vec<T>` | `str`、`[T]`、`dyn Trait` |
| 使用方式 | 直接使用 | 通过引用或智能指针 |

---

#### 2. Send

**底层原理：**
`Send` 表示类型的所有权可以安全地在线程间转移。大多数类型都自动实现 `Send`。

```rust
pub unsafe auto trait Send { }
```

**使用场景：**
- 多线程编程
- 跨线程传递数据
- 并发数据结构

**代码示例：**

```rust
use std::thread;

#[derive(Debug)]
struct Data {
    value: i32,
}

fn main() {
    let data = Data { value: 42 };
    
    // Data 实现了 Send，可以移动到新线程
    let handle = thread::spawn(move || {
        println!("Data in thread: {:?}", data);
    });
    
    handle.join().unwrap();
}
```

**不实现 Send 的类型：**

```rust
use std::rc::Rc;
use std::thread;

fn main() {
    let rc = Rc::new(42);
    
    // 编译错误：Rc<T> 不实现 Send
    // let handle = thread::spawn(move || {
    //     println!("{}", rc);
    // });
}
```

**负实现（Negative Implementation）：**

```rust
use std::marker::PhantomData;
use std::rc::Rc;

struct NotSend {
    _marker: PhantomData<Rc<()>>,
}

// 显式声明不实现 Send
impl !Send for NotSend {}
```

---

#### 3. Sync

**底层原理：**
`Sync` 表示类型的引用可以安全地在线程间共享。如果 `&T` 是 `Send`，则 `T` 是 `Sync`。

```rust
pub unsafe auto trait Sync { }
```

**使用场景：**
- 多线程共享数据
- 实现线程安全的数据结构
- 静态变量

**代码示例：**

```rust
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
struct SharedData {
    value: i32,
}

fn main() {
    let data = Arc::new(SharedData { value: 42 });
    
    let mut handles = vec![];
    
    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            println!("Thread {}: {:?}", i, data_clone);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

**不实现 Sync 的类型：**

```rust
use std::cell::Cell;

fn main() {
    let cell = Cell::new(42);
    
    // 编译错误：Cell<T> 不实现 Sync
    // let handle = thread::spawn(|| {
    //     println!("{}", cell.get());
    // });
}
```

**Send vs Sync 对比：**

| 特性 | Send | Sync |
|------|------|------|
| 含义 | 所有权可跨线程转移 | 引用可跨线程共享 |
| 关系 | `T: Send` | `&T: Send` ⟺ `T: Sync` |
| 不满足的类型 | `Rc<T>` | `Cell<T>`、`RefCell<T>` |
| 使用场景 | `thread::spawn(move \|\| ...)` | `Arc<T>` 共享 |

---

#### 4. Unpin

**底层原理：**
`Unpin` 表示类型在内存中移动是安全的。大多数类型都自动实现 `Unpin`。与 `Pin` 配合使用。

```rust
pub auto trait Unpin { }
```

**使用场景：**
- 异步编程（async/await）
- 自引用结构体
- 需要固定内存位置的类型

**代码示例：**

```rust
use std::pin::Pin;

struct Data {
    value: i32,
}

fn main() {
    let data = Data { value: 42 };
    
    // Data 实现了 Unpin，可以安全地从 Pin 中取出
    let mut pinned = Box::pin(data);
    let unpinned = Pin::into_inner(pinned);
    
    println!("Value: {}", unpinned.value);
}
```

**不实现 Unpin 的类型（使用 PhantomPinned）：**

```rust
use std::marker::PhantomPinned;
use std::pin::Pin;

struct SelfReferential {
    data: String,
    pointer: *const String,
    _pin: PhantomPinned,
}

impl SelfReferential {
    fn new(data: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(SelfReferential {
            data,
            pointer: std::ptr::null(),
            _pin: PhantomPinned,
        });
        
        // 设置自引用指针
        let ptr = &boxed.data as *const String;
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).pointer = ptr;
        }
        
        boxed
    }
}
```

---

### Blanket Trait

Blanket Trait 是为所有满足特定条件的类型自动实现的 trait。

#### 1. Any

**底层原理：**
`Any` trait 允许在运行时进行类型检查和转换，实现类型擦除。

```rust
pub trait Any: 'static {
    fn type_id(&self) -> TypeId;
}
```

**使用场景：**
- 动态类型检查
- 类型擦除
- 插件系统

**代码示例：**

```rust
use std::any::Any;

fn print_if_string(value: &dyn Any) {
    if let Some(s) = value.downcast_ref::<String>() {
        println!("String: {}", s);
    } else if let Some(i) = value.downcast_ref::<i32>() {
        println!("i32: {}", i);
    } else {
        println!("Unknown type");
    }
}

fn main() {
    let s = String::from("hello");
    let i = 42;
    
    print_if_string(&s);
    print_if_string(&i);
}
```

---

#### 2. Borrow<T> 和 BorrowMut<T>

**底层原理：**
`Borrow` 和 `BorrowMut` 用于抽象借用操作，允许从拥有的值或引用中借用数据。

```rust
pub trait Borrow<Borrowed: ?Sized> {
    fn borrow(&self) -> &Borrowed;
}

pub trait BorrowMut<Borrowed: ?Sized>: Borrow<Borrowed> {
    fn borrow_mut(&mut self) -> &mut Borrowed;
}
```

**使用场景：**
- `HashMap` 和 `BTreeMap` 的键查找
- 统一处理拥有值和借用值
- 泛型编程

**代码示例：**

```rust
use std::collections::HashMap;
use std::borrow::Borrow;

fn main() {
    let mut map = HashMap::new();
    map.insert(String::from("key"), 42);
    
    // 可以使用 &str 查找 String 键
    let key: &str = "key";
    println!("Value: {:?}", map.get(key));
    
    // 也可以使用 String
    let owned_key = String::from("key");
    println!("Value: {:?}", map.get(&owned_key));
}
```

**自定义实现：**

```rust
use std::borrow::Borrow;

struct Wrapper(String);

impl Borrow<str> for Wrapper {
    fn borrow(&self) -> &str {
        &self.0
    }
}

fn print_borrowed<T: Borrow<str>>(value: T) {
    println!("{}", value.borrow());
}

fn main() {
    let wrapper = Wrapper(String::from("hello"));
    print_borrowed(wrapper);
    print_borrowed("world");
}
```

---

#### 3. From<T> 和 Into<U>

**底层原理：**
`From` 和 `Into` 用于类型转换。实现 `From<T>` 会自动获得 `Into<U>` 的实现。

```rust
pub trait From<T> {
    fn from(value: T) -> Self;
}

pub trait Into<T> {
    fn into(self) -> T;
}
```

**使用场景：**
- 类型转换
- 错误处理（`?` 操作符）
- API 设计（接受多种类型）

**代码示例：**

```rust
#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
}

impl From<(&str, u32)> for Person {
    fn from((name, age): (&str, u32)) -> Self {
        Person {
            name: name.to_string(),
            age,
        }
    }
}

fn main() {
    // 使用 From
    let person1 = Person::from(("Alice", 30));
    println!("{:?}", person1);
    
    // 使用 Into（自动实现）
    let person2: Person = ("Bob", 25).into();
    println!("{:?}", person2);
}
```

**在函数参数中使用：**

```rust
fn create_person<T: Into<String>>(name: T, age: u32) -> Person {
    Person {
        name: name.into(),
        age,
    }
}

fn main() {
    let p1 = create_person("Alice", 30);
    let p2 = create_person(String::from("Bob"), 25);
}
```

**From vs Into 对比：**

| 特性 | From | Into |
|------|------|------|
| 实现方式 | 手动实现 | 自动派生自 From |
| 使用场景 | 定义转换逻辑 | 调用转换 |
| 推荐 | 优先实现 From | 优先使用 Into |
| 类型推断 | 明确 | 可能需要类型注解 |

---

#### 4. TryFrom<U> 和 TryInto<U>

**底层原理：**
`TryFrom` 和 `TryInto` 用于可能失败的类型转换，返回 `Result`。

```rust
pub trait TryFrom<T>: Sized {
    type Error;
    fn try_from(value: T) -> Result<Self, Self::Error>;
}

pub trait TryInto<T>: Sized {
    type Error;
    fn try_into(self) -> Result<T, Self::Error>;
}
```

**使用场景：**
- 可能失败的转换（如范围检查）
- 解析操作
- 验证输入

**代码示例：**

```rust
use std::convert::TryFrom;

#[derive(Debug)]
struct Age(u8);

#[derive(Debug)]
enum AgeError {
    TooYoung,
    TooOld,
}

impl TryFrom<i32> for Age {
    type Error = AgeError;
    
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            Err(AgeError::TooYoung)
        } else if value > 150 {
            Err(AgeError::TooOld)
        } else {
            Ok(Age(value as u8))
        }
    }
}

fn main() {
    match Age::try_from(25) {
        Ok(age) => println!("Valid age: {:?}", age),
        Err(e) => println!("Error: {:?}", e),
    }
    
    match Age::try_from(200) {
        Ok(age) => println!("Valid age: {:?}", age),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

---

#### 5. ToOwned

**底层原理：**
`ToOwned` 用于从借用数据创建拥有的数据，是 `Clone` 的泛化版本。

```rust
pub trait ToOwned {
    type Owned: Borrow<Self>;
    fn to_owned(&self) -> Self::Owned;
    fn clone_into(&self, target: &mut Self::Owned) { ... }
}
```

**使用场景：**
- 从 `&str` 创建 `String`
- 从 `&[T]` 创建 `Vec<T>`
- Cow（Clone on Write）类型

**代码示例：**

```rust
fn main() {
    // &str -> String
    let s: &str = "hello";
    let owned: String = s.to_owned();
    println!("{}", owned);
    
    // &[i32] -> Vec<i32>
    let slice: &[i32] = &[1, 2, 3];
    let vec: Vec<i32> = slice.to_owned();
    println!("{:?}", vec);
}
```

**使用 Cow：**

```rust
use std::borrow::Cow;

fn process_string(s: &str) -> Cow<str> {
    if s.contains("hello") {
        // 需要修改，返回拥有的数据
        Cow::Owned(s.replace("hello", "hi"))
    } else {
        // 不需要修改，返回借用
        Cow::Borrowed(s)
    }
}

fn main() {
    let s1 = "hello world";
    let s2 = "goodbye";
    
    println!("{}", process_string(s1)); // Owned
    println!("{}", process_string(s2)); // Borrowed
}
```

**Clone vs ToOwned 对比：**

| 特性 | Clone | ToOwned |
|------|-------|---------|
| 输入类型 | `&Self` | `&Self` |
| 输出类型 | `Self` | `Self::Owned` |
| 灵活性 | 相同类型 | 可以不同类型 |
| 示例 | `String -> String` | `&str -> String` |

---

### 标记结构体

#### 1. PhantomData<T>

**底层原理：**
`PhantomData<T>` 是一个零大小类型（ZST），用于标记类型参数的所有权、生命周期或 variance，而不实际存储该类型的值。

```rust
pub struct PhantomData<T: ?Sized>;
```

**使用场景：**
- 未使用的类型参数
- 控制 variance
- 实现 Drop Check
- 标记所有权关系

**代码示例 1：未使用的类型参数**

```rust
use std::marker::PhantomData;

struct Slice<'a, T> {
    start: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Slice<'a, T> {
    fn new(slice: &'a [T]) -> Self {
        Slice {
            start: slice.as_ptr(),
            end: unsafe { slice.as_ptr().add(slice.len()) },
            _marker: PhantomData,
        }
    }
}
```

**代码示例 2：控制 Variance**

```rust
use std::marker::PhantomData;

// 对 T 协变
struct Covariant<T> {
    _marker: PhantomData<T>,
}

// 对 T 不变
struct Invariant<T> {
    _marker: PhantomData<fn(T) -> T>,
}

// 对 T 逆变
struct Contravariant<T> {
    _marker: PhantomData<fn(T)>,
}
```

**代码示例 3：Drop Check**

```rust
use std::marker::PhantomData;

struct Inspector<'a, T: 'a> {
    data: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Drop for Inspector<'a, T> {
    fn drop(&mut self) {
        // PhantomData 确保 T 在这里仍然有效
        unsafe {
            println!("Dropping inspector");
        }
    }
}
```

**PhantomData 的常见用法：**

| 用法 | PhantomData 类型 | 效果 |
|------|------------------|------|
| 拥有 T | `PhantomData<T>` | 对 T 协变，拥有 T |
| 借用 T | `PhantomData<&'a T>` | 对 'a 和 T 协变 |
| 可变借用 T | `PhantomData<&'a mut T>` | 对 'a 协变，对 T 不变 |
| 不变 T | `PhantomData<fn(T) -> T>` | 对 T 不变 |
| 逆变 T | `PhantomData<fn(T)>` | 对 T 逆变 |

---

#### 2. PhantomPinned

**底层原理：**
`PhantomPinned` 用于标记类型不实现 `Unpin`，表示该类型不能在内存中安全移动。

```rust
pub struct PhantomPinned;
```

**使用场景：**
- 自引用结构体
- 异步编程中的 Future
- 需要固定内存位置的类型

**代码示例：自引用结构体**

```rust
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::NonNull;

struct SelfReferential {
    data: String,
    // 指向 self.data 的指针
    self_ptr: Option<NonNull<String>>,
    // 标记为 !Unpin
    _pin: PhantomPinned,
}

impl SelfReferential {
    fn new(data: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(SelfReferential {
            data,
            self_ptr: None,
            _pin: PhantomPinned,
        });
        
        // 初始化自引用指针
        let self_ptr = NonNull::from(&boxed.data);
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).self_ptr = Some(self_ptr);
        }
        
        boxed
    }
    
    fn get_data(&self) -> &str {
        &self.data
    }
    
    fn get_self_ptr_data(&self) -> &str {
        unsafe {
            self.self_ptr.unwrap().as_ref()
        }
    }
}

fn main() {
    let pinned = SelfReferential::new(String::from("hello"));
    
    println!("Data: {}", pinned.get_data());
    println!("Self ptr data: {}", pinned.get_self_ptr_data());
    
    // 无法移动 pinned，因为它不实现 Unpin
    // let moved = *pinned; // 编译错误
}
```

**异步编程示例：**

```rust
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::future::Future;
use std::task::{Context, Poll};

struct MyFuture {
    state: i32,
    _pin: PhantomPinned,
}

impl Future for MyFuture {
    type Output = i32;
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 安全地访问 pinned 数据
        let state = unsafe { &mut self.get_unchecked_mut().state };
        *state += 1;
        
        if *state >= 3 {
            Poll::Ready(*state)
        } else {
            Poll::Pending
        }
    }
}
```

---

### 总结对比表

#### 基础 Trait 对比

| Trait | 自动派生 | 主要用途 | 性能影响 |
|-------|---------|---------|---------|
| Clone | 可以 | 显式复制 | 可能较高 |
| Copy | 可以 | 隐式复制 | 低 |
| Debug | 可以 | 调试输出 | 无 |
| Default | 可以 | 默认值 | 无 |
| PartialEq | 可以 | 相等比较 | 低 |
| Eq | 可以 | 完全相等 | 无 |
| PartialOrd | 可以 | 大小比较 | 低 |
| Ord | 可以 | 完全排序 | 低 |
| Hash | 可以 | 哈希计算 | 低到中 |

#### 自动 Trait 对比

| Trait | 自动实现 | 负实现 | 主要用途 |
|-------|---------|--------|---------|
| Sized | 是 | 使用 `?Sized` | 编译时大小已知 |
| Send | 是 | 可以 | 跨线程转移所有权 |
| Sync | 是 | 可以 | 跨线程共享引用 |
| Unpin | 是 | 使用 PhantomPinned | 可安全移动 |

#### 转换 Trait 对比

| Trait | 可失败 | 自动实现 | 推荐实现 |
|-------|--------|---------|---------|
| From | 否 | Into | From |
| Into | 否 | 是（从 From） | 使用 Into |
| TryFrom | 是 | TryInto | TryFrom |
| TryInto | 是 | 是（从 TryFrom） | 使用 TryInto |

---

### 最佳实践

1. **优先使用派生宏**：对于标准 trait，尽可能使用 `#[derive(...)]`
2. **实现 From 而非 Into**：编译器会自动提供 Into 实现
3. **Clone vs Copy**：只有简单类型才实现 Copy
4. **Eq 需要 PartialEq**：实现 Eq 前必须先实现 PartialEq
5. **Hash 需要 Eq**：用作 HashMap 键的类型必须同时实现 Hash 和 Eq
6. **使用 PhantomData**：处理未使用的类型参数或控制 variance
7. **谨慎使用 !Send 和 !Sync**：只在确实需要时才使用负实现
8. **Pin 和 Unpin**：异步编程中使用 PhantomPinned 标记自引用类型

---

### 参考资源

- [Rust 标准库文档](https://doc.rust-lang.org/std/)
- [The Rust Reference - Traits](https://doc.rust-lang.org/reference/items/traits.html)
- [Rust Nomicon - PhantomData](https://doc.rust-lang.org/nomicon/phantom-data.html)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
