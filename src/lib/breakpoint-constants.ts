export interface ClassInfo {
  id: number;
  name: string;
  token: string;
}

export interface MorphInfo {
  name: string;
  token: string;
  baseClass: string;
}

export interface MercInfo {
  id: number;
  name: string;
  token: string;
}

export interface DebuffInfo {
  name: string;
  value: number;
}

export interface WeaponType {
  token: string;
  name: string;
  primaryAnim: string;
  blockAnim: string;
}

export const CLASSES: ClassInfo[] = [
  { id: 0, name: '亚马逊 (Amazon)', token: 'AM' },
  { id: 1, name: '法师 (Sorceress)', token: 'SO' },
  { id: 2, name: '死灵法师 (Necromancer)', token: 'NE' },
  { id: 3, name: '圣骑士 (Paladin)', token: 'PA' },
  { id: 4, name: '野蛮人 (Barbarian)', token: 'BA' },
  { id: 5, name: '德鲁伊 (Druid)', token: 'DZ' },
  { id: 6, name: '刺客 (Assassin)', token: 'AI' },
];

export const MORPHS: MorphInfo[] = [
  { name: '狼人形态 (Werewolf)', token: '40', baseClass: 'DZ' },
  { name: '熊人变形 (Werebear)', token: 'TG', baseClass: 'DZ' },
  { name: '枭兽形态 (Wereowl)', token: 'OW', baseClass: 'DZ' },
  { name: '超级变身 (Superbeast)', token: '~Z', baseClass: 'PA' },
  { name: '死灵领主 (Deathlord)', token: '0N', baseClass: 'NE' },
  { name: '树人守卫 (Treewarden)', token: 'TH', baseClass: 'BA' },
];

export const MERCS: MercInfo[] = [
  { id: 0, name: '第一幕: 萝格斥候 (弓)', token: 'RG' },
  { id: 1, name: '第二幕: 城镇守卫 (矛)', token: 'GU' },
  { id: 2, name: '第二幕: 变形守卫 (矛)', token: 'GU' },
  { id: 3, name: '第三幕: 铁狼 (单手剑)', token: 'IW' },
  { id: 4, name: '第五幕: 野蛮人 (剑)', token: '0A' },
];

export const DEBUFFS: DebuffInfo[] = [
  { name: '无 (None)', value: 0 },
  { name: '衰老 (Decrepify -20)', value: -20 },
  { name: '佛博斯 (Phobos -20)', value: -20 },
  { name: '乌迪贤 (Uldyssian -30)', value: -30 },
  { name: '冰冷减速 (Chill -50)', value: -50 },
];

export const ANIM_TYPES = ['A1', 'SC', 'GH', 'BL'] as const;
export type AnimType = (typeof ANIM_TYPES)[number];

export const ANIM_TYPE_LABELS: Record<AnimType, string> = {
  A1: '攻击速度 (IAS)',
  SC: '施法速度 (FCR)',
  GH: '击中回复 (FHR)',
  BL: '格挡速度 (FBR)',
};

// Mirrors https://dev.median-xl.com/speedcalc/. primaryAnim is the COF
// weapon token used for A1/SC/GH; blockAnim for BL.
export const WEAPON_TYPES: WeaponType[] = [
  { token: 'swor', name: '单手剑', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: 'crsd', name: '水晶剑', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: '2hsd', name: '双手剑', primaryAnim: '2HS', blockAnim: '1HS' },
  { token: 'axe', name: '单手斧', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: '2hax', name: '双手斧', primaryAnim: 'STF', blockAnim: '1HS' },
  { token: 'mace', name: '钉锤', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: 'hamm', name: '锤子', primaryAnim: 'STF', blockAnim: '1HS' },
  { token: 'scep', name: '权杖', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: 'jave', name: '标枪', primaryAnim: '1HT', blockAnim: '1HT' },
  { token: 'spea', name: '矛', primaryAnim: '2HT', blockAnim: '1HS' },
  { token: 'scyh', name: '镰刀', primaryAnim: 'STF', blockAnim: '1HS' },
  { token: 'knif', name: '匕首', primaryAnim: '1HT', blockAnim: '1HT' },
  { token: 'tkni', name: '飞刀', primaryAnim: '1HT', blockAnim: '1HT' },
  { token: 'taxe', name: '飞斧', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: 'staf', name: '法杖', primaryAnim: 'STF', blockAnim: '1HS' },
  { token: 'bow', name: '弓', primaryAnim: 'BOW', blockAnim: '1HS' },
  { token: 'xbow', name: '弩', primaryAnim: 'XBW', blockAnim: '1HS' },
  { token: 'abow', name: '专属-亚马逊 弓', primaryAnim: 'BOW', blockAnim: '1HS' },
  { token: 'aspe', name: '专属-亚马逊 矛', primaryAnim: '2HT', blockAnim: '1HS' },
  { token: 'ajav', name: '专属-亚马逊 标枪', primaryAnim: '1HT', blockAnim: '1HT' },
  { token: 'h2h', name: '专属-刺客 爪', primaryAnim: 'HT1', blockAnim: 'HT1' },
  { token: 'nagi', name: '专属-刺客 薙刀', primaryAnim: 'STF', blockAnim: '1HS' },
  { token: 'bswd', name: '专属-野蛮人 剑', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: 'baxe', name: '专属-野蛮人 单手斧', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: '2hbx', name: '专属-野蛮人 双手斧', primaryAnim: 'STF', blockAnim: '1HS' },
  { token: 'dbow', name: '专属-德鲁伊 弓', primaryAnim: 'BOW', blockAnim: '1HS' },
  { token: 'dstf', name: '专属-德鲁伊 法杖', primaryAnim: 'STF', blockAnim: '1HS' },
  { token: 'nscy', name: '专属-死灵法师 镰刀', primaryAnim: 'STF', blockAnim: '1HS' },
  { token: 'nstf', name: '专属-死灵法师 法杖', primaryAnim: 'STF', blockAnim: '1HS' },
  { token: 'nknf', name: '专属-死灵法师 匕首', primaryAnim: '1HT', blockAnim: '1HT' },
  { token: 'nxbw', name: '专属-死灵法师 弩', primaryAnim: 'XBW', blockAnim: '1HS' },
  { token: 'wand', name: '专属-死灵法师 魔杖', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: 'pclb', name: '专属-圣骑士 木棒', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: 'pmac', name: '专属-圣骑士 钉锤', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: 'pham', name: '专属-圣骑士 锤子', primaryAnim: 'STF', blockAnim: '1HS' },
  { token: 'pspe', name: '专属-圣骑士 矛', primaryAnim: '2HT', blockAnim: '1HS' },
  { token: 'orb', name: '专属-法师 宝珠', primaryAnim: '1HS', blockAnim: '1HS' },
  { token: 'scrd', name: '专属-法师 水晶剑', primaryAnim: '1HS', blockAnim: '1HS' },
];

