<script lang="ts">
  import { onMount } from 'svelte';
  import { CLASSES } from '../lib/breakpoint-constants';
  import { statsStore, type UnitStats } from '../stores';

  let activeEntity = $state<'player' | 'merc'>('player');

  let active = $derived(activeEntity === 'player' ? statsStore.player : statsStore.merc);
  // The first `stats-update` payload can lag behind the tab opening (or the
  // player loading into game) by a poll cycle or more — each tick reads ~100
  // stat ids one-by-one via the injector, twice (player + merc). Rather than
  // show a blank pane until that first payload lands, render the section/row
  // *shells* as soon as we know a game is running and fill them with a
  // skeleton placeholder instead of leaving the whole tab empty. `statsStore`
  // lives outside this component, so switching tabs away and back re-shows
  // its cached last-known data instantly instead of hitting this state again.
  let awaitingFirstData = $derived(
    statsStore.gameStatus === 'ingame' && !active && !statsStore.receivedFirstPayload,
  );

  // Life-per-Vitality / Mana-per-Energy growth factors, indexed by class id
  // (0=Amazon..6=Assassin, matches `CLASSES` in breakpoint-constants.ts).
  // These differ from vanilla D2's per-class values, so don't fall back to
  // vanilla numbers here. Both arrays confirmed against the official Median
  // XL class docs (docs.median-xl.com/doc/class/<class>).
  //
  // Known limitation: wielding Azurewrath multiplies Life-per-Vitality by
  // 0.9 — not modeled here (would need to detect the specific equipped
  // unique), so the "from Vitality" estimate reads ~11% high while it's
  // equipped. Mentally scale the figure by 0.9 in that case.
  const LIFE_PER_VIT = [2.25, 2.25, 1.5, 2.75, 2.75, 2.25, 2.25];
  const MANA_PER_ENE = [2.25, 2.5, 3, 1.5, 1.5, 3, 2.25];

  // Elemental resist cap is 75% by default, extendable via `+max resist`
  // item stats. Physical resist cap is a fixed 50% with no known
  // cap-increasing stat — its value can still overcap above 50 through
  // other means, it just isn't shown with a computed max here.
  const BASE_RESIST_CAP = 75;
  const PHYSICAL_RESIST_CAP = 50;
  // Elemental/poison resists have a hard absolute ceiling of 90% regardless
  // of how much +max-resist bonus is stacked — the displayed max is capped
  // here, with any excess bonus surfaced as "(+X%)" so it's visible without
  // implying it actually raises the real cap.
  const ABSOLUTE_RESIST_CAP = 90;

  function num(stats: Record<string, number>, id: number): number {
    return stats[String(id)] ?? 0;
  }

  /** Splits an aggregate stat into base / flat item-and-skill bonus / percent
   *  item-and-skill bonus, expressed as three terms that sum exactly to the
   *  total (mirrors D2Stats.au3:651's `iTotal/(1+iPercent/100) - iBase` for
   *  the flat portion, then folds any rounding into the percent term's
   *  absolute point value so the displayed numbers always add up). */
  function splitBonus(
    total: number,
    pct: number,
    base: number,
  ): { flat: number; pctPoints: number } {
    const denom = 1 + pct / 100;
    const flat = denom > 0 ? Math.ceil(total / denom - base) : 0;
    const pctPoints = total - base - flat;
    return { flat, pctPoints };
  }

  function attributeBreakdown(u: UnitStats, statId: number, pctId: number): string {
    const base = u.baseStats[String(statId)] ?? 0;
    const total = num(u.stats, statId);
    const pct = num(u.stats, pctId);
    const { flat, pctPoints } = splitBonus(total, pct, base);
    return `${base} (裸体) + ${flat} (装备技能) + ${pctPoints} (${pct}%) = ${total}`;
  }

  function lifeManaBreakdown(
    u: UnitStats,
    statId: number,
    pctId: number,
    vitalId: number,
    perVital: number,
  ): string {
    const total = num(u.stats, statId);
    const pct = num(u.stats, pctId);
    const vitalTotal = num(u.stats, vitalId);
    const fromVital = Math.floor(vitalTotal * perVital);
    const vitalLabel = vitalId === 3 ? '体力' : '精力';
    return `${total} (+${pct}%, 约 ${fromVital} 来自${vitalLabel})`;
  }

  function experienceLabel(u: UnitStats): string {
    const current = num(u.stats, 13);
    const levelStart = num(u.stats, 905);
    const levelNext = num(u.stats, 906);
    if (levelNext < 0) {
      return `${current.toLocaleString()} (MAX)`;
    }
    const span = levelNext - levelStart;
    const pct = span > 0 ? Math.max(0, Math.min(100, ((current - levelStart) / span) * 100)) : 0;
    return `${current.toLocaleString()} / ${levelNext.toLocaleString()} (${pct.toFixed(1)}%)`;
  }

  function spellFocusCapLabel(u: UnitStats): string {
    // stats[904] is effective Spell Focus, not yet divided by 10 — done
    // here (not in Rust) to keep the fractional percent (105 SF = 10.5%).
    const effectiveSf = num(u.stats, 904);
    const raw = effectiveSf / 10;
    const capped = Math.min(raw, 100);
    return raw > 100
      ? `${capped.toFixed(1)}% (overcap, raw ${raw.toFixed(1)}%)`
      : `${capped.toFixed(1)}%`;
  }

  function lifeRegenLabel(u: UnitStats): string {
    // D2 stat 74 is item_regenlife (25 frames/sec, value / 256 per frame).
    // Actual HP/sec = (raw * 25) / 256.
    const raw = num(u.stats, 74);
    return String(Math.round((raw * 25) / 256));
  }

  function resistLabel(
    u: UnitStats,
    currentId: number,
    maxBonusId: number | null,
    capAt90 = false,
  ): string {
    const current = num(u.stats, currentId);
    const rawMax =
      maxBonusId === null ? PHYSICAL_RESIST_CAP : BASE_RESIST_CAP + num(u.stats, maxBonusId);
    if (!capAt90) {
      return `${current}% / max ${rawMax}%`;
    }
    const cappedMax = Math.min(rawMax, ABSOLUTE_RESIST_CAP);
    const overcap = rawMax - ABSOLUTE_RESIST_CAP;
    return overcap > 0
      ? `${current}% / max ${cappedMax}% (+${overcap}%)`
      : `${current}% / max ${cappedMax}%`;
  }

  interface StatRow {
    label: string;
    render: (u: UnitStats) => string;
    tooltip?: string;
    colorVar?: string;
    /** When set, the row only renders if this returns truthy. */
    visible?: (u: UnitStats) => boolean;
  }

  interface StatSection {
    title: string;
    rows: StatRow[];
  }

  const tpl =
    (template: string): ((u: UnitStats) => string) =>
    (u) =>
      template.replace(/\{(\w+)\}/g, (_m, key: string) => {
        switch (key) {
          case '__phys1h':
            return u.damage ? `${u.damage.physMin1h}-${u.damage.physMax1h}` : '0-0';
          case '__phys2h':
            return u.damage ? `${u.damage.physMin2h}-${u.damage.physMax2h}` : '0-0';
          case '__fire':
            return u.damage ? `${u.damage.fireMin}-${u.damage.fireMax}` : '0-0';
          case '__cold':
            return u.damage ? `${u.damage.coldMin}-${u.damage.coldMax}` : '0-0';
          case '__lightning':
            return u.damage ? `${u.damage.lightningMin}-${u.damage.lightningMax}` : '0-0';
          case '__magic':
            return u.damage ? `${u.damage.magicMin}-${u.damage.magicMax}` : '0-0';
          case '__poison':
            return u.damage ? `${u.damage.poisonMinPerSec}-${u.damage.poisonMaxPerSec}` : '0-0';
          case '__strdmg':
            return String(u.damage?.strDamageBonusPct ?? 0);
          case '__dexdmg':
            return String(u.damage?.dexDamageBonusPct ?? 0);
          default:
            return String(num(u.stats, Number(key)));
        }
      });

  // Mirrors D2Stats.au3's `CreateGUI()` stat panels (Basic / Page 1 / Page 2),
  // D2Stats.au3:2510-2636, reorganized per in-house feedback (attribute and
  // life/mana breakdowns, weapon-damage regrouping, spell-damage renaming).
  // Stat ids are vanilla ItemStatCost.txt row indices.
  function buildSections(u: UnitStats): StatSection[] {
    return [
      {
        title: '综合',
        rows: [
          { label: '职业', render: () => CLASSES.find((c) => c.id === u.class)?.name ?? '?' },
          { label: '等级', render: tpl('{12}') },
          {
            label: '经验值',
            render: experienceLabel,
            tooltip: '当前经验值 / 升到下级所需经验值（大致估算）',
          },
          { label: '携带金币', render: tpl('{14}') },
          { label: '储物箱金币', render: tpl('{15}') },
          {
            label: '属性印章',
            render: tpl('{185} / 400'),
            tooltip: '由击杀敌人或分解物品获得的属性印章 (Signets of Attribute)，上限 400 点',
          },
          {
            label: '魔法物品获取率 (MF)',
            render: tpl('{80}%'),
            tooltip: '增加掉落物品为魔法、稀有、套装或暗金的概率',
          },
          { label: '额外金钱获取 (GF)', render: tpl('{79}%') },
          { label: '获得额外经验值', render: tpl('+{85}%') },
        ],
      },
      {
        title: '主要属性',
        rows: [
          {
            label: '力量 (STR)',
            render: (u) => attributeBreakdown(u, 0, 359),
            tooltip:
              '基础点数 + 装备/技能加成 + 百分比加成 = 最终总计。需求穿上大部分装备，提升部分BD的攻击',
          },
          {
            label: '敏捷 (DEX)',
            render: (u) => attributeBreakdown(u, 2, 360),
            tooltip:
              '基础点数 + 装备/技能加成 + 百分比加成 = 最终总计。需求穿上某些装备，提供攻击速度、盾牌格挡速度，增加部分BD的伤害',
          },
          {
            label: '体力 (VIT)',
            render: (u) => attributeBreakdown(u, 3, 362),
            tooltip: '基础点数 + 装备/技能加成 + 百分比加成 = 最终总计。提升生命',
          },
          {
            label: '精力 (ENG)',
            render: (u) => attributeBreakdown(u, 1, 361),
            tooltip:
              '基础点数 + 装备/技能加成 + 百分比加成 = 最终总计。提升法力点数，增加大部分法术的伤害',
          },
          {
            label: '生命',
            render: (u) => lifeManaBreakdown(u, 7, 76, 3, LIFE_PER_VIT[u.class] ?? 2),
            tooltip:
              '总生命值直接读取自游戏。%生命加成作用于包括体力换算的全部生命池。“来自体力”为估算值（当前总体力 × 职业系数）。装备碧蓝怒火(Azurewrath)时体力生命收益为0.9倍。',
          },
          {
            label: '法力',
            render: (u) => lifeManaBreakdown(u, 9, 77, 1, MANA_PER_ENE[u.class] ?? 2),
            tooltip:
              '总法力值直接读取自游戏。%法力加成作用于包括精力换算的全部法力池。“来自精力”为估算值（当前总精力 × 职业系数）。',
          },
        ],
      },
      {
        title: '武器伤害',
        rows: [
          { label: '火焰伤害', render: tpl('{__fire}'), colorVar: 'var(--stat-fire, #e05d44)' },
          { label: '冰冷伤害', render: tpl('{__cold}'), colorVar: 'var(--stat-cold, #5b9bd5)' },
          {
            label: '闪电伤害',
            render: tpl('{__lightning}'),
            colorVar: 'var(--stat-lightning, #d4b106)',
          },
          { label: '魔法伤害', render: tpl('{__magic}'), colorVar: 'var(--stat-magic, #b366cc)' },
          {
            label: '毒素伤害',
            render: tpl('{__poison}/秒'),
            colorVar: 'var(--stat-poison, #4caf50)',
          },
          {
            label: '固有元素伤害乘数',
            render: tpl('+{484}%'),
            tooltip:
              '固有元素伤害乘数，只有特定武器提供固有元素伤害转换加成 (例如自带元素伤害的弓、爪等)。仅对自带元素伤害的底模生效。',
          },
          {
            label: '增强伤害 (ED)',
            render: tpl('{25}%'),
            tooltip: '物理武器伤害乘数 (转换前)，添加力量/敏捷伤害加成',
          },
          {
            label: '力量武器物理增伤 (WPD)',
            render: tpl('+{__strdmg}%'),
            tooltip: '从力量点数获得的武器物理伤害 (WPD)',
          },
          {
            label: '敏捷武器物理增伤 (WPD)',
            render: tpl('+{__dexdmg}%'),
            tooltip: '从敏捷点数获得的武器物理伤害 (WPD)',
          },
          { label: '单手物理伤害', render: tpl('{__phys1h}') },
          { label: '双手/远程物理伤害', render: tpl('{__phys2h}') },
        ],
      },
      {
        title: '防御',
        rows: [
          { label: '总防御加成', render: tpl('{171}%') },
          {
            label: '准确率 (AR)',
            render: tpl('+{119}% / +{19} 基础'),
            tooltip: '增加你的武器伤害技能准确率',
          },
          {
            label: '物理伤害减少 (PDR)',
            render: tpl('{34}'),
            tooltip: '物理伤害减少，先于抗性生效',
          },
          {
            label: '元素伤害减少 (EDR)',
            render: tpl('{35}'),
            tooltip: '元素伤害减少，先于抗性生效',
          },
          {
            label: '毅力值 (Grit)',
            render: tpl('{184}%'),
            tooltip: '所有伤害减少 - 基于总力量，与其它来源相乘',
          },
          {
            label: '闪避 (Dodge)',
            render: tpl('{338}%'),
            tooltip: '在静止、攻击、施法和转向时躲避飞弹的几率',
          },
          {
            label: '规避 (Avoid)',
            render: tpl('{339}%'),
            tooltip: '当站立，攻击，施法，转向时几率忽视飞弹攻击',
          },
          {
            label: '躲避 (Evade)',
            render: tpl('{340}%'),
            tooltip: '当移动时几率忽视近战和远程攻击',
          },
          {
            label: '压碎性打击 (CB)',
            render: tpl('{136}%'),
            tooltip: '命中时有机会减少敌人百分之一的生命，不对Boss生效',
          },
          {
            label: '致命攻击 (DS)',
            render: tpl('{141}%'),
            tooltip: '几率造成双倍武器伤害 (转换前)',
          },
          { label: '双倍打击 (CS)', render: tpl('{344}%') },
        ],
      },
      {
        title: '抗性',
        rows: [
          {
            label: '火焰抗性',
            render: (u) => resistLabel(u, 39, 40, true),
            colorVar: 'var(--stat-fire, #e05d44)',
            tooltip: '绝对上限为 90% — 超过极限的抗性可用于应对减益或敌方元素穿透',
          },
          {
            label: '冰冷抗性',
            render: (u) => resistLabel(u, 43, 44, true),
            colorVar: 'var(--stat-cold, #5b9bd5)',
            tooltip: '绝对上限为 90% — 超过极限的抗性可用于应对减益或敌方元素穿透',
          },
          {
            label: '闪电抗性',
            render: (u) => resistLabel(u, 41, 42, true),
            colorVar: 'var(--stat-lightning, #d4b106)',
            tooltip: '绝对上限为 90% — 超过极限的抗性可用于应对减益或敌方元素穿透',
          },
          {
            label: '毒素抗性',
            render: (u) => resistLabel(u, 45, 46, true),
            colorVar: 'var(--stat-poison, #4caf50)',
            tooltip: '绝对上限为 90% — 超过极限的抗性可用于应对减益或敌方元素穿透',
          },
          {
            label: '魔法抵抗',
            render: (u) => resistLabel(u, 37, 38),
            colorVar: 'var(--stat-magic, #b366cc)',
          },
          {
            label: '物理抗性',
            render: (u) => resistLabel(u, 36, null),
            tooltip: '固定 50% 上限 — 暂无提升物理抗性上限的词条',
          },
          {
            label: '降低诅咒持续时间 (CLR)',
            render: tpl('{109}%'),
            tooltip: '减少对角色的诅咒持续时间',
          },
          {
            label: '减少毒素持续时间 (PLR)',
            render: tpl('{110}%'),
            tooltip: '减少对人物的毒素伤害时间影响长度',
          },
        ],
      },
      {
        title: '法术伤害与穿透',
        rows: [
          {
            label: '火焰系',
            render: tpl('{329}% 伤害 / {333}% 穿透'),
            colorVar: 'var(--stat-fire, #e05d44)',
            tooltip: '百分比提升火焰法术伤害 / 百分比减少敌人火焰抗性',
          },
          {
            label: '冰冷系',
            render: tpl('{331}% 伤害 / {335}% 穿透'),
            colorVar: 'var(--stat-cold, #5b9bd5)',
            tooltip: '百分比提升冰冷法术伤害 / 百分比减少敌人冰冷抗性',
          },
          {
            label: '闪电系',
            render: tpl('{330}% 伤害 / {334}% 穿透'),
            colorVar: 'var(--stat-lightning, #d4b106)',
            tooltip: '百分比提升闪电法术伤害 / 百分比减少敌人闪电抗性',
          },
          {
            label: '毒素系',
            render: tpl('{332}% 伤害 / {336}% 穿透'),
            colorVar: 'var(--stat-poison, #4caf50)',
            tooltip: '百分比提升毒素法术伤害 / 百分比减少敌人毒素抗性',
          },
          {
            label: '毒素技能持续时间',
            render: tpl('{431}%'),
            colorVar: 'var(--stat-poison, #4caf50)',
            tooltip: '增加毒素法术的持续时间',
          },
          { label: '物理 / 魔法穿透', render: tpl('{357}% / 0%') },
          { label: '法术专注 (基础数值)', render: tpl('{485}') },
          {
            label: '法术专注 (%)',
            render: tpl('+{488}%'),
            tooltip: '来自装备/符文的法术专注百分比加成',
          },
          {
            label: '法术专注伤害加成',
            render: spellFocusCapLabel,
            tooltip: '提升伤害乘数至法术专注，加 1% 总伤害 - 每 10 法术专注 (上限 100%)',
          },
          {
            label: '精力法术伤害加成',
            render: tpl('+{907}%'),
            tooltip: '提升法力点数，增加大部分法术的伤害，无上限',
          },
        ],
      },
      {
        title: '速度',
        rows: [
          {
            label: '攻击速度 (IAS)',
            render: tpl('{93}% 装备 / {68}% 技能'),
            tooltip: '攻击速度提高近战挥击或远程武器攻击的速度',
          },
          {
            label: '击中回复 (FHR)',
            render: tpl('{99}% 装备 / {69}% 技能'),
            tooltip: '击中回复决定你被击中时回复的速度',
          },
          {
            label: '格挡速度 (FBR)',
            render: tpl('{102}% 装备 / {69}% 技能'),
            tooltip: '格挡率决定你在格挡攻击后恢复的速度',
          },
          {
            label: '跑步/行走速度 (FRW)',
            render: tpl('{96}% 装备 / {67}% 技能'),
            tooltip: '提高走路和冲刺速度 (收益递减)',
          },
          {
            label: '施法速度 (FCR)',
            render: tpl('{105}%'),
            tooltip: '施法速度提升你释放法术的速度',
          },
        ],
      },
      {
        title: '伤害吸收',
        rows: [
          {
            label: '吸收火焰伤害',
            render: tpl('{142}% / {143} 固定值'),
            colorVar: 'var(--stat-fire, #e05d44)',
            tooltip:
              '吸收百分比在抵抗后应用，减少一定百分比的伤害并治疗该数值；固定吸收在吸收百分比后应用并恢复特定值',
          },
          {
            label: '吸收冰冷伤害',
            render: tpl('{148}% / {149} 固定值'),
            colorVar: 'var(--stat-cold, #5b9bd5)',
            tooltip:
              '吸收百分比在抵抗后应用，减少一定百分比的伤害并治疗该数值；固定吸收在吸收百分比后应用并恢复特定值',
          },
          {
            label: '吸收闪电伤害',
            render: tpl('{144}% / {145} 固定值'),
            colorVar: 'var(--stat-lightning, #d4b106)',
            tooltip:
              '吸收百分比在抵抗后应用，减少一定百分比的伤害并治疗该数值；固定吸收在吸收百分比后应用并恢复特定值',
          },
          {
            label: '吸收魔法伤害',
            render: tpl('{146}% / {147} 固定值'),
            colorVar: 'var(--stat-magic, #b366cc)',
            tooltip:
              '吸收百分比在抵抗后应用，减少一定百分比的伤害并治疗该数值；固定吸收在吸收百分比后应用并恢复特定值',
          },
        ],
      },
      {
        title: '持续作战能力',
        rows: [
          {
            label: '生命 / 法力偷取',
            render: tpl('{60}% / {62}%'),
            tooltip: '偷取造成的武器物理伤害的百分比生命/法力，噩梦和地狱中的效率降低',
          },
          { label: '杀敌后获得生命 / 法力 (EK)', render: tpl('{86} / {138}') },
          {
            label: '击中时获得生命 / 法力 (LoS/MoS)',
            render: tpl('{208} / {209}'),
            tooltip: '于每次被武器击中时获得生命或法力',
          },
          { label: '攻击时获得生命 / 法力 (LA8/MA8)', render: tpl('{210} / {295}') },
        ],
      },
      {
        title: '召唤',
        rows: [
          { label: '召唤物生命', render: tpl('+{444}%') },
          { label: '召唤物伤害', render: tpl('+{470}%') },
          { label: '召唤物所有抗性', render: tpl('+{487}%') },
          { label: '召唤物准确率', render: tpl('+{500}%') },
        ],
      },
      {
        title: '杂项',
        rows: [
          {
            label: '法术持续时间加成',
            render: tpl('{409}%'),
            tooltip: '增加大部分法术的持续效果',
          },
          {
            label: '生命恢复/秒',
            render: lifeRegenLabel,
            tooltip: '每秒恢复的生命值',
          },
          {
            label: '法力恢复速度',
            render: tpl('{27}%'),
            tooltip: '提供自然法力回复的百分比加成',
          },
          {
            label: '目标承受额外伤害 (DTDU)',
            render: tpl('{489}'),
            tooltip: '只影响武器物理伤害',
          },
          { label: '对恶魔的伤害', render: tpl('+{121}%') },
          { label: '对不死生物的伤害', render: tpl('+{122}%') },
          {
            label: '减速目标',
            render: tpl('{150}% / {376}%'),
            tooltip: '造成武器伤害时降低目标移动和攻击速度，对Boss的上限为 25%',
          },
          {
            label: '缓速攻击者',
            render: tpl('{363}% / {493}%'),
            tooltip: '被远程飞弹击中时缓速攻击者',
          },
        ],
      },
      {
        title: '特殊状态',
        rows: [
          {
            label: '杀死怪物彻底平息 (RIP)',
            render: tpl('已生效'),
            visible: (u) => num(u.stats, 108) >= 1,
            tooltip: '被杀死的怪物无法被复活或召唤',
          },
          {
            label: '冻结时间减半',
            render: tpl('已生效'),
            visible: (u) => num(u.stats, 118) >= 1,
          },
          { label: '无法被冻结', render: tpl('已生效'), visible: (u) => num(u.stats, 153) >= 1 },
        ],
      },
    ];
  }

  let visibleSections = $derived.by(() => {
    const unit = active;
    if (!unit) return [];
    return buildSections(unit)
      .map((section) => ({
        title: section.title,
        rows: section.rows
          .filter((row) => !row.visible || row.visible(unit))
          .map((row) => ({
            label: row.label,
            value: row.render(unit),
            tooltip: row.tooltip,
            colorVar: row.colorVar,
          })),
      }))
      .filter((section) => section.rows.length > 0);
  });

  // Zeroed stand-in unit, used only to pull the static section/row shape
  // (titles, labels, tooltips) out of `buildSections` while we wait for real
  // data — its computed values are never shown. Rows that are conditionally
  // `visible` based on real data (e.g. the Flags section) can't be resolved
  // from a stub and are left out of the skeleton; they appear once real data
  // arrives, same as any other stat that changes over time.
  const EMPTY_UNIT: UnitStats = { class: 0, stats: {}, baseStats: {}, damage: null };
  let skeletonSections = $derived.by(() =>
    buildSections(EMPTY_UNIT)
      .filter((section) => section.rows.some((row) => !row.visible))
      .map((section) => ({
        title: section.title,
        rows: section.rows.filter((row) => !row.visible),
      })),
  );

  onMount(() => {
    statsStore.initListeners();
    statsStore.startPolling();

    return () => {
      statsStore.stopPolling();
    };
  });
