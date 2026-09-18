<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { settingsStore } from '../stores';
  import { widgetPosition } from '../stores/widget-positions.svelte';

  interface MonsterInspectData {
    name: string;
    class_id: number;
    unit_id: number;
    hp_percent: number;
    cur_hp: number;
    max_hp: number;
    dr: number;
    mr: number;
    fr: number;
    lr: number;
    cr: number;
    pr: number;
  }

  interface ItemInspectData {
    name: string;
    class_id: number;
    unit_id: number;
    quality: string;
    sockets: number;
    is_ethereal: boolean;
  }

  type InspectPayload =
    | { kind: 'Monster'; data: MonsterInspectData }
    | { kind: 'Item'; data: ItemInspectData }
    | { kind: 'None' };

  let pos = $derived(widgetPosition('inspector'));

  let currentPayload = $state<InspectPayload>({ kind: 'None' });
  let fadeTimeout: number | null = null;

  let monsterInfo = $derived(settingsStore.settings.monsterInfo);
  let itemExtraInfo = $derived(settingsStore.settings.itemExtraInfo);

  let isActive = $derived(currentPayload.kind !== 'None');

  onMount(() => {
    let unlisten: (() => void) | null = null;

    listen<InspectPayload>('hover-inspector-update', (event) => {
      const payload = event.payload;

      if (payload.kind === 'Monster') {
        if (!monsterInfo.enabled) {
          clearFade();
          currentPayload = { kind: 'None' };
          return;
        }
        clearFade();
        currentPayload = payload;
      } else if (payload.kind === 'Item') {
        if (!itemExtraInfo.enabled) {
          clearFade();
          currentPayload = { kind: 'None' };
          return;
        }
        clearFade();
        currentPayload = payload;
      } else {
        triggerFade();
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
      clearFade();
    };
  });

  function clearFade() {
    if (fadeTimeout !== null) {
      clearTimeout(fadeTimeout);
      fadeTimeout = null;
    }
  }

  function triggerFade() {
    clearFade();
    fadeTimeout = window.setTimeout(() => {
      currentPayload = { kind: 'None' };
      fadeTimeout = null;
    }, 500);
  }
</script>

<div class="inspector-widget" class:is-active={isActive} style:left="{pos.x}%" style:top="{pos.y}%">
  {#if currentPayload.kind === 'Monster'}
    {@const monster = currentPayload.data}
    <!-- 怪物信息激活态 -->
    <div class="card-content monster-active">
      <div class="card-header">
        <div class="title-group">
          <span class="entity-name monster-name">{monster.name}</span>
          {#if monsterInfo.showClassId}
            <span class="id-tag">#{monster.class_id}</span>
          {/if}
        </div>
        <span class="hp-val">{monster.hp_percent}%</span>
      </div>

      <div class="hp-track">
        <div class="hp-bar" style="width: {monster.hp_percent}%"></div>
      </div>

      <div class="res-grid">
        <div class="res-item res-phys" class:immune={monster.dr >= 100}>
          <span class="tag">物</span>
          <span class="val">{monster.dr}%</span>
        </div>
        <div class="res-item res-magic" class:immune={monster.mr >= 100}>
          <span class="tag">魔</span>
          <span class="val">{monster.mr}%</span>
        </div>
        <div class="res-item res-fire" class:immune={monster.fr >= 100}>
          <span class="tag">火</span>
          <span class="val">{monster.fr}%</span>
        </div>
        <div class="res-item res-ltng" class:immune={monster.lr >= 100}>
          <span class="tag">电</span>
          <span class="val">{monster.lr}%</span>
        </div>
        <div class="res-item res-cold" class:immune={monster.cr >= 100}>
          <span class="tag">冰</span>
          <span class="val">{monster.cr}%</span>
        </div>
        <div class="res-item res-pois" class:immune={monster.pr >= 100}>
          <span class="tag">毒</span>
          <span class="val">{monster.pr}%</span>
        </div>
      </div>
    </div>
  {:else if currentPayload.kind === 'Item'}
    {@const item = currentPayload.data}
    <!-- 物品信息激活态 -->
    <div class="card-content item-active quality-{item.quality.toLowerCase()}">
      <div class="card-header">
        <span class="entity-name item-name">{item.name}</span>
        <span class="quality-badge">{item.quality}</span>
      </div>

      <div class="item-meta">
        <span class="meta-tag">UID: {item.unit_id}</span>
        <span class="meta-tag">CID: {item.class_id}</span>
        {#if itemExtraInfo.showSocketsAndEth}
          {#if item.sockets > 0}
            <span class="meta-tag socket-tag">{item.sockets}孔</span>
          {/if}
          {#if item.is_ethereal}
            <span class="meta-tag eth-tag">无形</span>
          {/if}
        {/if}
      </div>
    </div>
  {:else}
    <!-- 待机常驻态 (类似 DpsMeter 怠速状态) -->
    <div class="card-content standby-state">
      <div class="card-header standby-header">
        <span class="standby-title">目标侦测</span>
        <span class="standby-badge">待机中</span>
      </div>
      <div class="standby-hint">悬停怪物或物品查看</div>
      <div class="res-grid standby-grid">
        <div class="res-item"><span class="tag">物</span><span class="val">--</span></div>
        <div class="res-item"><span class="tag">魔</span><span class="val">--</span></div>
        <div class="res-item"><span class="tag">火</span><span class="val">--</span></div>
        <div class="res-item"><span class="tag">电</span><span class="val">--</span></div>
        <div class="res-item"><span class="tag">冰</span><span class="val">--</span></div>
        <div class="res-item"><span class="tag">毒</span><span class="val">--</span></div>
      </div>
    </div>
  {/if}
</div>

<style>
  .inspector-widget {
    position: absolute;
    min-width: 210px;
    max-width: 260px;
    background: rgba(15, 23, 42, 0.65);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 4px;
    padding: 6px 8px;
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    line-height: 1.35;
    color: #e2e8f0;
    opacity: 0.55;
    user-select: none;
    pointer-events: none; /* 拖拽由 OverlayEditGrid 中的 ghost 接管 */
    transition:
      opacity 200ms ease,
      border-color 200ms ease,
      box-shadow 200ms ease;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    z-index: 100;
  }

  .inspector-widget.is-active {
    opacity: 0.95;
    background: rgba(15, 23, 42, 0.88);
    border-color: rgba(255, 255, 255, 0.25);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.6);
  }

  .card-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 6px;
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }

  .entity-name {
    font-size: 12px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #f8fafc;
  }

  .id-tag {
    font-size: 10px;
    color: #94a3b8;
    background: rgba(255, 255, 255, 0.08);
    padding: 0 4px;
    border-radius: 2px;
  }

  .hp-val {
    font-size: 11px;
    font-weight: 700;
    color: #ef4444;
  }

  .hp-track {
    width: 100%;
    height: 3px;
    background: rgba(255, 255, 255, 0.12);
    border-radius: 2px;
    overflow: hidden;
  }

  .hp-bar {
    height: 100%;
    background: linear-gradient(90deg, #ef4444, #dc2626);
    border-radius: 2px;
    transition: width 0.15s ease;
  }

  /* 六系抗性网格 */
  .res-grid {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 3px;
    margin-top: 2px;
  }

  .res-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 2px 0;
    border-radius: 3px;
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .res-item .tag {
    font-size: 9px;
    font-weight: 600;
    opacity: 0.8;
  }

  .res-item .val {
    font-size: 10px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .res-phys {
    border-color: rgba(203, 213, 225, 0.4);
    color: #cbd5e1;
  }
  .res-magic {
    border-color: rgba(244, 114, 182, 0.4);
    color: #f472b6;
  }
  .res-fire {
    border-color: rgba(248, 113, 113, 0.4);
    color: #f87171;
  }
  .res-ltng {
    border-color: rgba(250, 204, 21, 0.4);
    color: #facc15;
  }
  .res-cold {
    border-color: rgba(96, 165, 250, 0.4);
    color: #60a5fa;
  }
  .res-pois {
    border-color: rgba(74, 222, 128, 0.4);
    color: #4ade80;
  }

  .res-item.immune {
    background: rgba(245, 158, 11, 0.2);
    border-color: #f59e0b;
    box-shadow: 0 0 4px rgba(245, 158, 11, 0.4);
    color: #fbbf24;
  }

  /* 物品部分 */
  .item-meta {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
  }

  .meta-tag {
    font-size: 10px;
    background: rgba(255, 255, 255, 0.08);
    padding: 0 4px;
    border-radius: 2px;
  }

  .quality-badge {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 0 4px;
    border-radius: 2px;
  }

  .quality-unique .item-name {
    color: #c8963e;
  }
  .quality-unique .quality-badge {
    background: rgba(200, 150, 62, 0.25);
    color: #c8963e;
  }

  .quality-set .item-name {
    color: #22c55e;
  }
  .quality-set .quality-badge {
    background: rgba(34, 197, 94, 0.25);
    color: #22c55e;
  }

  .quality-rare .item-name {
    color: #facc15;
  }
  .quality-rare .quality-badge {
    background: rgba(250, 204, 21, 0.25);
    color: #facc15;
  }

  .quality-magic .item-name {
    color: #60a5fa;
  }
  .quality-magic .quality-badge {
    background: rgba(96, 165, 250, 0.25);
    color: #60a5fa;
  }

  .socket-tag {
    color: #38bdf8;
    background: rgba(56, 189, 248, 0.15);
  }

  .eth-tag {
    color: #a78bfa;
    background: rgba(167, 139, 250, 0.15);
  }

  /* 待机状态 */
  .standby-header {
    opacity: 0.7;
  }

  .standby-title {
    font-weight: 600;
    color: #94a3b8;
  }

  .standby-badge {
    font-size: 9px;
    padding: 0 4px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 2px;
    color: #64748b;
  }

  .standby-hint {
    font-size: 10px;
    color: #64748b;
    margin: 2px 0;
  }

  .standby-grid .res-item {
    opacity: 0.4;
    border-color: rgba(255, 255, 255, 0.05);
  }
</style>
