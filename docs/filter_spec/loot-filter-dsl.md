# 掉落过滤器 DSL 语法规范 (Loot Filter DSL Syntax)

## 语法体系 (Grammar)

```
filter        := line*
line          := blank | comment | default_mode | rule | group_open | group_close
comment       := '#' any*
default_mode  := ('hide' | 'show') 'default'
rule          := [name] attr*
group_open    := '[' attr* ']' '{'
group_close   := '}'
name          := '"' regex '"'
attr          := quality
               | tier
               | socket
               | level
               | class
               | 'eth'
               | 'quest'
               | stat_pattern
               | color
               | visibility
               | sound
               | 'notify'
               | 'stat'
               | 'map'
socket        := 'sockets0' | 'sockets1' | 'sockets2' | 'sockets3'
               | 'sockets4' | 'sockets5' | 'sockets6'
level         := 'min_clvl' digit+ | 'max_clvl' digit+
               | 'min_ilvl' digit+ | 'max_ilvl' digit+
class         := 'amazon' | 'sorceress' | 'necromancer' | 'paladin'
               | 'barbarian' | 'druid' | 'assassin'
stat_pattern  := '{' regex '}'
```

`attr*` 允许重复，因此一条规则可以包含**多个** `{regex}` 属性匹配模式。所列出的所有模式必须全部与物品的属性文本匹配（即“且”/ AND 关系）。

分组不能嵌套。`default_mode` 指令仅在全局（文件顶层作用域）有效，且每个文件最多只能出现一次。

---

## 默认模式指令 (Default mode directive)

使用单个文件级作用域指令控制如何处理未被任何规则匹配到的物品。

```
hide default      # 默认隐藏所有物品，除非有规则显式使用 show 显示它们
show default      # 按游戏内置过滤器的默认机制显示物品（这是未配置时的默认行为）
```

- 仅允许在文件顶层作用域使用 —— 绝不能在 `[...] { ... }` 分组内使用。
- 每个文件最多出现**一次**。重复出现会导致解析错误。
- 缺省时 → 等同于 `show default`。
- 在文件中的位置不限（顶部、底部或中间）。通常规范：置于首个非注释行。

使用 `hide default` 时，只有带有显式 `show` 标记的规则才会显示物品。如果不加该指令，未被任何规则强制处理的物品将由游戏内置过滤机制决定。

---

## 规则组成要素 (Rule Components)

### 名称匹配模式（可选）

双引号内的正则表达式，不区分大小写匹配。如果正则命中了物品的**实时显示名称**（例如稀有词缀 `"Rune Turn"` 或暗金物品名称 `"Stone of Jordan"`），或者命中了 `items.txt` 中的**基础底模类型名称**（例如 `"Ring"`、`"Amulet"`、`"Great Axe"`），则该规则即判定为匹配。因此，`"Ring$"` 将匹配任何戒指，无论其品质或随机词缀为何。

```
"Ring$" unique gold
"Stone of Jordan" notify
"^(Ber|Jah|Sur|Lo|Ohm|Vex)$" orange
```

省略双引号表示匹配任意名称：

```
unique gold
set lime notify
```

`"."` 在效果上等同于省略名称匹配模式。

---

### 品质 (Quality)

| 关键字 | 品质说明 |
|---|---|
| `low` | 低劣/劣质 (Inferior) |
| `normal` | 普通 (Normal) |
| `superior` | 超强 (Superior) |
| `magic` | 魔法/蓝装 (Magic) |
| `set` | 套装/绿装 (Set) |
| `rare` | 稀有/黄装 (Rare) |
| `unique` | 暗金 (Unique) |
| `craft` | 手工/橙装 (Crafted) |
| `honor` | 尊贵装备 (Honorific) |

同一条规则中的多个品质关键字之间为**“或”（OR）逻辑** —— 只要物品的品质属于所列出的任意一种即可匹配。例如：`magic rare unique hide` 会同时隐藏魔法、稀有和暗金物品。重复项会被自动去重合并。

---

### 稀有度 / 暗金位阶 (Rarity / Unique tiers)

| 关键字 | 稀有度位阶 |
|---|---|
| `tu` | 阶级暗金 (Tiered Unique，wLvl 2–100，或 wLvl 0/1 的 1–4 阶底模) |
| `su` | 特异暗金 (Super Unique，wLvl 101–115) |
| `ssu` | 神圣特异暗金 (Sacred Super Unique，wLvl 116–120) |
| `sssu` | 神圣超特异暗金 (Sacred Super Super Unique，wLvl 121+) |

独立于 `quality` —— 只有暗金品质的物品才具有稀有度位阶，因此单写一个 `sssu` 规则就已隐式代表了暗金品质，无需额外写 `unique`。同品质一样，多个稀有度关键字之间为**“或”（OR）逻辑**。
非暗金物品（以及游戏报告中在非 1–4 阶底模上 `wLvl` 为 0/1 的暗金）绝不会命中稀有度位阶规则。

```
[sssu map] { . }         # 每一个 SSSU 掉落都在小地图上标记红叉
tu su hide               # 隐藏低阶/中阶暗金，保持 SSU/SSSU 可见
```

