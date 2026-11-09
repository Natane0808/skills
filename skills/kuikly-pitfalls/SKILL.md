---
name: kuikly-pitfalls
description: KuiklyUI 实战踩坑与排查手册。收录在真实项目中（Android 真机 + 源码级验证）踩过并以截图/逐像素采样取证的坑：Scroller padding 不对称、创建期读快照导致响应式静默失效、repeatForever 循环动画必被杀、BoxShadow 画成实心块、描边被取整成整数物理像素、Canvas 渐变端点退化静默变纯色、forEach 渲染空列表、View{} lamdba 接收者陷阱、textDidChange 输入法组字期触发、RichText 不响应 stretch、AGP 版本组合导致 D8 dexing 崩溃等。当遇到"代码看起来没问题但 UI 不更新""动画不动""布局左右不对称""阴影变成色块""描边太粗/不见""列表能展开但空""编译报冒号/接收者错误"这类 Kuikly 诡异现象时使用。与官方 API 文档类技能（kuikly-ui-framework 等）互补：那些讲"怎么用"，本技能讲"哪里会翻车、怎么快速定位"。
---

# Kuikly 实战踩坑手册

> 本技能只收录**在真实项目里踩过、并用真机截图或源码取证**的坑。
> 每条都给出：**现象 → 根因（带源码路径）→ 正确写法 → 排查方法**。
> 没有实测证据的条目不会写在这里（宁可缺失，不要误导）。

## 使用方式

遇到 Kuikly 的"诡异现象"时，**先按症状查下表**，再读对应小节：

| 症状 | 优先怀疑 | 小节 |
| --- | --- | --- |
| 布局左右不对称（左窄右宽） | `Scroller` 上写了 padding | §1 |
| 数据变了但界面不变 | 创建期读了快照（断订阅） | §2 |
| 列表/面板能展开但里面是空的 | `forEach` 渲染列表 | §2.1 |
| 动画完全不动 / 播一下停住 | `repeatForever` 被杀 | §3 |
| 卡片底下出现一块实心色块 | `BoxShadow` 缺少 `fill=false` | §4 |
| 描边比设计稿粗 3 倍 | 写了 `1f`，被取整成整数物理像素 | §5 |
| 渐变画出来是纯色 | 渐变端点退化成 `(0,0)→(0,0)` | §6 |
| 编译报"接收者无法解析" | `View {}` 的 lambda 不是 `ViewContainer` | §7 |
| 阴影完全看不到 | offsetX/Y 都是 0 | §4 |
| 输入框每敲一个字就触发一次 | 输入法组字期也会触发 | §8 |
| 富文本铺不满宽度 | `RichText` 不响应 `alignItemsStretch` | §9 |
| 构建在 dexing 阶段崩溃 | AGP 版本与 Kotlin 版本不匹配 | §10 |
| 装包后截图是黑的 | 应用被 force-stop 后首帧未就绪 | §12 |

---

## §1 `Scroller` 上绝对不能写 padding —— 会导致左右边距不对称

### 现象
卡片左右边距不等，左边正常、右边多出约 36dp。**肉眼能看出"有点偏"但说不出哪里错。**

实测数据（1080×2280 屏幕，density 2.75）：
```
卡片 x=[51..930]，左边距 51px，右边距 149px，差 98px
```
而且是**整屏所有卡片统一偏移**，说明是布局骨架的问题，不是单页写错。

### 根因（源码）
`ScrollerAttr.padding()` 把 padding **转发**给内部 contentView，而不是占位：

```kotlin
// core/views/ScrollerView.kt:416-423
override fun padding(top, left, bottom, right): ContainerAttr {
    val contentView = (view() as? ScrollerView)?.contentView
    if (contentView != null) {
        contentView.getViewAttr().padding(...)   // ← 转发，不是占位
        return this
    }
    return super.padding(...)
}
```

而 contentView 是按 `left/right` 钉死的：

