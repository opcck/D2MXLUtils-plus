<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { settingsStore } from '../stores';

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

  let currentPayload = $state<InspectPayload>({ kind: 'None' });
  let isVisible = $state(false);
  let fadeTimeout: number | null = null;

  let monsterInfo = $derived(settingsStore.settings.monsterInfo);
  let itemExtraInfo = $derived(settingsStore.settings.itemExtraInfo);

  onMount(() => {
    let unlisten: (() => void) | null = null;

    listen<InspectPayload>('hover-inspector-update', (event) => {
      const payload = event.payload;

      if (payload.kind === 'Monster') {
        if (!monsterInfo.enabled) {
          hideImmediate();
          return;
        }
        clearFade();
        currentPayload = payload;
        isVisible = true;
      } else if (payload.kind === 'Item') {
        if (!itemExtraInfo.enabled) {
          hideImmediate();
          return;
        }
        clearFade();
        currentPayload = payload;
        isVisible = true;
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

  function hideImmediate() {
    clearFade();
    isVisible = false;
    currentPayload = { kind: 'None' };
  }

  function triggerFade() {
    clearFade();
    fadeTimeout = window.setTimeout(() => {
      isVisible = false;
      currentPayload = { kind: 'None' };
      fadeTimeout = null;
    }, 400);
  }
</script>

{#if isVisible && currentPayload.kind !== 'None'}
  <div class="inspector-hud-container">
    {#if currentPayload.kind === 'Monster'}
      {@const monster = currentPayload.data}
      <div class="inspector-card monster-card">
        <div class="card-header">
          <div class="title-group">
            <span class="entity-name monster-name">{monster.name}</span>
            {#if monsterInfo.showClassId}
              <span class="class-id-badge">ID: {monster.class_id}</span>
            {/if}
          </div>
          <span class="hp-percent-text">{monster.hp_percent}%</span>
        </div>

        <div class="hp-bar-track">
          <div class="hp-bar-fill" style="width: {monster.hp_percent}%"></div>
        </div>

        <div class="resistances-row">
          <!-- 物理抗性 -->
          <div class="res-badge res-phys" class:is-immune={monster.dr >= 100}>
            <span class="res-tag">物</span>
            <span class="res-val">{monster.dr}%</span>
            {#if monster.dr >= 100}<span class="immune-mark">免疫</span>{/if}
          </div>
          <!-- 魔法抗性 -->
          <div class="res-badge res-magic" class:is-immune={monster.mr >= 100}>
            <span class="res-tag">魔</span>
            <span class="res-val">{monster.mr}%</span>
            {#if monster.mr >= 100}<span class="immune-mark">免疫</span>{/if}
          </div>
          <!-- 火焰抗性 -->
          <div class="res-badge res-fire" class:is-immune={monster.fr >= 100}>
            <span class="res-tag">火</span>
            <span class="res-val">{monster.fr}%</span>
            {#if monster.fr >= 100}<span class="immune-mark">免疫</span>{/if}
          </div>
          <!-- 闪电抗性 -->
          <div class="res-badge res-ltng" class:is-immune={monster.lr >= 100}>
            <span class="res-tag">电</span>
            <span class="res-val">{monster.lr}%</span>
            {#if monster.lr >= 100}<span class="immune-mark">免疫</span>{/if}
          </div>
          <!-- 冰霜抗性 -->
          <div class="res-badge res-cold" class:is-immune={monster.cr >= 100}>
            <span class="res-tag">冰</span>
            <span class="res-val">{monster.cr}%</span>
            {#if monster.cr >= 100}<span class="immune-mark">免疫</span>{/if}
          </div>
          <!-- 毒素抗性 -->
          <div class="res-badge res-pois" class:is-immune={monster.pr >= 100}>
            <span class="res-tag">毒</span>
            <span class="res-val">{monster.pr}%</span>
            {#if monster.pr >= 100}<span class="immune-mark">免疫</span>{/if}
          </div>
        </div>
      </div>
    {:else if currentPayload.kind === 'Item'}
      {@const item = currentPayload.data}
      <div class="inspector-card item-card quality-{item.quality.toLowerCase()}">
        <div class="card-header">
          <span class="entity-name item-name">{item.name}</span>
          <span class="quality-badge">{item.quality}</span>
        </div>

        <div class="item-meta-row">
          <span class="meta-item">UID: 0x{item.unit_id.toString(16).toUpperCase()}</span>
          <span class="meta-item">CID: {item.class_id}</span>
          {#if itemExtraInfo.showSocketsAndEth}
            {#if item.sockets > 0}
              <span class="meta-item socket-badge">{item.sockets} 孔 ({item.sockets}s)</span>
            {/if}
            {#if item.is_ethereal}
              <span class="meta-item eth-badge">无形 (Ethereal)</span>
            {/if}
          {/if}
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .inspector-hud-container {
    position: fixed;
    top: 24px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 9999;
    pointer-events: none;
    user-select: none;
    display: flex;
    justify-content: center;
    align-items: center;
    animation: fadeIn 0.15s ease forwards;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translate(-50%, -6px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
  }

  .inspector-card {
    background: rgba(15, 23, 42, 0.88);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    padding: 8px 14px;
    box-shadow:
      0 8px 24px rgba(0, 0, 0, 0.55),
      0 0 0 1px rgba(0, 0, 0, 0.4);
    min-width: 280px;
    max-width: 480px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .entity-name {
    font-size: 13px;
    font-weight: 700;
    color: #f8fafc;
    letter-spacing: 0.3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .class-id-badge {
    font-size: 11px;
    color: #94a3b8;
    background: rgba(255, 255, 255, 0.08);
    padding: 1px 5px;
    border-radius: 4px;
    font-family: monospace;
  }

  .hp-percent-text {
    font-size: 12px;
    font-weight: 700;
    color: #ef4444;
    font-family: monospace;
  }

  .hp-bar-track {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
  }

  .hp-bar-fill {
    height: 100%;
    background: linear-gradient(90deg, #ef4444, #dc2626);
    border-radius: 2px;
    transition: width 0.15s ease;
  }

  .resistances-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 4px;
    margin-top: 2px;
  }

  .res-badge {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 3px 2px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.06);
    transition: all 0.15s ease;
  }

  .res-tag {
    font-size: 10px;
    font-weight: 600;
    opacity: 0.85;
  }

  .res-val {
    font-size: 11px;
    font-weight: 700;
    font-family: monospace;
  }

  .immune-mark {
    font-size: 9px;
    font-weight: 800;
    background: #f59e0b;
    color: #000;
    padding: 0 3px;
    border-radius: 2px;
    margin-top: 1px;
    letter-spacing: -0.5px;
  }

  /* 六系抗性配色 */
  .res-phys {
    border-color: rgba(203, 213, 225, 0.3);
    color: #cbd5e1;
  }
  .res-magic {
    border-color: rgba(244, 114, 182, 0.3);
    color: #f472b6;
  }
  .res-fire {
    border-color: rgba(248, 113, 113, 0.3);
    color: #f87171;
  }
  .res-ltng {
    border-color: rgba(250, 204, 21, 0.3);
    color: #facc15;
  }
  .res-cold {
    border-color: rgba(96, 165, 250, 0.3);
    color: #60a5fa;
  }
  .res-pois {
    border-color: rgba(74, 222, 128, 0.3);
    color: #4ade80;
  }

  .res-badge.is-immune {
    background: rgba(245, 158, 11, 0.15);
    border-color: #f59e0b;
    box-shadow: 0 0 6px rgba(245, 158, 11, 0.35);
  }

  /* 物品卡片样式 */
  .item-card {
    border-left: 3px solid #60a5fa;
  }

  .item-meta-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: #cbd5e1;
    font-family: monospace;
    flex-wrap: wrap;
  }

  .meta-item {
    background: rgba(255, 255, 255, 0.08);
    padding: 1px 6px;
    border-radius: 3px;
  }

  .quality-badge {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 1px 6px;
    border-radius: 3px;
  }

  .quality-unique .item-name {
    color: #c8963e;
  }
  .quality-unique .quality-badge {
    background: rgba(200, 150, 62, 0.2);
    color: #c8963e;
  }

  .quality-set .item-name {
    color: #22c55e;
  }
  .quality-set .quality-badge {
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
  }

  .quality-rare .item-name {
    color: #facc15;
  }
  .quality-rare .quality-badge {
    background: rgba(250, 204, 21, 0.2);
    color: #facc15;
  }

  .quality-magic .item-name {
    color: #60a5fa;
  }
  .quality-magic .quality-badge {
    background: rgba(96, 165, 250, 0.2);
    color: #60a5fa;
  }

  .socket-badge {
    color: #38bdf8;
    background: rgba(56, 189, 248, 0.15);
    font-weight: 600;
  }

  .eth-badge {
    color: #a78bfa;
    background: rgba(167, 139, 250, 0.15);
    font-weight: 600;
  }
</style>
