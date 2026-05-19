# servo-zero Feature 说明

## Feature 矩阵

| Feature | 说明 | 依赖 |
|---------|------|------|
| `default` | 完整浏览器功能 | html + network |
| `html` | HTML 解析 | html-core |
| `network` | 网络请求 | reqwest |
| `embed` | 嵌入模式 | network (无 html) |

## 使用场景

### default (完整模式)
```toml
servo-bridge = { path = "..." }
# 等价于
servo-bridge = { path = "...", features = ["html", "network"] }
```

**包含**:
- ✅ HTML 解析 (html-core)
- ✅ CSS 布局 (css-core)
- ✅ 渲染 (tiny-skia)
- ✅ 网络请求 (reqwest)
- ✅ 文件操作
- ✅ 事件系统

**用途**: 完整浏览器功能

### embed (嵌入模式)
```toml
servo-bridge = { path = "...", default-features = false, features = ["embed"] }
# 等价于
servo-bridge = { path = "...", default-features = false, features = ["network"] }
```

**包含**:
- ❌ HTML 解析 (移除，减少依赖)
- ✅ CSS 布局
- ✅ 渲染
- ✅ 网络请求
- ✅ 文件操作
- ✅ 事件系统

**用途**: 
- 嵌入其他程序
- 体积优化
- 启动更快

### 最小模式
```toml
servo-bridge = { path = "...", default-features = false }
```

**包含**:
- ❌ HTML 解析
- ❌ 网络请求
- ✅ CSS 布局
- ✅ 渲染
- ✅ 事件系统

**用途**:
- 仅渲染已有 HTML
- 最小体积
- 无外部依赖

## 依赖对比

| 模式 | 依赖数 | 编译时间 | 二进制大小 |
|------|--------|---------|-----------|
| default | ~150 | ~25s | ~15MB |
| embed | ~100 | ~15s | ~10MB |
| 最小 | ~50 | ~5s | ~5MB |

## @irisverse/jade 使用

`@irisverse/jade` 内部使用 `embed` 模式：

```toml
# jrust-browser/Cargo.toml
servo-bridge = { 
    path = "...", 
    default-features = false, 
    features = ["embed"] 
}
```

**原因**:
- 用户 JS 已编译，无需 HTML 解析
- 需要网络请求（下载资源）
- 需要文件操作（输出）
- 体积更小，启动更快