```kotlin
// ScrollerView.kt:282-286（垂直滚动）
super.addChild(contentView!!, {
    attr { absolutePosition(top = 0f, left = 0f, right = 0f) }  // 宽度 = 父宽 − paddingLeft
})
// :290
flexNode.setPadding(StyleSpace.Type.ALL, 0f)  // 清掉 Scroller 自己的 padding
```

**结果**：左边距只算一次，右边距被算两次 → 左右不对称。

### 正确写法
`Scroller` 自己**不写 padding**，padding 全部交给内层容器：

```kotlin
Scroller {
    attr { flex(1f) }                    // ⚠️ 绝不写 padding
    View {
        attr {
            paddingLeft(Tokens.PadPage)
            paddingRight(Tokens.PadPage)
            paddingBottom(Tokens.DockSafe + safeAreaInsets.bottom)
            flexDirectionColumn()
        }
        // ... 所有内容
    }
}
```

建议封装成统一的 `PageScroller` 组件，**从结构上防止再犯**。

### 排查方法
**不要靠肉眼。** 写一个像素量测脚本：沿水平扫描线找卡片左右边界，断言 `|左边距 − 右边距| ≤ 1px`（1px 是奇数宽度的取整容差）。

---

## §2 🔴 「创建期读快照 = 断订阅」—— Kuikly 最高频、最隐蔽的一类 bug

### 原理
Kuikly 的响应式是**在 `attr {}` 执行期读取 observable 时建立订阅**。如果某个值在**创建期**被读走（快照），之后 observable 变化时那块 UI **根本不会重跑** —— 而且**不报错、不警告、不崩溃**。

### 三种表现形式（本质完全相同）

#### 2.1 `forEach` 渲染列表 → 面板能展开但里面空

```kotlin
// ❌ 错：列表是首帧快照
ctx.servers.forEach { s ->
    View { attr { /* ... */ } Text { attr { text(s.name) } } }
}

// ✅ 对：用 vfor
vfor({ ctx.servers }) { s ->
    View { attr { /* ... */ } Text { attr { text(s.name) } } }
}
```

**典型场景**：数据是异步补的，面板能打开但列表永远是空的。

#### 2.2 `forEachIndexed` 渲染动画/交互元素 → 看起来像"动画坏了"

```kotlin
// ❌ 错：这些 View 根本没登记 observable 订阅
dots.items.forEachIndexed { i, item ->
    View { attr { opacity(...); animate(..., item.on) } }
}
```

**现象特别容易误判**：动画"不播放"，你会去查 `animate` 的参数、顺序、Timer —— 但**真正的问题是订阅从未建立**。
**判别方法**：加一个"手动点击翻转 observable"的对照实验。如果**手动改也不生效**，就排除了 timer/animate，直接怀疑订阅。

#### 2.3 把值算成 String 传给普通函数 → **最隐蔽，代码看起来毫无问题**

```kotlin
// ❌ 错：gap 在这里被求值成 String，子函数是普通函数，不会重跑
val gap = ctx.gearGap()                      // 读 observable
ResultRow("还差", Format.thousands(gap), …)   // 传进去就死了

// ✅ 对：用 vbind 包住结果区
vbind({ ctx.gearGap() }) { gap ->
    ResultRow("还差", Format.thousands(gap), …)
}
```

**为什么最隐蔽**：没有 `forEach`、没有显式"提前取值"的痕迹，只是一次普通的函数参数传递。

### 排查口诀
> **数据变了 UI 没变 → 先找"读取发生在哪里"，而不是找"更新为什么没触发"。**

### 规则（可直接抄进项目规范）
> ❌ 不要在 `body()` 里用 `forEach` / `map` / `forEachIndexed` / `sumOf` 迭代**任何会在之后变化的集合**
> ❌ 不要把读自 observable 的值**算成普通值**传给普通函数
> ✅ 一律用 `vfor` / `vbind`，或**数量固定且永不变化时写死**

**注意**：这条规则要按**原理**记忆（"创建期不能读快照"），不要按场景记忆（"列表要用 vfor"）—— 否则换个场景（动画、计算结果）就不会触发了。

