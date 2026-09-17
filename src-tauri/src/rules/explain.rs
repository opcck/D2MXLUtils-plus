//! 针对掉落过滤规则 DSL 单行语句的中文自然语言解释器。

use super::dsl::{classify_line, ParsedLine};
use super::{ItemQuality, ItemTier, NotifyColor, Rule, UniqueKind, Visibility};

pub fn explain_line(line: &str) -> Option<String> {
    match classify_line(line) {
        ParsedLine::Empty | ParsedLine::GroupClose | ParsedLine::Unparseable => None,
        ParsedLine::Directive(hide) => Some(format_directive(hide)),
        ParsedLine::GroupHeader(rule) => Some(format_group_header(&rule)),
        ParsedLine::Rule(rule) => Some(format_rule(&rule)),
    }
}

// =====================================================================
// Top-level formatters
// =====================================================================

fn format_directive(hide: bool) -> String {
    if hide {
        "文件指令：隐藏所有未被任何规则匹配的物品。包含 'show' 的规则可针对特定物品覆盖此设置。"
            .to_string()
    } else {
        "文件指令：未被规则匹配的物品，交由游戏内置的过滤器决定。".to_string()
    }
}

fn format_rule(rule: &Rule) -> String {
    let predicate_bullets = predicate_lines(rule);
    let action_bullets = action_lines(rule);

    let mut out = String::new();

    if predicate_bullets.is_empty() {
        out.push_str("匹配所有物品。");
    } else if predicate_bullets.len() == 1 {
        out.push_str("匹配条件：\n");
        out.push_str("  • ");
        out.push_str(&predicate_bullets[0]);
        if let Some(note) = unrestricted_categories(rule) {
            out.push_str("\n\n");
            out.push_str(&note);
        }
    } else {
        out.push_str("当满足以下全部条件时匹配：");
        for bullet in &predicate_bullets {
            out.push_str("\n  • ");
            out.push_str(bullet);
        }
    }

    if !action_bullets.is_empty() {
        out.push_str("\n\n触发动作：");
        for bullet in &action_bullets {
            out.push_str("\n  • ");
            out.push_str(bullet);
        }
    }

    out
}

fn format_group_header(rule: &Rule) -> String {
    let mut bullets = predicate_lines(rule);
    bullets.extend(action_lines(rule));

    let mut out =
        String::from("规则分组头部 — 以下默认属性将应用于花括号内的所有规则（除非单条规则覆盖）：");
    if bullets.is_empty() {
        out.push_str("\n  (未设置默认属性)");
    } else {
        for bullet in &bullets {
            out.push_str("\n  • ");
            out.push_str(bullet);
        }
    }
    out
}

// =====================================================================
// Predicate lines
// =====================================================================

fn predicate_lines(rule: &Rule) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(ref pat) = rule.name_pattern {
        out.push(format!("物品名称匹配模式 \"{}\"", pat));
    }
    if !rule.tiers.is_empty() {
        out.push(tier_bullet(&rule.tiers));
    }
    if !rule.qualities.is_empty() {
        out.push(quality_bullet(&rule.qualities));
    }
    if !rule.unique_kinds.is_empty() {
        out.push(unique_kind_bullet(&rule.unique_kinds));
    }
    if rule.ethereal {
        out.push("物品为无形 (Ethereal)".to_string());
    }
    if rule.quest {
        out.push("物品为任务物品".to_string());
    }
    if !rule.stat_patterns.is_empty() {
        out.push(stat_bullet(&rule.stat_patterns));
    }
    out
}

fn tier_bullet(tiers: &[ItemTier]) -> String {
    if tiers.len() == 1 {
        format!("阶级为 {}", tier_label(tiers[0]))
    } else {
        let labels: Vec<&str> = tiers.iter().map(|t| tier_label(*t)).collect();
        format!("阶级为以下之一：{}", labels.join(", "))
    }
}

fn quality_bullet(qualities: &[ItemQuality]) -> String {
    if qualities.len() == 1 {
        format!("品质为 {}", quality_label(qualities[0]))
    } else {
        let labels: Vec<&str> = qualities.iter().map(|q| quality_label(*q)).collect();
        format!("品质为以下之一：{}", labels.join(", "))
    }
}

fn unique_kind_bullet(kinds: &[UniqueKind]) -> String {
    if kinds.len() == 1 {
        format!("稀有度为 {}", kinds[0].label())
    } else {
        let labels: Vec<&str> = kinds.iter().map(|k| k.label()).collect();
        format!("稀有度为以下之一：{}", labels.join(", "))
    }
}