---

### 品级 / 阶级 (Tier - MedianXL)

| 关键字 | 品级 |
|---|---|
| `0`, `1`, `2`, `3`, `4` | 普通阶级 (Tier 0 ~ Tier 4) |
| `sacred` | 神圣 (Sacred) |
| `angelic` | 天使 (Angelic) |
| `master` | 大师级 (Mastercrafted) |

同一条规则中的多个阶级关键字之间为**“或”（OR）逻辑** —— 物品满足任意一个阶级即匹配。例如：`1 2 3 4 hide` 会隐藏 1–4 阶的所有物品。若结合品质关键字可求交集：`1 2 3 4 unique hide` 仅隐藏 1–4 阶的暗金物品。

---

### 孔数 (Sockets)

| 关键字 | 孔数 |
|---|---|
| `sockets0` | 物品无孔 / 0 孔 |
| `sockets1`–`sockets6` | 精确的孔数 (1 至 6 孔) |

同一条规则中的多个孔数关键字之间为**“或”（OR）逻辑** —— 物品只要满足任一孔数即可匹配。例如：
`sockets4 sockets5 sockets6 notify` 会在出现 4 孔、5 孔或 6 孔物品时发出通知。孔数直接读取自暗黑 2 的 `NumSockets` 属性（`0xC2`）；对于未设置 SOCKETED 标志的物品视作 `0` 孔。

当孔数 `N > 0` 时，通知组件还会自动在物品属性文本前追加一行 `Socketed (N)`，因此现有的属性正则模式如 `{Socketed \(6\)}` 仍可正常生效。

---

### 角色等级 / 物品等级 (Character / Item level)

| 关键字 | 效果说明 |
|---|---|
| `min_clvl<N>` | 仅当玩家角色等级 >= `N` 时匹配 |
| `max_clvl<N>` | 仅当玩家角色等级 <= `N` 时匹配 |
| `min_ilvl<N>` | 仅匹配物品等级 (ilvl) >= `N` 的物品 |
| `max_ilvl<N>` | 仅匹配物品等级 (ilvl) <= `N` 的物品 |

这四个关键字每条规则最多各出现一次（同一行中后出现的生效，同 `sound`/`color` 一致）。Min 和 Max 组合构成闭区间范围：`min_clvl20 max_clvl99` 匹配角色等级在 20 至 99 级之间。
角色等级在每次扫描轮次采样一次（非每件物品重复采样）；物品等级直接从掉落物品内存读取。

```
min_ilvl85 unique notify           # 仅对 ilvl >= 85 的暗金发出通知
min_clvl1 max_clvl30 "Ring" notify # 仅在练小号开荒阶段通知戒指掉落
```

---

### 角色职业 (Character class)

| 关键字 | 别名 | 职业说明 |
|---|---|---|
| `amazon` | `zon` | 亚马逊 |
| `sorceress` | `sorc` | 法师 |
| `necromancer` | `necro` | 死灵法师 |
| `paladin` | `pal`, `pally` | 圣骑士 |
| `barbarian` | `barb` | 野蛮人 |
| `druid` | `dru` | 德鲁伊 |
| `assassin` | `sin` | 刺客 |

同一条规则中的多个职业关键字之间为**“或”（OR）逻辑** —— 当玩家正在游玩所列出的任意职业时即生效。职业读取自玩家当前操控的角色，而非掉落物品。

```
necro barb notify {Faster Cast Rate}   # 仅在玩死灵或野蛮人时通知施法速度装备
```

---

### 无形 (Ethereal)

```
eth     # 仅匹配无形物品
```

---

### 任务物品 (Quest)

```
quest     # 仅匹配任务物品
```

匹配 `items.txt` 中的基底类型（或类别）为 Median XL 的 `Quest Item` 的物品 —— 这与名称模式检查的字段相同，因此 `"Quest Item"` 和 `quest` 完全等价。催化剂/炼金试剂（`Cube Reagent`）属于独立的基底类型，不会被 `quest` 匹配。

```
quest orange map notify   # 橙色高亮、小地图标记并在屏幕通知所有任务物品
```

---

### 属性匹配模式 (Stat pattern)

大括号包裹的正则表达式，不区分大小写匹配物品的属性词条文本。

```
{All Skills}
{\+[3-5] to All Skills}
{(Fire|Cold|Lightning) Resist}
```

一条规则可以包含**多个**属性匹配模式。物品的全部属性文本必须满足**所有**列出的模式才能触发该规则（“且”/ AND 关系）。每个模式独立进行匹配 —— 属性词条在物品上的排列顺序不影响匹配。

```
rare {All Skills} {Faster Cast Rate}                 # 两个属性必须同时满足
"Amulet" rare {[3-9] to All Skills} {focus} {enemy fire} stat notify
```

在单个 `{…}` 内部使用管道符 `|` 可表示“或”（OR）逻辑：

```
rare {(Fire|Cold|Lightning) Resist}                  # 满足三系抗性中任意一种即可
```