---

## §3 `repeatForever` 循环动画**在 Android 上必被杀**

### 现象
`animate(Animation.easeInOut(0.6f).repeatForever(true), observable)` —— 动画**播一段就停**，不循环。连续 8 帧截图，区域均值恒为 205，零变化。

### 根因（源码，架构级冲突）
```kotlin
// KRCSSAnimation.kt:160-169
value.start(targetView) {
    animationRunningCount--          // ← 只有 onAnimationEnd 才减
}
animationRunningCount++              // ← 启动时 +1

// KRCSSViewExtension.kt:690-700 —— 每轮 attr 结束写 ANIMATION=""
if (v.isPlaying()) { v.cancelAnimation() }
```

`repeatForever` 下动画永不结束 → `onAnimationEnd` 永不触发 → **计数恒 ≥1 → `isPlaying()` 恒 true** → 宿主每轮 `attr{}` 结束都会把它取消。

**这是架构级冲突，换参数、换顺序、换属性都救不回来。**

### 正确写法：`setTimeout` 手动往返
```kotlin
// 用 setTimeout 定时翻转 observable，而不是依赖 repeatForever
setTimeout(600) {
    dot.on = !dot.on
    // 递归安排下一次（注意在 viewWillUnload 里清理）
}

override fun viewWillUnload() {
    super.viewWillUnload()
    // 清 timer，否则页面销毁后回调仍会触发
}
```

### 注意 `setTimeout` 的签名
```kotlin
// ❌ 错：这是 0ms（把 lambda 当成 delay 参数之前的位置）
setTimeout { ... }

// ✅ 对：PagerScope 扩展，先传 timeout
setTimeout(200) { ... }
```

### 计时器启动时机的坑
- `addTaskWhenPagerUpdateLayoutFinish` **只在布局脏循环里执行**（`Pager.kt:443-450` 的 `while(flexNode.isDirty)`），视图稳定时不触发；
- 启动时机用 **`viewDidLoad()` 最简单**。

---

## §4 `BoxShadow` 的两个致命默认值

### 4.1 `fill` 默认 `true` → 画出来是**实心色块**，不是投影
```kotlin
// ❌ 错：卡片底下出现一块实心色块
boxShadow(BoxShadow(0f, 2f, 8f, color))

// ✅ 对：必须显式 fill = false
boxShadow(BoxShadow(0f, 2f, 8f, color, false))
```
源码：`KRViewDecoration.kt:259-286`，只有 `!shadowFill` 才走 `clipPath(DIFFERENCE)`。

⚠️ **常量层埋雷**：如果 `Tokens.ShadowXxx` 这类常量没带 `fill=false`，**任何引用它的组件都会画成实心块**。用的时候看不出来，界面"好像有点脏"。

### 4.2 `offsetX/Y` 全为 0 → **阴影根本不渲染**
源码：`KRViewDecoration.kt:686-688`，`offsetX/Y` 全为 0 时 `isEmpty() = true`，跳过绘制。
**四边均匀外发光做不出来**，必须有偏移。

### 4.3 单层限制
源码用硬编码下标取**一组**值（`:657-684`），没有循环 → **结构上不支持多层阴影**。多层必须拆成嵌套 View。

### 4.4 性能
实现是 `BlurMaskFilter`（**CPU 软件绘制**），大半径 + 大面积会掉帧。大投影建议直接烘焙进底图。

---

## §5 描边宽度会被**四舍五入成整数物理像素**

### 结论速查（440dpi / density 2.75 实测）
| 写入 | 实测物理像素 | 说明 |
| --- | --- | --- |
| `0.25f` | 1 | 与 0.5f 视觉相同，**零价值**；低密度机会归零消失 |
| **`0.5f`** | **1** | ✅ **推荐**，视觉 ≈ 1px |
| `0.75f` | 2 | |
| `1.0f` | **3** | ❌ **比设计稿粗 3 倍** |

