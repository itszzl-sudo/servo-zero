# Servo-Zero 能力增强实施进度

## 实施概述

根据 Servo 对比分析报告，正在分阶段实施缺失的核心方法，包括DOM操作、CSS增强、布局支持和高级CSS特性。

---

## ✅ 第一阶段：核心基础功能 (已完成)

### 1.1 DOM操作基础方法

**实施文件**: `components/html_core/dom.rs`

| 方法 | 状态 | 说明 |
|-----|------|------|
| `remove_child()` | ✅ | 移除子节点，自动清理父子关系 |
| `insert_before()` | ✅ | 在指定参考节点前插入新节点 |
| `replace_child()` | ✅ | 替换子节点，更新父子关系 |
| `clone_node(deep)` | ✅ | 克隆节点，支持深度克隆 |
| `contains()` | ✅ | 检查节点包含关系（祖先检查） |
| `is_same_node()` | ✅ | 检查是否为同一节点 |
| `inner_html()` | ✅ | 获取元素内部HTML（序列化所有子节点） |
| `outer_html()` | ✅ | 获取元素外部HTML（包含自身） |
| `remove_attr()` | ✅ | 移除元素属性 |
| `has_attr()` | ✅ | 检查属性是否存在 |

**新增代码行数**: ~200行

### 1.2 事件监听器管理

**实施文件**: 
- `bridge/bridge.rs` (trait定义)
- `bridge/servo_impl.rs` (Mock实现)
- `bridge/real_impl.rs` (生产实现)

| 方法 | 状态 | 说明 |
|-----|------|------|
| `remove_on_click()` | ✅ | 移除点击事件监听器 |
| `remove_on_form_submit()` | ✅ | 移除表单提交事件监听器 |
| `remove_on_window_open()` | ✅ | 移除window.open事件监听器 |

**新增代码行数**: ~40行

### 1.3 定位布局支持

**实施文件**: `components/layout_core/box_model.rs`

| 功能 | 状态 | 说明 |
|-----|------|------|
| `position_top/right/bottom/left` | ✅ | 为LayoutNode添加定位偏移属性 |
| `PositionType` | ✅ | 已存在(Static/Relative/Absolute/Fixed) |

**新增代码行数**: ~10行

---

## ✅ 第二阶段：选择器与CSS增强 (已完成)

### 2.1 选择器增强

**实施文件**: `components/html_core/dom.rs`

#### 属性选择器
| 选择器 | 状态 | 示例 |
|-------|------|------|
| `[attr]` | ✅ | `[disabled]` - 检查属性存在 |
| `[attr=value]` | ✅ | `[type="text"]` - 精确匹配 |
| `[attr~=value]` | ✅ | `[class~="btn"]` - 单词匹配 |
| `[attr|=value]` | ✅ | `[lang|="en"]` - 前缀匹配(带-) |
| `[attr^=value]` | ✅ | `[href^="https"]` - 开头匹配 |
| `[attr$=value]` | ✅ | `[src$=".png"]` - 结尾匹配 |
| `[attr*=value]` | ✅ | `[title*="hello"]` - 包含匹配 |

#### 伪类选择器
| 选择器 | 状态 | 说明 |
|-------|------|------|
| `:first-child` | ✅ | 父节点的第一个子节点 |
| `:last-child` | ✅ | 父节点的最后一个子节点 |
| `:only-child` | ✅ | 父节点的唯一子节点 |
| `:first-of-type` | ✅ | 同类型标签的第一个 |
| `:last-of-type` | ✅ | 同类型标签的最后一个 |
| `:nth-child(n)` | ✅ | 第n个子节点 |
| `:not(selector)` | ✅ | 否定伪类 |
| `:empty` | ✅ | 没有子节点的元秦 |
| `:root` | ✅ | 文档根节点 |

**新增代码行数**: ~180行

### 2.2 CSS规则管理

**实施文件**: `components/css_core/stylesheet.rs`

