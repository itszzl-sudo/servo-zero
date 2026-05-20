# Servo vs Servo-Zero 方法对比分析报告

## 报告说明

本报告对比了 Servo 浏览器引擎的核心业务逻辑方法与 servo-zero 项目的实现情况,重点关注以下三个核心组件:
- **DOM/HTML** (Servo: `components/script/dom` → servo-zero: `components/html_core`)
- **CSS** (Servo: `style` + `components/script/css` → servo-zero: `components/css_core`)
- **Layout** (Servo: `components/layout` → servo-zero: `components/layout_core`)

---

## 一、DOM/HTML 核心方法对比

### 1.1 节点操作方法 (Node Operations)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `query_selector()` | CSS选择器查找第一个元素 | ✅ 已实现 (`HtmlDocument::query_selector`) | - |
| `query_selector_all()` | CSS选择器查找所有元素 | ✅ 已实现 (`HtmlDocument::query_selector_all`) | - |
| `get_element_by_id()` | 通过ID获取元素 | ✅ 已实现 (通过 `query_selector("#id")`) | - |
| `parent_node()` | 获取父节点 | ✅ 已实现 (`HtmlDocument::parent`) | - |
| `child_nodes()` | 获取子节点列表 | ✅ 已实现 (`HtmlDocument::children`) | - |
| `first_child()` | 获取第一个子节点 | ⚠️ 部分实现 (需通过 `children()[0]`) | **建议新增**便捷方法 |
| `last_child()` | 获取最后一个子节点 | ⚠️ 部分实现 (需通过 `children().last()`) | **建议新增**便捷方法 |
| `next_sibling()` | 获取下一个兄弟节点 | ❌ 未实现 | **需要新增** |
| `previous_sibling()` | 获取上一个兄弟节点 | ❌ 未实现 | **需要新增** |
| `append_child()` | 添加子节点 | ✅ 已实现 (`HtmlDocument::append_child`) | - |
| `remove_child()` | 移除子节点 | ❌ 未实现 | **需要新增** (DOM操作基础) |
| `insert_before()` | 在指定位置插入节点 | ❌ 未实现 | **需要新增** (DOM操作基础) |
| `replace_child()` | 替换子节点 | ❌ 未实现 | **需要新增** |
| `clone_node()` | 克隆节点 | ❌ 未实现 | **需要新增** |
| `contains()` | 检查是否包含某节点 | ❌ 未实现 | **建议新增** |
| `is_same_node()` | 检查是否为同一节点 | ❌ 未实现 | **建议新增** |

### 1.2 元素属性操作 (Element Attributes)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `get_attribute()` | 获取属性值 | ✅ 已实现 (`DomNode::get_attr`) | - |
| `set_attribute()` | 设置属性值 | ✅ 已实现 (`DomNode::set_attr`) | - |
| `remove_attribute()` | 移除属性 | ❌ 未实现 | **需要新增** |
| `has_attribute()` | 检查是否有某属性 | ❌ 未实现 | **建议新增** |
| `get_attribute_node()` | 获取属性节点对象 | ❌ 未实现 | 不需要 (servo-zero 简化设计) |
| `set_attribute_node()` | 设置属性节点对象 | ❌ 未实现 | 不需要 (servo-zero 简化设计) |

### 1.3 文本内容操作 (Text Content)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `text_content()` | 获取文本内容 | ✅ 已实现 (`DomNode::text`, `HtmlDocument::collect_text`) | - |
| `inner_text()` | 获取渲染后的文本 | ⚠️ 部分实现 (`text_content` 获取原始文本) | **建议新增** (去除隐藏元素文本) |
| `inner_html()` | 获取内部HTML | ❌ 未实现 | **需要新增** (常用API) |
| `outer_html()` | 获取包含自身的HTML | ❌ 未实现 | **需要新增** (常用API) |

### 1.4 文档操作 (Document Operations)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `create_element()` | 创建元素节点 | ❌ 未实现 (由Parser内部创建) | **建议新增**公共方法 |
| `create_text_node()` | 创建文本节点 | ❌ 未实现 (由Parser内部创建) | **建议新增**公共方法 |
| `create_document_fragment()` | 创建文档片段 | ❌ 未实现 | **需要新增** (批量操作优化) |
| `import_node()` | 从其他文档导入节点 | ❌ 未实现 | 不需要 (servo-zero 单文档设计) |
| `adopt_node()` | 从其他文档采纳节点 | ❌ 未实现 | 不需要 (servo-zero 单文档设计) |