### 根因（源码）
```kotlin
// KuiklyRenderExtension.kt:62
fun Float.toPxI() = (toPxF() + 0.5f).toInt()   // 四舍五入

// KRCSSBackgroundDrawable.kt:108
lineWidth = kuiklyContext.toPxI(borderStyles[...].toFloat())

// KRCSSBackgroundDrawable.kt:120
if (lineWidth <= 0 || lineColor == Color.TRANSPARENT) {
    setStroke(0, TRANSPARENT); return            // ← 归零就彻底不画
}
```

### 规则
> **CSS 的 `1px` 描边 → Kuikly 写 `0.5f`。**
> 线太淡时**加深颜色**，不要加粗宽度。
> 建一个 `Tokens.Hairline = 0.5f` 常量，全项目统一引用。

---

## §6 Canvas 渐变**可用**，但端点退化会**静默变纯色**

### 结论
`strokeStyle(LinearGradientObject)` 在 Android **完全可用**（实测 E1/E3/E4 全通，逐像素采样沿弧有色差）。

### 陷阱
端点退化（如 `(0,0) → (0,0)`）时，**不报错、不警告，静默变成纯色** —— 极容易被误判成"API 不支持渐变"。

```kotlin
// ❌ 风险：如果 startX/startY 都算成 0，会静默变纯色
context.createLinearGradient(startX, startY, endX, endY)

// ✅ 排查：先打印四个端点值，确认不是退化值
```

### 另一条：`createLinearGradient` 空实现是死代码
`KRCanvasView.kt:741-744` 有问题，但**不影响使用**。

---

## §7 `View {}` / `Scroller {}` 的 lambda **不是** `ViewContainer` 接收者

### 现象
```kotlin
View {
    attr { ... }
    this.myHelper()   // ❌ 报 cannot be called in this context with an implicit receiver
}
```

### 根因
嵌套的 `View {}` lambda 接收者类型与 `ViewContainer<*, *>` 不同，**扩展函数无法隐式解析**。

### 三种正解
```kotlin
// 1. 把 helper 写成内联（最简单，最保守）
View { attr { ... }; Text { attr { ... } } }

// 2. 在 View 外部调用，用显式接收者
val ctx = this
View { attr { ... } }
ctx.myHelper()

// 3. 顶层扩展函数，参数全部显式传入（推荐用于可复用组件）
fun ViewContainer<*, *>.MyCard(title: String) { /* 内联展开 */ }
```

### 相关：`protected val attr` 在扩展函数里访问不到
```kotlin
// ❌ 错：attr 是 protected
fun GlassCard.helper() { attr.level = ... }

// ✅ 对：用公开的 getViewAttr()
fun GlassCard.helper() { getViewAttr().level = ... }
```

---

## §8 `textDidChange` 在**输入法组字期**也会触发

### 触发矩阵（源码 `KRTextFieldView.kt:1062-1077`）
| 场景 | 是否触发 | 说明 |
| --- | --- | --- |
| 逐字输入 | ✅ 每次 | 正常 |
| **输入法组字中（拼音未上屏）** | ✅ **每敲一个字母** | 拿到的是**半成品文本** |
| 程序 `view.setText()` | ✅ **会触发** | 无额外守卫 |
| `setTextInputState()` | ❌ | 被标志位拦截 |

### 影响
中文输入「唯满侠」会依次回调 `w` → `we` → `wei` → … **每次触发一次网络请求 = 十几次无效请求**。

### 正确写法
```kotlin
// 输入用防抖
setTimeout(200) { /* 真正的查询 */ }   // 注意：setTimeout(timeout) { } 签名
```

**校验不要用 `textDidChange`** —— 用 `inputBlur`，否则用户打字到一半就报错。

---

## §9 `RichText` 不响应 `alignItemsStretch()`

要铺满宽度必须自己写 `alignSelfStretch()`。

---

## §10 构建环境：AGP 版本必须与 Kotlin 版本匹配

### 现象
```
:androidApp:mergeExtDexDebug FAILED
ERROR:D8: com.android.tools.r8.kotlin.H
R8_4.0.52 ... at com.android.tools.r8.kotlin.j.a
```