fn stat_bullet(patterns: &[String]) -> String {
    if patterns.len() == 1 {
        format!("包含词条模式：\"{}\"", patterns[0])
    } else {
        let quoted: Vec<String> = patterns.iter().map(|p| format!("\"{}\"", p)).collect();
        format!("包含全部词条模式：{}", quoted.join(", "))
    }
}

fn unrestricted_categories(rule: &Rule) -> Option<String> {
    let mut missing: Vec<&str> = Vec::new();
    if rule.name_pattern.is_none() {
        missing.push("名称");
    }
    if rule.tiers.is_empty() {
        missing.push("阶级");
    }
    if rule.qualities.is_empty() {
        missing.push("品质");
    }
    if rule.unique_kinds.is_empty() {
        missing.push("稀有度");
    }
    if !rule.ethereal {
        missing.push("无形");
    }
    if !rule.quest {
        missing.push("任务物品");
    }
    if rule.stat_patterns.is_empty() {
        missing.push("词条");
    }
    let list = match missing.len() {
        0 => return None,
        1 => missing[0].to_string(),
        2 => format!("{} 与 {}", missing[0], missing[1]),
        _ => format!(
            "{} 以及 {}",
            missing[..missing.len() - 1].join("、"),
            missing.last().unwrap()
        ),
    };
    Some(format!("(其他类别 — {} — 不受限制。)", list))
}

// =====================================================================
// Effect lines
// =====================================================================

fn visibility_line(v: Visibility) -> Option<String> {
    match v {
        Visibility::Default => None,
        Visibility::Hide => Some("在地面隐藏此物品".to_string()),
        Visibility::Show => Some("强制显示此物品（覆盖游戏内置隐藏）".to_string()),
    }
}

fn action_lines(rule: &Rule) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(line) = visibility_line(rule.visibility) {
        out.push(line);
    }
    if rule.notify {
        out.push(notification_bullet(rule));
    } else if rule.color.is_some() || rule.sound.is_some() {
        out.push("已设置颜色/音效标记，但未配置 'notify'，因此不会触发提醒。".to_string());
    }
    if rule.map {
        out.push("在小地图对应位置标记物品".to_string());
    }
    if rule.display_stats && !rule.notify {
        out.push("已设置 'stat' 标记但不会显示 — 需要配合 'notify' 触发。".to_string());
    }
    out
}

fn notification_bullet(rule: &Rule) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(c) = rule.color {
        parts.push(format!("颜色: {}", color_label(c)));
    }
    match rule.sound {
        Some(0) => parts.push("静音".to_string()),
        Some(n) => parts.push(format!("音效 {}", n)),
        None => {}
    }
    if rule.display_stats || !rule.stat_patterns.is_empty() {
        parts.push("包含物品词条".to_string());
    }
    if parts.is_empty() {
        "显示悬浮窗掉落通知".to_string()
    } else {
        format!("显示悬浮窗掉落通知 ({})", parts.join(", "))
    }
}

// =====================================================================
// Token labels
// =====================================================================

fn quality_label(q: ItemQuality) -> &'static str {
    match q {
        ItemQuality::Inferior => "劣质 (low)",
        ItemQuality::Normal => "普通 (normal)",
        ItemQuality::Superior => "超强 (superior)",
        ItemQuality::Magic => "魔法 (magic)",
        ItemQuality::Set => "套装 (set)",
        ItemQuality::Rare => "稀有 (rare)",
        ItemQuality::Unique => "暗金 (unique)",
        ItemQuality::Crafted => "手工 (crafted)",
        ItemQuality::Honorific => "光荣 (honorific)",
    }
}

fn tier_label(t: ItemTier) -> &'static str {
    match t {
        ItemTier::Tier0 => "0",
        ItemTier::Tier1 => "1阶",
        ItemTier::Tier2 => "2阶",
        ItemTier::Tier3 => "3阶",
        ItemTier::Tier4 => "4阶",
        ItemTier::Sacred => "神圣 (sacred)",
        ItemTier::Angelic => "天使 (angelic)",
        ItemTier::Master => "匠品 (mastercrafted)",
    }
}

fn color_label(c: NotifyColor) -> &'static str {
    match c {
        NotifyColor::White => "白色",
        NotifyColor::Red => "红色",
        NotifyColor::Lime => "浅绿",
        NotifyColor::Blue => "蓝色",
        NotifyColor::Gold => "暗金",
        NotifyColor::Grey => "灰色",
        NotifyColor::Black => "黑色",
        NotifyColor::Pink => "粉色",
        NotifyColor::Orange => "橙色",
        NotifyColor::Yellow => "黄色",
        NotifyColor::Green => "绿色",
        NotifyColor::Purple => "紫色",
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
#[path = "explain_tests.rs"]
mod tests;