### 1.5 事件处理 (Event Handling)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `add_event_listener()` | 添加事件监听器 | ✅ 已实现 (通过 `WebNativeBridge::on_click` 等) | - |
| `remove_event_listener()` | 移除事件监听器 | ❌ 未实现 | **需要新增** |
| `dispatch_event()` | 分发事件 | ✅ 已实现 (通过 `WebNativeBridge::handle_click`) | - |
| `handle_event()` | 处理事件 (内部) | ✅ 已实现 (内部实现) | - |

---

## 二、CSS 核心方法对比

### 2.1 样式表管理 (Stylesheet Management)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `parse_stylesheet()` | 解析CSS文本 | ✅ 已实现 (`Stylesheet::from_str`) | - |
| `add_rule()` | 添加CSS规则 | ✅ 已实现 (`Stylesheet::add_rule`) | - |
| `insert_rule()` | 插入CSS规则到指定位置 | ❌ 未实现 | **需要新增** |
| `delete_rule()` | 删除CSS规则 | ❌ 未实现 | **需要新增** |
| `get_rule_at()` | 获取指定位置的规则 | ❌ 未实现 | **建议新增** |

### 2.2 样式级联与计算 (Cascade & Computation)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `resolve_style()` | 计算元素的最终样式 | ✅ 已实现 (`StyleResolver::resolve`) | - |
| `compute_style()` | 计算样式值 | ✅ 已实现 (`ComputedStyle`) | - |
| `match_rules()` | 匹配适用的CSS规则 | ✅ 已实现 (`StyleResolver::match_rules`) | - |
| `get_computed_value()` | 获取计算后的属性值 | ✅ 已实现 (`ComputedStyle` 字段访问) | - |
| `inherit_style()` | 从父元素继承样式 | ✅ 已实现 (级联过程中处理) | - |

### 2.3 CSS 属性解析 (Property Parsing)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `parse_color()` | 解析颜色值 | ✅ 已实现 (`parse_color`) | - |
| `parse_length()` | 解析长度值 | ✅ 已实现 (`Length::parse`) | - |
| `parse_background()` | 解析背景属性 | ⚠️ 部分实现 (仅支持颜色) | **建议增强** (支持图片、位置等) |
| `parse_border()` | 解析边框属性 | ⚠️ 部分实现 (基础支持) | **建议增强** (支持样式、圆角等) |
| `parse_transform()` | 解析变换属性 | ❌ 未实现 | 不需要 (servo-zero 2D渲染) |
| `parse_animation()` | 解析动画属性 | ❌ 未实现 | 不需要 (servo-zero 静态渲染) |
| `parse_transition()` | 解析过渡属性 | ❌ 未实现 | 不需要 (servo-zero 静态渲染) |

### 2.4 选择器匹配 (Selector Matching)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `match_simple_selector()` | 匹配简单选择器 | ✅ 已实现 | - |
| `match_compound_selector()` | 匹配复合选择器 | ✅ 已实现 | - |
| `match_descendant_selector()` | 匹配后代选择器 | ✅ 已实现 | - |
| `match_pseudo_class()` | 匹配伪类选择器 | ❌ 未实现 | **建议新增** (`:hover`, `:focus`, `:first-child` 等) |
| `match_pseudo_element()` | 匹配伪元素选择器 | ❌ 未实现 | 不需要 (servo-zero 简化设计) |
| `match_attribute_selector()` | 匹配属性选择器 | ❌ 未实现 | **需要新增** (`[attr=value]`) |
| `match_sibling_selector()` | 匹配兄弟选择器 | ❌ 未实现 | **建议新增** (`+`, `~`) |

---

## 三、Layout 核心方法对比

### 3.1 布局计算 (Layout Calculation)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `layout()` | 执行布局计算 | ✅ 已实现 (`LayoutTree::layout`) | - |
| `compute_damage()` | 计算布局损伤 (增量更新) | ❌ 未实现 | 不需要 (servo-zero 全量重排) |
| `reflow()` | 重排布局 | ✅ 已实现 (通过 `layout()` 触发) | - |
| `resolve_style_for_layout()` | 为布局解析样式 | ✅ 已实现 (内部处理) | - |

### 3.2 盒模型 (Box Model)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `compute_content_box()` | 计算内容盒 | ✅ 已实现 (`LayoutBox`) | - |
| `compute_padding_box()` | 计算填充盒 | ✅ 已实现 (`LayoutBox`) | - |
| `compute_border_box()` | 计算边框盒 | ✅ 已实现 (`LayoutBox`) | - |
| `compute_margin_box()` | 计算外边距盒 | ✅ 已实现 (`LayoutBox`) | - |
| `get_box()` | 获取元素的布局盒 | ✅ 已实现 (`LayoutTree::get_box`) | - |