当启用 `stat`（或隐式属性模式触发）时，文本匹配规则中**任何**模式的每一行属性，都会在屏幕通知卡片中进行高亮显示。跨行模式（例如 `{(?s)a.*b}`）仍然可以触发规则匹配，但不会单独高亮某一行。

---

### 颜色 (Color)

支持以下颜色之一：

`white`（白）, `red`（红）, `lime`（浅绿）, `blue`（蓝）, `gold`（金）, `grey`（灰）, `black`（黑）, `pink`（粉）, `orange`（橙）, `yellow`（黄）, `green`（绿）, `purple`（紫）。

仅设置颜色本身不会生成屏幕通知。必须配合 `notify` 一同使用才会弹出通知。

---

### 可见性 (Visibility)

| 关键字 | 效果说明 |
|---|---|
| `show` | 强制在地面显示该物品（覆盖 `hide default` 以及游戏内置的隐藏） |
| `hide` | 强制在地面隐藏该物品 |

缺省时 → 适用默认可见性（由游戏原版机制或文件开头的 `hide default` 决定）。

---

### 提示音 (Sound)

| 关键字 | 效果说明 |
|---|---|
| `sound1`–`sound7` | 通知时播放的对应音效编号 |
| `sound_none` | 显式静音（不播放任何声音） |

仅设置音效本身不会触发通知。必须配合 `notify` 一同使用。

---

### 屏幕通知 (Notify)

```
notify    # 为该物品发出悬浮窗屏幕通知卡片
```

独立于颜色与提示音。任何物品若需在屏幕弹出卡片提醒，都必须包含此关键字。

---

### 显示标志 (Display flags)

| 关键字 | 效果说明 |
|---|---|
| `stat` | 在屏幕通知卡片中完整展示该物品的词条属性明细 |

套装/TU/SU/SSU/SSSU 等掉落的暗金/套装专属名称行会自动显示（受通知设置中的 **简洁名称 / Compact name** 选项控制，而非规则标志）。其他品质一律渲染为单行基底类型名称。

---

### 小地图标记 (Map marker)

```
map    # 在游戏内置小地图的物品掉落坐标放置一个红叉标记
```

独立于 `notify`。标记直接生成于游戏原生小地图上；在切换场景/区域时自动清理，并在物品掉落或被拾取时实时刷新。被解析为 `hide`（隐藏）的物品不会在地图上做标记。

---

## 规则分组 (Groups)

```
[公共属性] {
  规则1
  规则2
  ...
}
```

- 分组头部接受除**名称匹配模式以外**的所有规则属性。
- 分组体内的每条子规则都会自动继承头部的公共属性。
- 子规则级别的属性会**覆盖**组级别中针对同一字段的设置。
- 分组**不能嵌套**。
- 分组内的规则按照其在文件中的实际顺序展开评估（分组会被扁平化展开，原有顺序完全保留）。

### 示例 1：共享高亮与提醒配置

```
[unique gold notify sound1] {
  "Jordan"
  "Tyrael"
  "Windforce"
}
```

展开等效于：

```
"Jordan" unique gold notify sound1
"Tyrael" unique gold notify sound1
"Windforce" unique gold notify sound1
```

### 示例 2：共享属性过滤器

```
[unique {All Skills} red notify stat] {
  "Ring$"
  "Amulet"
  "Circlet"
}
```

### 示例 3：分组内局部覆盖公共属性

```
[hide] {
  normal
  low
  superior
  unique show gold notify    # show 覆盖了外层分组设置的 hide
}
```

---

## 注释 (Comments)

```
# 单独占一行的整行注释
unique gold notify    # 行末注释
```

---

## 规则评估机制 (Evaluation)

对于地面掉落的每一件物品，所有规则（包括从分组中展开的规则）都会**按文件源码书写顺序自上而下**进行评估。**最后一条匹配命中的规则**将决定该物品的最终处理结果（后匹配优先原则）。详见 `loot-filter-spec.md` 了解完整语义规范。

---

## 快速参考速查表 (Quick Reference)

```
# 全局文件级指令（最多一条，可选）
hide default      # 默认隐藏所有未匹配物品
show default      # 默认显示所有未匹配物品（隐式默认值）

# 完整通用规则形式
[名称模式] [品质] [稀有度] [品级] [孔数] [等级] [职业] [eth] [{属性模式}]* [颜色] [show|hide] [提示音] [notify] [stat] [map]

# 语法原子
品质 quality    := low | normal | superior | magic | set | rare | unique | craft | honor
稀有度 rarity   := tu | su | ssu | sssu
品级 tier       := 0 | 1 | 2 | 3 | 4 | sacred | angelic | master
孔数 socket     := sockets0 | sockets1 | sockets2 | sockets3 | sockets4 | sockets5 | sockets6
等级 level      := min_clvl<N> | max_clvl<N> | min_ilvl<N> | max_ilvl<N>
职业 class      := amazon | sorceress | necromancer | paladin | barbarian | druid | assassin
颜色 color      := white | red | lime | blue | gold | grey | black
                 | pink | orange | yellow | green | purple
可见性 visibility := show | hide
提示音 sound    := sound1 | sound2 | sound3 | sound4 | sound5 | sound6 | sound7 | sound_none
```