### 根因
R8 4.0.52（AGP 7.4.2 自带）**无法解析 Kotlin 2.1.21 的元数据**。

### 正确组合（实测可构建）
| 项 | 版本 |
| --- | --- |
| Kuikly | 2.30.0 |
| Kotlin | 2.1.21 |
| **AGP** | **8.5.2** |
| **Gradle** | **8.11** |
| KSP | 2.1.21-2.0.1 |
| JDK | 17 |

### 两个坑
1. **AGP 8 默认不生成 `BuildConfig`** → 需在 `androidApp/build.gradle.kts` 加：
   ```kotlin
   buildFeatures { buildConfig = true }
   ```
   否则引用 `BuildConfig.DEBUG` 会 `Unresolved reference`。
2. **不要加 `android.disableAutomaticComponentCreation=true`** —— AGP 8 已移除该选项，会直接报 `EvalIssueException` 构建失败。（官方 demo 里有这行是因为它跑 AGP 7.x，照抄就踩坑。）

### 无害告警
```
WARNING: D8: An error occurred when parsing kotlin metadata
```
这是 AGP 8.5.2 自带的 R8 8.5.10 略低于 Google 对 Kotlin 2.1 的要求（R8 8.6.17），**仅告警、不影响 dex**，忽略即可。

---

## §11 资源：Kuikly **不支持 SVG**

`core-render-android` 的 `KRImageAdapter` 走 Glide + `BitmapFactory`，**只认位图**。

### 两个方案
1. **让设计师出 PNG**（@2x/@3x，有切图问题）；
2. **用 Canvas 重画**（推荐用于单色线条图标）：
   ```kotlin
   // 按 24×24 viewBox 写坐标，绘制时按 size/24 缩放
   Canvas({ attr { size(20f, 20f) } }) { ctx, _, _ ->
       val s = 20f / 24f   // 缩放系数
       ctx.beginPath()
       ctx.moveTo(x * s, y * s)
       // ...
       ctx.strokeStyle(color)
       ctx.stroke()
   }
   ```
   优点：任意分辨率清晰、零资源体积、无切图问题。

---

## §12 真机截图与验证方法

### 装包后立刻截图可能全黑
应用被 `force-stop` 后首帧未就绪。
```powershell
adb install -r app.apk
adb shell am force-stop <package>
Start-Sleep -Seconds 5        # ← 必须等
adb exec-out screencap -p > shot.png
```

### `--es pageName` 对已运行实例无效
需要先 `force-stop`。

### 用构建参数切换起始页（推荐）
```kotlin
// build.gradle.kts
buildConfigField("String", "DXX_START_PAGE", "\"${project.findProperty("dxxStartPage") ?: \"main\"}\"")
```
```powershell
.\gradlew.bat :androidApp:assembleDebug -PdxxStartPage=verify
```

### 布局问题**必须像素量测**
肉眼只能看出"有点怪"，量测能一秒钟给出 98px 的结论。

---

## §13 本地存储的容量限制

Kuikly 的 `SharedPreferencesModule` 在**保存超长字符串时有容量限制**。
存大对象（如长列表 JSON）前先确认长度，超过阈值应拆分或改用其他方案。

---

## 附：一条通用原则

> **凡是「源码看起来正确、但实测不生效」的 API，一律以实测为准，并登记到项目文档。**
>
> `repeatForever` 就是典型：源码设了 `ObjectAnimator.INFINITE`，端侧却不生效。
> 如果只看源码，你会以为是自己调用方式错了，然后在错误方向上浪费大量时间。

---

## 参考

本技能内容来自真实项目「稻香匣」（Kuikly 2.30.0 / 微信未公开）的实战记录，所有结论均有：
- 真机截图（Pixel_4 / API 29 / 1080×2280 / density 2.75），或
- 源码路径（`core` / `core-render-android`），或
- 逐像素采样数据

官方 API 用法请参阅 `kuikly-ui-framework` 等技能；本技能只回答"哪里会翻车"。