### 3.3 Flexbox 布局 (Flexbox Layout)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `layout_flex_container()` | 布局Flex容器 | ✅ 已实现 (`FlexboxLayout`) | - |
| `layout_flex_items()` | 布局Flex子项 | ✅ 已实现 (`FlexboxLayout`) | - |
| `compute_flex_basis()` | 计算flex-basis | ✅ 已实现 | - |
| `compute_flex_grow()` | 计算flex-grow分配 | ✅ 已实现 | - |
| `compute_flex_shrink()` | 计算flex-shrink收缩 | ✅ 已实现 | - |

### 3.4 其他布局模式 (Other Layout Modes)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `layout_grid()` | Grid布局 | ❌ 未实现 | **建议新增** (现代Web必备) |
| `layout_table()` | 表格布局 | ❌ 未实现 | **建议新增** (兼容性需求) |
| `layout_block()` | 块级布局 | ✅ 已实现 (基础支持) | - |
| `layout_inline()` | 行内布局 | ⚠️ 部分实现 (基础文本) | **建议增强** (支持行内元素混排) |
| `layout_positioned()` | 定位布局 (relative/absolute/fixed) | ⚠️ 部分实现 | **需要新增** (常用布局需求) |
| `layout_float()` | 浮动布局 | ❌ 未实现 | **建议新增** (兼容性需求) |

### 3.5 渲染相关 (Rendering)

| Servo 方法 | 功能描述 | servo-zero 状态 | 建议 |
|-----------|---------|----------------|------|
| `build_display_list()` | 构建显示列表 | ✅ 已实现 (通过 `all_rects()`) | - |
| `hit_test()` | 命中测试 | ✅ 已实现 (`LayoutTree::hit_test`) | - |
| `get_rect()` | 获取元素矩形 | ✅ 已实现 (`LayoutTree::get_rect`) | - |
| `render()` | 渲染到图像 | ✅ 已实现 (`WebNativeBridge::render`) | - |

---

## 四、优先级建议

### 🔴 高优先级 (必须实现)

以下方法是DOM操作的基础,强烈建议补充:

1. **`remove_child()`** - 移除子节点
   - 位置: `html_core/dom.rs` → `HtmlDocument`
   - 用途: DOM操作基础,删除节点必需

2. **`insert_before()`** - 在指定位置插入节点
   - 位置: `html_core/dom.rs` → `HtmlDocument`
   - 用途: DOM操作基础,精确控制插入位置

3. **`replace_child()`** - 替换子节点
   - 位置: `html_core/dom.rs` → `HtmlDocument`
   - 用途: DOM操作基础,节点替换

4. **`inner_html()`** - 获取内部HTML
   - 位置: `html_core/dom.rs` → `HtmlDocument` / `DomNode`
   - 用途: 非常常用的API,获取元素HTML内容

5. **`outer_html()`** - 获取包含自身的HTML
   - 位置: `html_core/dom.rs` → `DomNode`
   - 用途: 常用API,获取完整元素HTML

6. **`remove_attribute()`** - 移除属性
   - 位置: `html_core/dom.rs` → `DomNode`
   - 用途: 属性操作基础

7. **`remove_event_listener()`** - 移除事件监听器
   - 位置: `bridge/bridge.rs` → `WebNativeBridge`
   - 用途: 事件管理必需,防止内存泄漏

8. **`layout_positioned()`** - 定位布局支持
   - 位置: `layout_core/` 新增模块或扩展现有布局
   - 用途: relative/absolute/fixed定位是现代Web基础

### 🟡 中优先级 (建议实现)

以下方法能显著提升功能完整性:

9. **`next_sibling()` / `previous_sibling()`** - 兄弟节点导航
   - 位置: `html_core/dom.rs` → `HtmlDocument`
   - 用途: DOM遍历常用操作

10. **`has_attribute()`** - 检查属性存在性
    - 位置: `html_core/dom.rs` → `DomNode`
    - 用途: 条件判断常用

11. **`clone_node()`** - 克隆节点
    - 位置: `html_core/dom.rs` → `DomNode`
    - 用途: 节点复制,模板复用

12. **`create_document_fragment()`** - 创建文档片段
    - 位置: `html_core/dom.rs` → `HtmlDocument`
    - 用途: 批量DOM操作优化性能

13. **`inner_text()`** - 获取渲染后文本
    - 位置: `html_core/dom.rs` → `DomNode`
    - 用途: 获取用户可见文本

14. **`match_attribute_selector()`** - 属性选择器
    - 位置: `html_core/dom.rs` → `HtmlDocument`
    - 用途: CSS选择器完整性 (`[attr=value]`)

15. **`match_pseudo_class()`** - 伪类选择器
    - 位置: `html_core/dom.rs` / `css_core/cascade.rs`
    - 用途: `:hover`, `:first-child` 等常用伪类

