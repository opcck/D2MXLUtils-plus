<script lang="ts">
  import { onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { Toggle } from '../components';
  import {
    settingsStore,
    type AutoPotionSettings,
    type AutoPotionSlotConfig,
    type AutoPotionTarget,
  } from '../stores';

  interface PlayerVitals {
    curHp: number;
    maxHp: number;
    hpPercent: number;
    curMana: number;
    maxMana: number;
    manaPercent: number;
  }

  // Reactive settings from store
  let radarEnabled = $derived(settingsStore.settings.radarEnabled);
  let radarShowNormal = $derived(settingsStore.settings.radarShowNormal);
  let continuousAttack = $derived(settingsStore.settings.continuousAttack);
  let autoBelt = $derived(settingsStore.settings.autoBelt);
  let removeShadows = $derived(settingsStore.settings.removeShadows);
  let autoPotion = $derived(settingsStore.settings.autoPotion);

  // Live vitals from backend
  let vitals = $state<PlayerVitals | null>(null);

  // Key recording state for slots
  let recordingSlotIndex = $state<number | null>(null);

  let unlistenVitals: UnlistenFn | null = null;

  onMount(() => {
    listen<PlayerVitals>('player-vitals-update', (event) => {
      vitals = event.payload;
    }).then((unlisten) => {
      unlistenVitals = unlisten;
    });

    const handleKeyDown = (e: KeyboardEvent) => {
      if (recordingSlotIndex === null) return;
      e.preventDefault();
      e.stopPropagation();

      // Escape cancels recording
      if (e.key === 'Escape') {
        recordingSlotIndex = null;
        return;
      }

      // Format display key
      let keyDisplay = e.key.toUpperCase();
      if (e.code.startsWith('Digit')) {
        keyDisplay = e.code.replace('Digit', '');
      } else if (e.code.startsWith('Key')) {
        keyDisplay = e.code.replace('Key', '');
      } else if (e.code.startsWith('Numpad')) {
        keyDisplay = 'Num ' + e.code.replace('Numpad', '');
      }

      updateSlot(recordingSlotIndex, {
        keyCode: e.keyCode,
        keyDisplay,
      });

      recordingSlotIndex = null;
    };

    window.addEventListener('keydown', handleKeyDown, true);

    return () => {
      if (unlistenVitals) unlistenVitals();
      window.removeEventListener('keydown', handleKeyDown, true);
    };
  });

  // Tweak handlers
  function handleRadarEnabledChange(enabled: boolean) {
    settingsStore.setRadarEnabled(enabled);
  }

  function handleRadarShowNormalChange(enabled: boolean) {
    settingsStore.setRadarShowNormal(enabled);
  }

  function handleContinuousAttackChange(enabled: boolean) {
    settingsStore.setContinuousAttack(enabled);
  }

  function handleAutoBeltChange(enabled: boolean) {
    settingsStore.setAutoBelt(enabled);
  }

  function handleRemoveShadowsChange(enabled: boolean) {
    settingsStore.setRemoveShadows(enabled);
  }

  // Auto potion handlers
  function handleAutoPotionMasterToggle(enabled: boolean) {
    const updated: AutoPotionSettings = {
      ...autoPotion,
      enabled,
    };
    settingsStore.setAutoPotionSettings(updated);
  }

  function updateSlot(index: number, changes: Partial<AutoPotionSlotConfig>) {
    const slots = autoPotion.slots.map((s, i) => (i === index ? { ...s, ...changes } : s));
    settingsStore.setAutoPotionSettings({
      ...autoPotion,
      slots,
    });
  }

  function startRecordKey(index: number) {
    recordingSlotIndex = index;
  }
</script>

<div class="assist-tab">
  <!-- 实时角色血蓝监测面板 -->
  <div class="settings-section vitals-section">
    <div class="vitals-header">
      <h2 class="section-title">实时角色状态 (Live Vitals)</h2>
      {#if vitals && vitals.maxHp > 0}
        <span class="vitals-badge live">● 游戏中实时监测中</span>
      {:else}
        <span class="vitals-badge idle">○ 等待进入游戏…</span>
      {/if}
    </div>

    <div class="vitals-grid">
      <div class="vital-card hp-card">
        <div class="vital-label">
          <span>生命值 (HP)</span>
          {#if vitals && vitals.maxHp > 0}
            <span class="vital-val">{vitals.curHp} / {vitals.maxHp} ({vitals.hpPercent}%)</span>
          {:else}
            <span class="vital-val">-- / --</span>
          {/if}
        </div>
        <div class="vital-bar-track">
          <div
            class="vital-bar-fill hp-fill"
            style="width: {vitals && vitals.maxHp > 0 ? vitals.hpPercent : 0}%"
          ></div>
        </div>
      </div>

      <div class="vital-card mana-card">
        <div class="vital-label">
          <span>法力值 (Mana)</span>
          {#if vitals && vitals.maxMana > 0}
            <span class="vital-val"
              >{vitals.curMana} / {vitals.maxMana} ({vitals.manaPercent}%)</span
            >
          {:else}
            <span class="vital-val">-- / --</span>
          {/if}
        </div>
        <div class="vital-bar-track">
          <div
            class="vital-bar-fill mana-fill"
            style="width: {vitals && vitals.maxMana > 0 ? vitals.manaPercent : 0}%"
          ></div>
        </div>
      </div>
    </div>
  </div>

  <!-- 自动喝药设置卡片 (3 栏位) -->
  <div class="settings-section">
    <div class="section-header-row">
      <div>
        <h2 class="section-title">自动喝药配置 (Auto Potion)</h2>
        <span class="setting-hint"
          >读取游戏内实时生命与法力值，当指标低于设定百分比时向游戏窗口发送喝药按键。</span
        >
      </div>
      <Toggle checked={autoPotion.enabled} onchange={handleAutoPotionMasterToggle} />
    </div>

    <div class="slots-container {autoPotion.enabled ? '' : 'is-disabled'}">
      {#each autoPotion.slots as slot, idx}
        <div class="slot-card {slot.enabled ? 'is-active' : ''}">
          <div class="slot-header">
            <div class="slot-title">
              <span class="slot-badge">栏位 {idx + 1}</span>
              <span class="slot-name">
                {slot.target === 'hp' ? '生命值监测' : '法力值监测'}
              </span>
            </div>
            <Toggle checked={slot.enabled} onchange={(enabled) => updateSlot(idx, { enabled })} />
          </div>

          <div class="slot-body">
            <!-- 监控类型选择 -->
            <div class="control-group">
              <label for="slot-target-{idx}" class="field-label">监测类型</label>
              <select
                id="slot-target-{idx}"
                class="select-input"
                value={slot.target}
                onchange={(e) =>
                  updateSlot(idx, {
                    target: (e.target as HTMLSelectElement).value as AutoPotionTarget,
                  })}
              >
                <option value="hp">生命值 (HP)</option>
                <option value="mana">法力值 (Mana)</option>
              </select>
            </div>

            <!-- 触发百分比滑块 -->
            <div class="control-group threshold-group">
              <label for="slot-threshold-{idx}" class="field-label">
                触发阈值：低于 <span class="highlight-val">{slot.thresholdPercent}%</span> 时喝药
              </label>
              <div class="slider-row">
                <input
                  id="slot-threshold-{idx}"
                  type="range"
                  min="5"
                  max="95"
                  step="1"
                  value={slot.thresholdPercent}
                  oninput={(e) =>
                    updateSlot(idx, { thresholdPercent: parseInt(e.currentTarget.value) })}
                  class="slider"
                />
              </div>
            </div>

            <!-- 自定义喝药按键 -->
            <div class="control-group">
              <label for="slot-key-btn-{idx}" class="field-label">喝药按键</label>
              <button
                id="slot-key-btn-{idx}"
                type="button"
                class="key-button {recordingSlotIndex === idx ? 'is-recording' : ''}"
                onclick={() => startRecordKey(idx)}
              >
                {recordingSlotIndex === idx ? '按任意键绑定…' : slot.keyDisplay}
              </button>
            </div>

            <!-- 防连按冷却 (ms) -->
            <div class="control-group cd-group">
              <label for="slot-cd-{idx}" class="field-label">冷却 CD</label>
              <div class="cd-input-wrap">
                <input
                  id="slot-cd-{idx}"
                  type="number"
                  min="100"
                  max="5000"
                  step="50"
                  class="num-input"
                  value={slot.cooldownMs}
                  oninput={(e) =>
                    updateSlot(idx, {
                      cooldownMs: Math.max(100, parseInt(e.currentTarget.value) || 600),
                    })}
                />
                <span class="unit-text">ms</span>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>

  <!-- 游戏增强辅助 (已迁移整理) -->
  <div class="settings-section">
    <h2 class="section-title">游戏增强辅助 (QoL Tweaks)</h2>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">小地图怪物雷达 (Monster Radar)</span>
        <span class="setting-hint"
          >在小地图/全屏地图上高亮标记怪物：暗金Boss显示为金色、精英怪显示为紫色、普通怪显示为红点。</span
        >
      </div>
      <Toggle checked={radarEnabled} onchange={handleRadarEnabledChange} />
    </div>

    {#if radarEnabled}
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">雷达显示普通怪小红点</span>
          <span class="setting-hint"
            >关闭后雷达只标记金色暗金怪/Boss和紫色精英怪，保持小地图清爽。</span
          >
        </div>
        <Toggle checked={radarShowNormal} onchange={handleRadarShowNormalChange} />
      </div>
    {/if}

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">持续攻击不打断 (Continuous Attack)</span>
        <span class="setting-hint"
          >按住鼠标攻击或施法时，当前目标怪物死亡后不中断动作，继续朝光标方向连击。</span
        >
      </div>
      <Toggle checked={continuousAttack} onchange={handleContinuousAttackChange} />
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">药水自动补充入腰带 (Auto Potion Belt)</span>
        <span class="setting-hint">腰带存在空位时，自动从背包将药水补充入腰带。</span>
      </div>
      <Toggle checked={autoBelt} onchange={handleAutoBeltChange} />
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">去除场景与单位阴影 (Remove Shadows)</span>
        <span class="setting-hint"
          >关闭游戏内的环境与单位阴影渲染，提升同屏多怪时的帧率与画面清晰度。</span
        >
      </div>
      <Toggle checked={removeShadows} onchange={handleRemoveShadowsChange} />
    </div>
  </div>
</div>

<style>
  .assist-tab {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  /* 实时血蓝条 */
  .vitals-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-3);
  }

  .vitals-badge {
    font-size: var(--text-xs);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-weight: 500;
  }

  .vitals-badge.live {
    background: rgba(46, 204, 113, 0.15);
    color: var(--status-success-text, #2ecc71);
  }

  .vitals-badge.idle {
    background: var(--bg-tertiary);
    color: var(--text-muted);
  }

  .vitals-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-3);
  }

  .vital-card {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .vital-label {
    display: flex;
    justify-content: space-between;
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .vital-val {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--text-primary);
  }

  .vital-bar-track {
    width: 100%;
    height: 8px;
    background: var(--bg-primary);
    border-radius: var(--radius-full);
    overflow: hidden;
  }

  .vital-bar-fill {
    height: 100%;
    border-radius: var(--radius-full);
    transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .hp-fill {
    background: linear-gradient(90deg, #e74c3c, #ff7675);
  }

  .mana-fill {
    background: linear-gradient(90deg, #2980b9, #3498db);
  }

  /* 栏位配置 */
  .section-header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--space-3);
  }

  .slots-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    transition: opacity 0.2s ease;
  }

  .slots-container.is-disabled {
    opacity: 0.45;
    pointer-events: none;
  }

  .slot-card {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    transition: border-color 0.2s ease;
  }

  .slot-card.is-active {
    border-color: rgba(var(--accent-primary-rgb, 108, 92, 231), 0.4);
  }

  .slot-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    padding-bottom: var(--space-2);
  }

  .slot-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .slot-badge {
    background: var(--accent-primary);
    color: white;
    font-size: var(--text-xs);
    font-weight: 600;
    padding: 1px 6px;
    border-radius: var(--radius-xs, 3px);
  }

  .slot-name {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-primary);
  }

  .slot-body {
    display: grid;
    grid-template-columns: 140px 1fr 120px 100px;
    gap: var(--space-3);
    align-items: center;
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .field-label {
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .highlight-val {
    color: var(--accent-primary);
    font-weight: 600;
    font-family: var(--font-mono);
  }

  .select-input {
    background: var(--bg-primary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    padding: var(--space-1) var(--space-2);
    font-size: var(--text-sm);
    outline: none;
  }

  .slider-row {
    display: flex;
    align-items: center;
  }

  .slider {
    width: 100%;
    height: 6px;
    appearance: none;
    background: var(--bg-primary);
    border-radius: var(--radius-full);
    cursor: pointer;
  }

  .slider::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    background: var(--accent-primary);
    border-radius: var(--radius-full);
    cursor: pointer;
  }

  .key-button {
    background: var(--bg-primary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 600;
    padding: var(--space-1) var(--space-2);
    cursor: pointer;
    text-align: center;
    transition: all 0.15s ease;
  }

  .key-button:hover {
    border-color: var(--accent-primary);
  }

  .key-button.is-recording {
    background: rgba(231, 76, 60, 0.2);
    border-color: #e74c3c;
    color: #ff7675;
    animation: pulse 1.2s infinite;
  }

  .cd-input-wrap {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .num-input {
    width: 65px;
    background: var(--bg-primary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    padding: var(--space-1);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    text-align: right;
  }

  .unit-text {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }
</style>
