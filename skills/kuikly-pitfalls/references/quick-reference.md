# Kuikly 踩坑速查表

> 一页纸版本。详细内容见 `SKILL.md`。

## 按症状索引

| 症状 | 根因 | 一行修复 |
| --- | --- | --- |
| **左右边距不对称**（左窄右宽） | `Scroller` 上写了 padding，被转发给 contentView 且宽度算两次 | padding 移到内层 `View` |
| **数据变了 UI 不变** | 创建期读了快照，订阅从未建立 | 用 `vfor` / `vbind` |
| **列表能展开但里面空** | `forEach` 渲染列表 | 改 `vfor` |
| **动画完全不动** | 同上（订阅没建立），或 `repeatForever` 被杀 | 写死元素 + `setTimeout` |
| **动画播一下停住** | `repeatForever` 架构级冲突 | 改 `setTimeout` 手动往返 |
| **阴影变成实心色块** | `BoxShadow` 的 `fill` 默认 `true` | 显式 `fill = false` |
| **阴影完全看不到** | `offsetX/Y` 都是 0，`isEmpty()` 为 true | offsetY ≠ 0 |
| **描边比设计稿粗 3 倍** | `1f` 被四舍五入成 3 个物理像素 | 用 `0.5f` |
| **描边消失** | 值太小被归零（`lineWidth <= 0` 不画） | 别用 `0.25f` 以下 |
| **渐变画出来是纯色** | 端点退化成 `(0,0)→(0,0)` | 先打印四个端点值 |
| **编译报"接收者无法解析"** | `View {}` 的 lambda 不是 `ViewContainer` | 内联 / 顶层扩展函数 |
| **扩展函数访问不到 attr** | `attr` 是 `protected` | 用 `getViewAttr()` |
| **输入框每敲一个字触发一次** | 输入法组字期也会触发 `textDidChange` | 加 `setTimeout` 防抖 |
| **RichText 铺不满** | 不响应 `alignItemsStretch()` | 自己写 `alignSelfStretch()` |
| **构建在 dexing 崩溃** | AGP 版本与 Kotlin 版本不匹配 | AGP 8.5.2 + Gradle 8.11 |
| **BuildConfig 找不到** | AGP 8 默认不生成 | `buildFeatures { buildConfig = true }` |
| **装包后截图全黑** | force-stop 后首帧未就绪 | 等 5s 再截 |
| **SVG 加载不出来** | Kuikly 只认位图 | 出 PNG 或 Canvas 重画 |

## 三条最高频规则

```
1. Scroller 不写 padding，padding 交给内层 View
2. 创建期不能读快照：用 vfor / vbind，写死也行，就是不能 forEach / 提前求值
3. 描边用 0.5f，不用 1f
```

## 排查决策树

```
UI 不对
├── 布局问题？
│   ├── 左右不对称 → 查 Scroller padding
│   ├── 完全没显示  → 查 flex / 高度是否为 0
│   └── 大小不对    → 像素量测，不要靠肉眼
├── 数据不更新？
│   └── 先找「读取发生在哪里」，不是找「更新为什么没触发」
│       ├── forEach/map → vfor
│       ├── 提前求值传给普通函数 → vbind
│       └── 数量固定 → 写死
├── 动画不动？
│   ├── 手动点击翻转也不生效 → 订阅问题（见上）
│   ├── 播一下停 → repeatForever，改 setTimeout
│   └── 完全没有 → 查 animate 参数重载
└── 渲染异常？
    ├── 阴影成块 → fill = false
    ├── 阴影没有 → offsetY ≠ 0
    ├── 描边太粗 → 0.5f
    └── 渐变成纯色 → 端点退化
```

## 验证方法（不要靠肉眼）

| 问题类型 | 验证手段 |
| --- | --- |
| 布局边距 | 像素量测脚本，断言 `|左−右| ≤ 1px` |
| 颜色 | 取色器采样具体像素值 |
| 动画 | 连续多帧截图，比对是否变化 |
| 渐变 | 沿绘制路径采样，看是否有色差 |
| 描边宽度 | 2x/3x 放大截图，数物理像素 |

## 一条通用原则

> **凡是源码看起来正确、但实测不生效的 API，一律以实测为准并登记。**
>
> `repeatForever` 的源码设了 `ObjectAnimator.INFINITE`，端侧却不生效。
> 只看源码会让你以为是自己的调用方式错了。