16. **`layout_grid()`** - Grid布局
    - 位置: `layout_core/` 新增 `grid.rs`
    - 用途: 现代Web布局标准

17. **`insert_rule()` / `delete_rule()`** - CSS规则管理
    - 位置: `css_core/stylesheet.rs` → `Stylesheet`
    - 用途: 动态CSS管理

### 🟢 低优先级 (按需实现)

以下方法根据项目需求决定:

18. **`first_child()` / `last_child()`** - 便捷访问方法
    - 用途: 虽然可通过 `children()[0]` 实现,但便捷方法更好用

19. **`contains()` / `is_same_node()`** - 节点关系检查
    - 用途: 特定场景需要

20. **`create_element()` / `create_text_node()`** - 节点创建
    - 用途: 如果需要编程式创建DOM则需要

21. **`layout_table()`** - 表格布局
    - 用途: 如需渲染复杂表格则需要

22. **`layout_float()`** - 浮动布局
    - 用途: 老式布局支持

23. **`parse_background()` 增强** - 背景属性完整支持
    - 用途: 如需完整CSS背景支持

24. **`parse_border()` 增强** - 边框属性完整支持
    - 用途: 如需完整CSS边框支持

---

## 五、不需要实现的方法

以下方法在servo-zero的简化设计下不需要实现:

### 5.1 JavaScript相关
- `eval_js()` 的详细实现 (servo-zero已有占位)
- JS事件绑定内部机制

### 5.2 高级CSS特性
- `parse_transform()` - 3D变换 (servo-zero专注2D)
- `parse_animation()` - 动画 (servo-zero静态渲染)
- `parse_transition()` - 过渡 (servo-zero静态渲染)
- `match_pseudo_element()` - 伪元素 (`::before`, `::after`)

### 5.3 复杂文档操作
- `import_node()` / `adopt_node()` - 跨文档操作 (servo-zero单文档)
- `get_attribute_node()` - 属性节点对象 (简化为键值对)

### 5.4 性能优化
- `compute_damage()` - 增量布局 (servo-zero全量重排)

---

## 六、总结

### 6.1 当前实现覆盖率

| 模块 | Servo方法总数 | servo-zero已实现 | 覆盖率 |
|-----|-------------|----------------|-------|
| DOM/HTML | ~25个核心方法 | ~12个 | ~48% |
| CSS | ~15个核心方法 | ~10个 | ~67% |
| Layout | ~15个核心方法 | ~10个 | ~67% |
| **总计** | **~55个核心方法** | **~32个** | **~58%** |

### 6.2 建议实施路线

**第一阶段** (高优先级):
- 补充DOM操作基础方法 (remove_child, insert_before, replace_child)
- 添加常用属性/内容方法 (inner_html, outer_html, remove_attribute)
- 添加事件监听器管理 (remove_event_listener)
- 实现定位布局支持

**第二阶段** (中优先级):
- 增强选择器支持 (属性选择器、伪类)
- 添加Grid布局
- 补充CSS规则管理
- 添加便捷DOM方法

**第三阶段** (低优先级):
- 按需实现表格、浮动等布局
- 增强CSS属性解析
- 性能优化 (如需要)

### 6.3 项目定位建议

servo-zero 作为轻量级渲染引擎,应该:
1. ✅ **保持简洁** - 不追求完整实现Servo的所有功能
2. ✅ **专注核心** - DOM操作、CSS布局、2D渲染是核心
3. ✅ **实用优先** - 优先实现常用API,高级特性按需添加
4. ✅ **性能考量** - 在功能完整性和性能之间找到平衡

---

## 附录:关键文件对照

| Servo 文件 | servo-zero 对应文件 | 说明 |
|-----------|-------------------|------|
| `components/script/dom/node/node.rs` | `components/html_core/dom.rs` | DOM节点定义 |
| `components/script/dom/element/element.rs` | `components/html_core/dom.rs` | 元素操作 |
| `components/script/dom/document/document.rs` | `components/html_core/dom.rs` | 文档操作 |
| `style/stylesheets/*.rs` | `components/css_core/stylesheet.rs` | 样式表 |
| `style/properties/*.rs` | `components/css_core/cascade.rs` | 样式级联 |
| `components/layout/layout_impl.rs` | `components/layout_core/tree.rs` | 布局树 |
| `components/layout/flexbox/*.rs` | `components/layout_core/flexbox.rs` | Flexbox布局 |
| `components/layout/flow/*.rs` | `components/layout_core/box_model.rs` | 盒模型 |
| `components/paint/painter.rs` | `bridge/real_impl.rs` | 渲染实现 |

---

*报告生成时间: 2026-05-20*
*分析基于 Servo 源码和 servo-zero 项目当前状态*
