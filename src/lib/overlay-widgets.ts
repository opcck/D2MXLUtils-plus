/**
 * Static registry of overlay positions and edit-mode repositionable widgets.
 *
 * Adding a new persisted overlay position:
 *   1. Add a default to OVERLAY_POSITION_DEFAULTS below.
 *   2. In the widget's component, read its position via
 *      `widgetPosition(id)` from `src/stores/widget-positions.svelte.ts`.
 *   3. Style with `top: {y}%; left: {x}%;` (percent of overlay size).
 *   4. Add to OVERLAY_WIDGETS only if it should appear in overlay edit mode.
 */

export const OVERLAY_POSITION_DEFAULTS = {
  notifications: { x: 1, y: 1 },
  'dps-meter': { x: 1, y: 1 },
  'loot-history': { x: 50, y: 25 },
  'item-search': { x: 30, y: 16 },
  'monster-hud': { x: 2, y: 12 },
  'item-hud': { x: 75, y: 12 },
} as const;

export type OverlayPositionId = keyof typeof OVERLAY_POSITION_DEFAULTS;

export interface OverlayWidgetSpec {
  /** Settings key — NEVER change after release. */
  id: OverlayPositionId;
  label: string;
  /** Pixels. Sizes the ghost and clamps drag. */
  ghostSize: { width: number; height: number };
}

export const OVERLAY_WIDGETS = [
  {
    id: 'notifications',
    label: '掉落通知',
    ghostSize: { width: 300, height: 80 },
  },
  {
    id: 'dps-meter',
    label: '秒伤统计 (DPS)',
    ghostSize: { width: 130, height: 110 },
  },
  {
    id: 'monster-hud',
    label: '怪物信息与抗性',
    ghostSize: { width: 220, height: 120 },
  },
  {
    id: 'item-hud',
    label: '物品详细信息',
    ghostSize: { width: 220, height: 90 },
  },
] as const satisfies readonly OverlayWidgetSpec[];

export type OverlayWidgetId = (typeof OVERLAY_WIDGETS)[number]['id'];