// Wereform morphs and the Rogue/Town Guard/Shapeshifter mercs force HTH
// weaponAnim for every anim type — mirrors dev.median-xl.com/speedcalc's
// per-`subject` ATanim/GHanim switches, which override to `*A1HTH`/`*GHHTH`
// for all of these regardless of equipped weapon (Iron Wolf and Son of
// Harrogath mercs get no such override there, so they're absent here too).
export interface CharOverride {
  allAnims: string;
  /** Overrides the COF prefix used for cast (normally "SC") — Deathlord
   *  and Treewarden's SC table reuses their own `A1HTH` (attack) anim
   *  data in the reference calculator, not a dedicated cast anim. */
  castPrefix?: string;
  /** Overrides the COF prefix used for block (normally "BL") — Deathlord,
   *  Treewarden and Superbeast's BL table reuses their `GHHTH` (hit
   *  recovery) anim data in the reference calculator: `TH`/`~Z` have no
   *  distinct `*BLHTH` entry in SpeedcalcData.txt at all, and the
   *  reference ignores `0N`'s (which does exist) the same way. */
  blPrefix?: string;
  /** Werebear's `TGBLHTH` animSpeed in SpeedcalcData.txt is 220, but the
   *  reference calculator hardcodes 200 for its Block table instead of
   *  using that value — replicated here to match its output exactly. */
  blAnimSpeedOverride?: number;
}

export const CHAR_OVERRIDES: Record<string, CharOverride> = {
  '40': { allAnims: 'HTH' },
  TG: { allAnims: 'HTH', blAnimSpeedOverride: 200 },
  OW: { allAnims: 'HTH' },
  '~Z': { allAnims: 'HTH', blPrefix: 'GH' },
  '0N': { allAnims: 'HTH', castPrefix: 'A1', blPrefix: 'GH' },
  TH: { allAnims: 'HTH', castPrefix: 'A1', blPrefix: 'GH' },
  RG: { allAnims: 'HTH' },
  GU: { allAnims: 'HTH' },
};

const CLASS_SPECIFIC_TOKENS: Record<string, string[]> = {
  AM: ['abow', 'aspe', 'ajav'],
  AI: ['h2h', 'nagi'],
  BA: ['bswd', 'baxe', '2hbx'],
  DZ: ['dbow', 'dstf'],
  NE: ['nscy', 'nstf', 'nknf', 'nxbw', 'wand'],
  PA: ['pclb', 'pmac', 'pham', 'pspe'],
  SO: ['orb', 'scrd'],
};

// MXL mercs can wield class-specific weapons (Rogue ↔ Amazon Bows etc.).
// Town Guard and Shapeshifter share token "GU" and the same allowed list.
const MERC_WEAPON_TOKENS: Record<string, string[]> = {
  RG: ['bow', 'abow'],
  GU: ['jave', 'spea', 'scyh', 'pspe'],
  IW: ['swor', 'crsd'],
  '0A': ['swor', 'crsd', '2hsd', 'bswd'],
};

const ALL_CLASS_SPECIFIC = new Set(Object.values(CLASS_SPECIFIC_TOKENS).flat());

