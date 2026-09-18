<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { settingsStore } from '../stores';
  import { widgetPosition } from '../stores/widget-positions.svelte';

  interface ItemInspectData {
    name: string;
    class_id: number;
    unit_id: number;
    quality: string;
    sockets: number;
    is_ethereal: boolean;
  }

  let pos = $derived(widgetPosition('item-hud'));
  let itemExtraInfo = $derived(settingsStore.settings.itemExtraInfo);

  let item = $state<ItemInspectData | null>(null);
  let fadeTimer: number | null = null;
  let isActive = $derived(item !== null);

  onMount(() => {
    let unlisten: (() => void) | null = null;

    listen<ItemInspectData | null>('item-inspect-update', (event) => {
      if (!itemExtraInfo.enabled) {
        item = null;
        return;
      }
      if (event.payload) {
        if (fadeTimer !== null) {
          clearTimeout(fadeTimer);
          fadeTimer = null;
        }
        item = event.payload;
      } else {
        if (fadeTimer === null && item !== null) {
          fadeTimer = window.setTimeout(() => {
            item = null;
            fadeTimer = null;
          }, 600);
        }
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
      if (fadeTimer !== null) clearTimeout(fadeTimer);
    };
  });
</script>

<div class="item-hud-widget" class:is-active={isActive} style:left="{pos.x}%" style:top="{pos.y}%">
  {#if item}
    <div class="card-content item-active quality-{item.quality.toLowerCase()}">
      <div class="card-header">
        <span class="item-name" title={item.name}>{item.name}</span>
        <span class="quality-badge">{item.quality}</span>
      </div>

      <div class="item-meta">
        <span class="meta-tag">UID: 0x{item.unit_id.toString(16).toUpperCase()}</span>
        <span class="meta-tag">CID: {item.class_id}</span>
        {#if item.sockets > 0}
          <span class="meta-tag socket-tag">{item.sockets}孔</span>
        {/if}
        {#if item.is_ethereal}
          <span class="meta-tag eth-tag">无形</span>
        {/if}
      </div>
    </div>
  {:else}
    <div class="card-content standby-state">
      <div class="card-header standby-header">
        <span class="standby-title">物品详细信息</span>
        <span class="standby-badge">待机中</span>
      </div>
      <div class="standby-hint">悬停地面/背包物品查看</div>
      <div class="item-meta standby-meta">
        <span class="meta-tag">UID: --</span>
        <span class="meta-tag">CID: --</span>
        <span class="meta-tag">品质: --</span>
      </div>
    </div>
  {/if}
</div>

<style>
  .item-hud-widget {
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
    pointer-events: none;
    transition:
      opacity 200ms ease,
      border-color 200ms ease,
      box-shadow 200ms ease;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    z-index: 100;
  }

  .item-hud-widget.is-active {
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

  .item-name {
    font-size: 12px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #f8fafc;
  }

  .quality-badge {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 0 4px;
    border-radius: 2px;
  }

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
    color: #cbd5e1;
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
    font-weight: 600;
  }

  .eth-tag {
    color: #a78bfa;
    background: rgba(167, 139, 250, 0.15);
    font-weight: 600;
  }

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
  .standby-meta .meta-tag {
    opacity: 0.5;
  }
</style>