</script>

<div class="stats-tab">
  <div class="entity-toggle">
    <button
      class="entity-btn"
      class:active={activeEntity === 'player'}
      onclick={() => {
        activeEntity = 'player';
      }}
    >
      玩家角色
    </button>
    <button
      class="entity-btn"
      class:active={activeEntity === 'merc'}
      onclick={() => {
        activeEntity = 'merc';
      }}
    >
      雇佣兵
    </button>
  </div>

  {#if !active && !awaitingFirstData}
    <p class="no-data">
      暂无数据 — 请确保暗黑破坏神II已在运行且{activeEntity === 'merc'
        ? '已雇佣随从'
        : '角色已载入游戏'}。
    </p>
  {:else if awaitingFirstData}
    <div class="stats-grid">
      {#each skeletonSections as section (section.title)}
        <div class="stats-card">
          <h3 class="stats-card-title">{section.title}</h3>
          <div class="stats-list">
            {#each section.rows as row (row.label)}
              <div class="stat-row" title={row.tooltip}>
                <span class="label-cell">{row.label}</span>
                <span class="value-cell skeleton" style:color={row.colorVar}>&nbsp;</span>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="stats-grid">
      {#each visibleSections as section (section.title)}
        <div class="stats-card">
          <h3 class="stats-card-title">{section.title}</h3>
          <div class="stats-list">
            {#each section.rows as row (row.label)}
              <div class="stat-row" title={row.tooltip}>
                <span class="label-cell">{row.label}</span>
                <span class="value-cell" style:color={row.colorVar}>{row.value}</span>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .stats-tab {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    height: 100%;
    overflow-y: auto;
  }

  .entity-toggle {
    display: flex;
    gap: var(--space-1);
  }

  .entity-btn {
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--border-primary);
    background: var(--bg-secondary);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--text-sm);
  }

  .entity-btn.active {
    background: var(--accent-primary);
    color: var(--accent-text);
    border-color: var(--accent-primary);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(450px, 1fr));
    gap: var(--space-3);
    align-content: start;
  }

  .stats-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    overflow: hidden;
    min-width: 0;
  }

  .stats-card-title {
    margin: 0;
    padding: var(--space-2);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border-primary);
    background: var(--bg-tertiary, transparent);
  }

  .stats-list {
    font-size: var(--text-sm);
    font-family: var(--font-mono);
  }

  .stat-row {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    column-gap: var(--space-3);
    padding: var(--space-1) var(--space-2);
    border-bottom: 1px solid var(--border-primary);
  }

  .stat-row:last-child {
    border-bottom: none;
  }

  .label-cell {
    color: var(--text-secondary);
    flex: 0 0 210px;
  }

  .value-cell {
    text-align: right;
    color: var(--text-primary);
    font-weight: 500;
    word-break: break-word;
    flex: 1 1 auto;
    min-width: 0;
  }

  .no-data {
    color: var(--text-muted);
    font-size: var(--text-sm);
    text-align: center;
    padding: var(--space-4);
  }

  .value-cell.skeleton {
    display: inline-block;
    width: 64px;
    height: 0.9em;
    border-radius: var(--radius-sm);
    background: linear-gradient(
      90deg,
      var(--bg-tertiary) 25%,
      var(--border-primary) 50%,
      var(--bg-tertiary) 75%
    );
    background-size: 200% 100%;
    animation: skeleton-shimmer 1.4s linear infinite;
  }

  @keyframes skeleton-shimmer {
    from {
      background-position: 200% 0;
    }
    to {
      background-position: -200% 0;
    }
  }
</style>