const GENERIC_WEAPON_TOKENS = WEAPON_TYPES.filter((wt) => !ALL_CLASS_SPECIFIC.has(wt.token)).map(
  (wt) => wt.token,
);

export const WEAPON_TYPE_MAP = new Map(WEAPON_TYPES.map((wt) => [wt.token, wt]));

export function getWeaponTypesForCharacter(charToken: string): WeaponType[] {
  const mercTokens = MERC_WEAPON_TOKENS[charToken];
  if (mercTokens) {
    const allowed = new Set(mercTokens);
    return WEAPON_TYPES.filter((wt) => allowed.has(wt.token));
  }
  const classSpecific = CLASS_SPECIFIC_TOKENS[charToken] ?? [];
  const allowed = new Set([...GENERIC_WEAPON_TOKENS, ...classSpecific]);
  return WEAPON_TYPES.filter((wt) => allowed.has(wt.token));
}

// Fallback for findWeaponTypeByWclass when the live family chain misses
// every WEAPON_TYPES token: pick the class-preferred weapon for the
// (class, primaryAnim) pair.
const CLASS_PREFERRED_ANIM: Record<string, Record<string, string>> = {
  AM: { BOW: 'abow', '2HT': 'aspe', '1HT': 'ajav' },
  AI: { HT1: 'h2h', STF: 'nagi' },
  BA: { '1HS': 'bswd' },
  DZ: { BOW: 'dbow', STF: 'dstf' },
  NE: { '1HT': 'nknf', STF: 'nscy', XBW: 'nxbw' },
  PA: { '1HS': 'pclb', '2HT': 'pspe', STF: 'pham' },
  SO: { '1HS': 'scrd' },
};

export function findWeaponTypeByWclass(
  charToken: string,
  wclass: string,
  familyCodes?: string[],
): WeaponType | null {
  if (familyCodes && familyCodes.length > 0) {
    for (const code of familyCodes) {
      const wt = WEAPON_TYPE_MAP.get(code.toLowerCase());
      if (wt) return wt;
    }
  }
  const anim = wclass.toUpperCase();
  const preferred = CLASS_PREFERRED_ANIM[charToken]?.[anim];
  if (preferred) {
    const wt = WEAPON_TYPE_MAP.get(preferred);
    if (wt) return wt;
  }
  const available = getWeaponTypesForCharacter(charToken);
  return available.find((wt) => wt.primaryAnim === anim) ?? null;
}

export const STARTING_FRAME_CLASSES = new Set(['AM', 'SO']);
// Mirrors the reference calculator's `StartingFrame = 2` weapon cases
// exactly: 1HT (daggers/javelins), 1HS (default bucket + Throwing Axes),
// 2HS (Two-Handed Swords), and STF. Spears (2HT) are a separate weapon
// case there that never sets `StartingFrame` — it stays 0 — so 2HT must
// NOT be in this set even though it looks like a sibling of 1HT/2HS.
export const STARTING_FRAME_ANIMS = new Set(['1HS', '1HT', '2HS', 'STF']);

export const THROWING_FAMILIES = new Set(['tkni', 'jave', 'ajav', 'taxe']);

// Mercs and wereforms can't throw.
export const THROWING_DISALLOWED_TOKENS = new Set([
  'RG',
  'GU',
  'IW',
  '0A',
  '40',
  'TG',
  'OW',
  '~Z',
  '0N',
  'TH',
]);

// Families a Barb can't dual-wield (2H grip / ranged / caster off-hand).
// Everything else — including normally-2H melee like Two-Handed Swords or
// Throwing Axes — is fair game.
export const BARB_DW_EXCLUDED_FAMILIES = new Set([
  'spea',
  'aspe',
  'pspe',
  'scyh',
  'nscy',
  'staf',
  'dstf',
  'nstf',
  'bow',
  'abow',
  'dbow',
  'xbow',
  'nxbw',
  'nagi',
  'wand',
  'orb',
]);

// Per-class weaponAnim overrides for SC/GH/BL when DW is active. Looked up
// via `{charToken}{animType}{weaponAnim}` (e.g. BAGH1SS, AISCHT2, AIBLHT2).
// `null` = suppress that anim type (Barb BL — can't block while DW). Missing
// key = anim unchanged; A1 always stays on the base primaryAnim because the
// speedcalc JS comments "Double attack animations are currently not in use".
export const DW_WEAPON_ANIMS: Record<string, Partial<Record<AnimType, string | null>>> = {
  BA: { SC: '1SS', GH: '1SS', BL: null },
  AI: { SC: 'HT2', GH: 'HT2', BL: 'HT2' },
};

// Maul-class hammers (WSM == 10) swing as 1HS, not STF, for A1/SC/GH.
export function effectiveWeaponAnimForBase(
  weapon: WeaponType,
  wsm: number,
  animType: AnimType,
): string {
  if (weapon.token === 'hamm' && wsm === 10) {
    if (animType === 'BL') return weapon.blockAnim;
    return '1HS';
  }
  if (animType === 'BL') return weapon.blockAnim;
  return weapon.primaryAnim;
}
