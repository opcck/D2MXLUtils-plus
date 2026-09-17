use super::*;

#[test]
fn lines_with_no_explanation_return_none() {
    for src in ["", "   ", "# a comment", "   # spaced", "}"] {
        assert!(explain_line(src).is_none(), "expected None for {:?}", src);
    }
}

#[test]
fn directives_explained() {
    let hide = explain_line("hide default").unwrap();
    assert!(hide.contains("隐藏所有未被任何规则匹配的物品"));
    assert!(hide.contains("'show'"));

    let show = explain_line("show default").unwrap();
    assert!(show.contains("游戏内置的过滤器"));
}

#[test]
fn single_predicate_includes_unrestricted_note() {
    let s = explain_line("1 2 3 4 hide").unwrap();
    assert!(s.contains("阶级为以下之一：1阶, 2阶, 3阶, 4阶"));
    assert!(s.contains("不受限制"));
    assert!(s.contains("触发动作："));
    assert!(s.contains("在地面隐藏此物品"));
}

#[test]
fn two_predicates_omit_unrestricted_note() {
    let s = explain_line("sacred superior magic rare hide").unwrap();
    assert!(s.contains("全部条件"));
    assert!(s.contains("阶级为 神圣 (sacred)"));
    assert!(s.contains("品质为以下之一：超强 (superior), 魔法 (magic), 稀有 (rare)"));
    assert!(!s.contains("不受限制"));
    assert!(s.contains("触发动作："));
    assert!(s.contains("在地面隐藏此物品"));
}

#[test]
fn name_pattern_in_quotes() {
    let s = explain_line("\"Ring$\" unique gold notify").unwrap();
    assert!(s.contains("物品名称匹配模式 \"Ring$\""));
    assert!(s.contains("品质为 暗金 (unique)"));
    assert!(s.contains("显示悬浮窗掉落通知"));
    assert!(s.contains("颜色: 暗金"));
}

#[test]
fn no_predicate_says_matches_every_item() {
    let s = explain_line("gold notify").unwrap();
    assert!(s.starts_with("匹配所有物品。"));
    assert!(s.contains("显示悬浮窗掉落通知"));
}

#[test]
fn show_visibility_describes_override() {
    let s = explain_line("unique show").unwrap();
    assert!(s.contains("触发动作："));
    assert!(s.contains("强制显示此物品"));
    assert!(s.contains("覆盖游戏内置隐藏"));
    assert!(!s.contains("'hide default'"));
}

#[test]
fn visibility_and_notification_under_one_actions_section() {
    let s = explain_line("unique gold notify map").unwrap();
    assert_eq!(s.matches("触发动作：").count(), 1);
    assert!(!s.contains("Effects:"));
    assert!(s.contains("显示悬浮窗掉落通知"));
    assert!(s.contains("在小地图对应位置标记物品"));
}

#[test]
fn group_header_lists_defaults() {
    let s = explain_line("[unique gold notify] {").unwrap();
    assert!(s.starts_with("规则分组头部"));
    assert!(s.contains("品质为 暗金 (unique)"));
    assert!(s.contains("显示悬浮窗掉落通知"));
}

#[test]
fn group_header_with_no_defaults() {
    let s = explain_line("[] {").unwrap();
    assert!(s.contains("(未设置默认属性)"));
}

#[test]
fn rarity_predicate_rendered() {
    let s = explain_line("sssu map").unwrap();
    assert!(s.contains("稀有度为 SSSU"));
    assert!(s.contains("在小地图对应位置标记物品"));
}

#[test]
fn multi_rarity_predicate_rendered() {
    let s = explain_line("tu su hide").unwrap();
    assert!(s.contains("稀有度为以下之一：TU, SU"));
}

#[test]
fn ethereal_predicate_rendered() {
    let s = explain_line("eth unique").unwrap();
    assert!(s.contains("物品为无形 (Ethereal)"));
    assert!(s.contains("品质为 暗金 (unique)"));
}

#[test]
fn single_stat_pattern_rendered() {
    let s = explain_line("rare {All Skills} notify").unwrap();
    assert!(s.contains("包含词条模式：\"All Skills\""));
    assert!(s.contains("包含物品词条"));
}

#[test]
fn multi_stat_patterns_use_list_phrase() {
    let s = explain_line("rare {All Skills} {Faster Cast} notify").unwrap();
    assert!(s.contains("包含全部词条模式：\"All Skills\", \"Faster Cast\""));
}

#[test]
fn map_effect_listed() {
    let s = explain_line("unique map").unwrap();
    assert!(s.contains("在小地图对应位置标记物品"));
}

#[test]
fn color_without_notify_warns_in_tooltip() {
    let s = explain_line("unique gold").unwrap();
    assert!(s.contains("已设置颜色/音效标记，但未配置 'notify'，因此不会触发提醒。"));
}

#[test]
fn sound_modes_rendered() {
    assert!(explain_line("unique notify sound_none")
        .unwrap()
        .contains("静音"));
    assert!(explain_line("unique notify sound1")
        .unwrap()
        .contains("音效 1"));
    assert!(explain_line("unique notify sound3")
        .unwrap()
        .contains("音效 3"));
    assert!(explain_line("unique notify sound7")
        .unwrap()
        .contains("音效 7"));
}

#[test]
fn sound_modes_above_seven_rendered() {
    // After widening parse_sound_keyword to accept 1..=255, the explain
    // output must surface those numbers too — not silently drop them.
    assert!(explain_line("unique notify sound8")
        .unwrap()
        .contains("音效 8"));
    assert!(explain_line("unique notify sound99")
        .unwrap()
        .contains("音效 99"));
    assert!(explain_line("unique notify sound255")
        .unwrap()
        .contains("音效 255"));
}