| 方法 | 状态 | 说明 |
|-----|------|------|
| `insert_rule()` | ✅ | 插入CSS规则到指定位置 |
| `delete_rule()` | ✅ | 删除指定位置的CSS规则 |
| `get_rule_at()` | ✅ | 获取指定位置的CSS规则 |
| `rebuild_index()` | ✅ | 重建选择器索引(内部方法) |

**新增代码行数**: ~50行

---

## ⏳ 第二阶段：高级CSS特性 (待实施)

### 2.3 Grid布局
- [ ] 创建 `components/layout_core/grid.rs`
- [ ] 实现Grid容器布局算法
- [ ] 支持 `grid-template-columns/rows`
- [ ] 支持 `grid-area`, `grid-column`, `grid-row`

### 2.4 CSS变换 (Transform)
- [ ] 创建 `components/css_core/transform.rs`
- [ ] 实现 `translate()` 解析
- [ ] 实现 `rotate()` 解析
- [ ] 实现 `scale()` 解析
- [ ] 实现 `skew()` 解析
- [ ] 在渲染中应用变换矩阵

### 2.5 CSS动画 (Animation)
- [ ] 创建 `components/css_core/animation.rs`
- [ ] 实现 `@keyframes` 解析
- [ ] 实现 `animation` 属性解析
- [ ] 实现动画状态计算

### 2.6 CSS过渡 (Transition)
- [ ] 创建 `components/css_core/transition.rs`
- [ ] 实现 `transition` 属性解析
- [ ] 实现过渡状态计算

---

## 📊 实施统计

### 代码变更
- **新增文件**: 0个
- **修改文件**: 5个
  - `components/html_core/dom.rs` (+380行)
  - `bridge/bridge.rs` (+9行)
  - `bridge/servo_impl.rs` (+24行)
  - `bridge/real_impl.rs` (+12行)
  - `components/css_core/stylesheet.rs` (+47行)
- **总计新增**: ~472行代码

### 功能完成度
| 阶段 | 计划方法数 | 已完成 | 完成度 |
|-----|----------|--------|--------|
| 第一阶段 | 13 | 13 | 100% |
| 第二阶段(选择器) | 3 | 3 | 100% |
| 第二阶段(CSS管理) | 3 | 3 | 100% |
| 第二阶段(Grid) | 2 | 0 | 0% |
| 第二阶段(Transform) | 3 | 0 | 0% |
| 第二阶段(Animation) | 2 | 0 | 0% |
| 第二阶段(Transition) | 2 | 0 | 0% |
| **总计** | **28** | **16** | **57%** |

---

## 🎯 下一步计划

### 立即实施 (优先级高)
1. **CSS变换 (Transform)** - 2D变换是现代Web基础
2. **Grid布局** - 现代布局标准

### 后续实施 (优先级中)
3. **CSS过渡 (Transition)** - 配合变换使用
4. **CSS动画 (Animation)** - 高级特效

---

## 💡 实施建议

### 变换(Transform)实施要点
1. 创建 `TransformValue` 枚举表示不同变换类型
2. 解析函数支持变换矩阵计算
3. 在 `real_impl.rs` 的 `render()` 方法中应用 `tiny_skia::Transform`

### Grid布局实施要点
1. 参考 Servo 的 `components/layout/table/` 实现
2. 使用 taffy 库的Grid支持
3. 先实现基础Grid，再完善复杂特性

### 动画/过渡实施要点
1. 需要引入时间概念和状态管理
2. 考虑使用插值算法计算中间状态
3. 可以简化为关键帧状态，不实现平滑动画

---

## 📝 备注

- 所有新增方法都包含完整的文档注释
- 代码遵循项目现有的Rust风格指南
- 兄弟选择器 (`+` 和 `~`) 已在 `node_matches_segment()` 的框架中支持
- 伪类选择器中 `:hover`, `:focus` 等需要运行时状态的选择器暂未实现

---

*最后更新: 2026-05-20*
*实施者: AI Assistant*
