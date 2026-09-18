<script lang="ts">
  import { onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { Toggle } from '../components';
  import {
    settingsStore,
    type AutoPotionSettings,
    type AutoPotionSlotConfig,
    type AutoPotionTarget,
    type AutoPickupSettings,
    type MonsterInfoSettings,
    type ItemExtraInfoSettings,
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
  let autoPickup = $derived(settingsStore.settings.autoPickup);
  let monsterInfo = $derived(settingsStore.settings.monsterInfo);
  let itemExtraInfo = $derived(settingsStore.settings.itemExtraInfo);

  // Live vitals from backend
  let vitals = $state<PlayerVitals | null>(null);

  // Key recording state for slots
  let recordingSlotIndex = $state<number | null>(null);

  // Auto pickup local state
  let pickupRulesText = $state(settingsStore.settings.autoPickup.rulesText || '');
  let isSavingRules = $state(false);
  let rulesSaveSuccess = $state(false);
  let recordingPickupHotkey = $state(false);

  // Monster info and Item extra info recording state
  let recordingMonsterHotkey = $state(false);
  let recordingItemHotkey = $state(false);

  // Sync initial loaded rulesText
  $effect(() => {
    if (settingsStore.isLoaded && !pickupRulesText && settingsStore.settings.autoPickup.rulesText) {
      pickupRulesText = settingsStore.settings.autoPickup.rulesText;
    }
  });

  let unlistenVitals: UnlistenFn | null = null;

  onMount(() => {
    listen<PlayerVitals>('player-vitals-update', (event) => {
      vitals = event.payload;
    }).then((unlisten) => {
      unlistenVitals = unlisten;
    });

    const handleKeyDown = (e: KeyboardEvent) => {
      if (recordingMonsterHotkey) {
        e.preventDefault();
        e.stopPropagation();

        if (e.key === 'Escape') {
          recordingMonsterHotkey = false;
          return;
        }

        let modifiers = 0;
        if (e.altKey) modifiers |= 0x0001;
        if (e.ctrlKey) modifiers |= 0x0002;
        if (e.shiftKey) modifiers |= 0x0004;

        let keyDisplay = '';
        if (e.ctrlKey) keyDisplay += 'Ctrl+';
        if (e.altKey) keyDisplay += 'Alt+';
        if (e.shiftKey) keyDisplay += 'Shift+';

        let name = e.key;
        if (e.code === 'BracketLeft') name = '[';
        else if (e.code === 'BracketRight') name = ']';
        else if (e.code.startsWith('Key')) name = e.code.replace('Key', '');
        else if (e.code.startsWith('Digit')) name = e.code.replace('Digit', '');
        else if (e.code === 'PageUp') name = 'PageUp';
        else if (e.code === 'PageDown') name = 'PageDown';
        else if (e.key === ' ') name = 'Space';
        keyDisplay += name;

        settingsStore.setMonsterInfoSettings({
          ...monsterInfo,
          hotkey: {
            keyCode: e.keyCode,
            modifiers,
            display: keyDisplay,
          },
        });

        recordingMonsterHotkey = false;
        return;
      }

      if (recordingItemHotkey) {
        e.preventDefault();
        e.stopPropagation();

        if (e.key === 'Escape') {
          recordingItemHotkey = false;
          return;
        }

        let modifiers = 0;
        if (e.altKey) modifiers |= 0x0001;
        if (e.ctrlKey) modifiers |= 0x0002;
        if (e.shiftKey) modifiers |= 0x0004;

        let keyDisplay = '';
        if (e.ctrlKey) keyDisplay += 'Ctrl+';
        if (e.altKey) keyDisplay += 'Alt+';
        if (e.shiftKey) keyDisplay += 'Shift+';

        let name = e.key;
        if (e.code === 'BracketLeft') name = '[';
        else if (e.code === 'BracketRight') name = ']';
        else if (e.code.startsWith('Key')) name = e.code.replace('Key', '');
        else if (e.code.startsWith('Digit')) name = e.code.replace('Digit', '');
        else if (e.code === 'PageUp') name = 'PageUp';
        else if (e.code === 'PageDown') name = 'PageDown';
        else if (e.key === ' ') name = 'Space';
        keyDisplay += name;

        settingsStore.setItemExtraInfoSettings({
          ...itemExtraInfo,
          hotkey: {
            keyCode: e.keyCode,
            modifiers,
            display: keyDisplay,
          },
        });

        recordingItemHotkey = false;
        return;
      }

      if (recordingPickupHotkey) {
        e.preventDefault();
        e.stopPropagation();

        if (e.key === 'Escape') {
          recordingPickupHotkey = false;
          return;
        }

        let modifiers = 0;
        if (e.altKey) modifiers |= 0x0001;
        if (e.ctrlKey) modifiers |= 0x0002;
        if (e.shiftKey) modifiers |= 0x0004;

        let keyDisplay = '';
        if (e.ctrlKey) keyDisplay += 'Ctrl+';
        if (e.altKey) keyDisplay += 'Alt+';
        if (e.shiftKey) keyDisplay += 'Shift+';

        let name = e.key;
        if (e.code === 'BracketLeft') name = '[';
        else if (e.code === 'BracketRight') name = ']';
        else if (e.code.startsWith('Key')) name = e.code.replace('Key', '');
        else if (e.code.startsWith('Digit')) name = e.code.replace('Digit', '');
        else if (e.code === 'PageUp') name = 'PageUp';
        else if (e.code === 'PageDown') name = 'PageDown';
        else if (e.key === ' ') name = 'Space';
        keyDisplay += name;

        settingsStore.setAutoPickupSettings({
          ...autoPickup,
          hotkey: {
            keyCode: e.keyCode,
            modifiers,
            display: keyDisplay,
          },
        });

        recordingPickupHotkey = false;
        return;
      }

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

  // Auto pickup handlers
  function handleAutoPickupMasterToggle(enabled: boolean) {
    settingsStore.setAutoPickupEnabled(enabled);
  }

  function startRecordPickupHotkey() {
    recordingPickupHotkey = true;
  }

  function handleClearPickupHotkey() {
    settingsStore.setAutoPickupSettings({
      ...autoPickup,
      hotkey: null,
    });
  }

  function handleDistanceChange(val: number) {
    settingsStore.setAutoPickupSettings({
      ...autoPickup,
      pickupDistance: val,
    });
  }

  async function handleSaveRules() {
    isSavingRules = true;
    try {
      await settingsStore.setAutoPickupSettings({
        ...autoPickup,
        rulesText: pickupRulesText,
      });
      rulesSaveSuccess = true;
      setTimeout(() => {
        rulesSaveSuccess = false;
      }, 2000);
    } catch (e) {
      console.error('Failed to save pickup rules:', e);
    } finally {
      isSavingRules = false;
    }
  }

  async function handleResetRules() {
    if (confirm('确定要重置自动拾取规则为默认模板吗？当前自定义编辑的内容将被覆盖。')) {
      const defaultRules = `# ==============================================================================
# D2MXLUtils-plus 自动拾取规则模板 (基于 HackMap 规则清洗移植)
# 规则说明：
# 1. 匹配机制：自底向上（倒序遍历匹配，写在越靠后的规则优先级越高）。
# 2. 拾取行为 (pickup)：
#    pickup = 0 -> 不捡（用于特定高价值物品防误捡保护，需手动拾取）
#    pickup = 1 -> 直接入背包 (Inventory)
#    pickup = 2 -> 优先放入赫拉迪克方块 (Cube，方块满自动降级入包)
#    pickup = 3 -> 自动补入腰带药水栏 (AutoBelt，腰带满静默跳过)
# 3. 匹配属性：
#    match_name = 1 -> 匹配物品名称 (prop/regex)
#    match_name = 0 -> 匹配装备词条属性 (regex)
#    quality -> 品质 (normal / superior / magic / rare / set / unique)
#    id -> 物品类型 Class ID
#    socks -> 孔数，eth -> 无形
# ==============================================================================

# --- 宝石与通用圣坛 ---
{ match_name = 1, prop = "完美的", pickup = 0 },
{ match_name = 1, prop = "圣坛", pickup = 1 },

# --- 绿色套装与暗金装备 ---
{ quality = "set", pickup = 2 },
{ quality = "unique", pickup = 2 },

# --- 任务道具与特定符文 ---
{ match_name = 1, prop = "符文", pickup = 1 },
# 特殊/高价值大号符文防误捡（保护设为 0，需手动拾取）
{ match_name = 1, prop = "查姆", pickup = 0 },
{ match_name = 1, prop = "佐德", pickup = 0 },
{ match_name = 1, prop = "海", pickup = 0 },
{ match_name = 1, prop = "贝", pickup = 0 },
{ match_name = 1, prop = "乔", pickup = 0 },

# --- Median XL 特色材料与消耗品 ---
{ match_name = 1, prop = "奥能之角", pickup = 2 },
{ match_name = 1, prop = "神秘碎片", pickup = 2 },
{ match_name = 1, prop = "神秘晶体", pickup = 2 },
{ match_name = 1, prop = "催化剂", pickup = 2 },
{ match_name = 1, prop = "光辉", pickup = 2 },
{ match_name = 1, prop = "魔印", pickup = 2 },
{ match_name = 1, prop = "印记", pickup = 2 },
{ match_name = 1, prop = "徽记", pickup = 2 },
{ match_name = 1, prop = "奇迹", pickup = 2 },
{ match_name = 1, prop = "遗物", pickup = 2 },
{ match_name = 1, prop = "钟楼", pickup = 2 },
{ match_name = 1, prop = "符文之石", pickup = 2 },
{ match_name = 1, prop = "经验之书", pickup = 2 },
{ match_name = 1, prop = "古代之书", pickup = 2 },
{ match_name = 1, prop = "遗忘之塔", pickup = 2 },
{ match_name = 1, prop = "恶魔之角", pickup = 2 },
{ match_name = 1, prop = "世界之石碎片", pickup = 2 },
{ match_name = 1, prop = "天使之羽", pickup = 2 },
{ match_name = 1, prop = "黑塔之钥", pickup = 2 },
{ match_name = 1, prop = "虚空之石", pickup = 2 },
{ match_name = 1, prop = "纳塔尔之眼", pickup = 2 },
{ match_name = 1, prop = "恶魔之心", pickup = 2 },
{ match_name = 1, prop = "恶魔之血", pickup = 2 },
{ match_name = 1, prop = "恶魔之眼", pickup = 2 },
{ match_name = 1, prop = "恶魔之脑", pickup = 2 },
{ match_name = 1, prop = "恶魔之爪", pickup = 2 },
{ match_name = 1, prop = "恶魔之翼", pickup = 2 },
{ match_name = 1, prop = "恶魔之牙", pickup = 2 },
{ match_name = 1, prop = "恶魔之尾", pickup = 2 },
{ match_name = 1, prop = "恶魔之角", pickup = 2 },
{ match_name = 1, prop = "恶魔之骨", pickup = 2 },
{ match_name = 1, prop = "恶魔之皮", pickup = 2 },
{ match_name = 1, prop = "恶魔之肉", pickup = 2 },
{ match_name = 1, prop = "恶魔之魂", pickup = 2 },
{ match_name = 1, prop = "恶魔之影", pickup = 2 },
{ match_name = 1, prop = "恶魔之核", pickup = 2 },
{ match_name = 1, prop = "恶魔之精", pickup = 2 },
{ match_name = 1, prop = "恶魔之息", pickup = 2 },
{ match_name = 1, prop = "恶魔之源", pickup = 2 },
{ match_name = 1, prop = "恶魔之血", pickup = 2 },
{ match_name = 1, prop = "恶魔之灵", pickup = 2 },

# --- 常用药水与消耗品 ---
# 恢复活力药水
{ match_name = 1, prop = "恢复活力药水", pickup = 2 },
# 生命/法力药水优先自动补入腰带 (AutoBelt)
{ match_name = 1, prop = "治疗药水", pickup = 3 },
{ match_name = 1, prop = "法力药水", pickup = 3 },
{ match_name = 1, prop = "超强治疗药水", pickup = 3 },
{ match_name = 1, prop = "超强法力药水", pickup = 3 },

# --- 卷轴与金币 ---
{ match_name = 1, prop = "辨识卷轴", pickup = 1 },
{ match_name = 1, prop = "城镇传送卷轴", pickup = 1 },
{ match_name = 1, prop = "金币", pickup = 1 },

# --- 稀有专属草药与材料 ---
{ match_name = 1, prop = "Belladonna Extract", pickup = 2 },
{ match_name = 1, prop = "Sunless Crystal", pickup = 2 },
{ match_name = 1, prop = "Arcane Shard", pickup = 2 },
{ match_name = 1, prop = "Arcane Crystal", pickup = 2 },
{ match_name = 1, prop = "Oil of", pickup = 2 },
{ match_name = 1, prop = "Signet of", pickup = 2 },
{ match_name = 1, prop = "Apple", pickup = 2 },
{ match_name = 1, prop = "Trance Herb", pickup = 2 },
{ match_name = 1, prop = "骨堆", pickup = 2 },
`;
      pickupRulesText = defaultRules;
      await handleSaveRules();
    }
  }

  // Monster Info Handlers
  function handleMonsterInfoMasterToggle(enabled: boolean) {
    settingsStore.setMonsterInfoEnabled(enabled);
  }

  function handleMonsterInfoShowClassIdToggle(showClassId: boolean) {
    settingsStore.setMonsterInfoSettings({
      ...monsterInfo,
      showClassId,
    });
  }

  function startRecordMonsterHotkey() {
    recordingMonsterHotkey = true;
    recordingItemHotkey = false;
    recordingPickupHotkey = false;
    recordingSlotIndex = null;
  }

  function handleClearMonsterHotkey() {
    settingsStore.setMonsterInfoSettings({
      ...monsterInfo,
      hotkey: null,
    });
  }

  // Item Extra Info Handlers
  function handleItemExtraInfoMasterToggle(enabled: boolean) {
    settingsStore.setItemExtraInfoEnabled(enabled);
  }

  function handleItemExtraInfoShowSocketsAndEthToggle(showSocketsAndEth: boolean) {
    settingsStore.setItemExtraInfoSettings({
      ...itemExtraInfo,
      showSocketsAndEth,
    });
  }

  function startRecordItemHotkey() {
    recordingItemHotkey = true;
    recordingMonsterHotkey = false;
    recordingPickupHotkey = false;
    recordingSlotIndex = null;
  }

  function handleClearItemHotkey() {
    settingsStore.setItemExtraInfoSettings({
      ...itemExtraInfo,
      hotkey: null,
    });
  }
</script>

<section class="tab-content assist-tab">
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

  <!-- 自动拾取配置卡片 (Auto Pickup) -->
  <div class="settings-section">
    <div class="section-header-row">
      <div>
        <h2 class="section-title">自动拾取配置 (Auto Pickup)</h2>
      </div>
      <Toggle checked={autoPickup.enabled} onchange={handleAutoPickupMasterToggle} />
    </div>

    <div class="pickup-container {autoPickup.enabled ? '' : 'is-disabled'}">
      <!-- 快捷键与距离调节控制行 -->
      <div class="pickup-controls-grid">
        <div class="control-item">
          <span class="control-label">游戏内实时开关快捷键</span>
          <div class="hotkey-wrapper">
            <button
              type="button"
              class="hotkey-btn {recordingPickupHotkey ? 'recording' : ''}"
              onclick={startRecordPickupHotkey}
            >
              {recordingPickupHotkey
                ? '按下按键… (Esc取消)'
                : autoPickup.hotkey?.display || '未设置'}
            </button>
            {#if autoPickup.hotkey}
              <button
                type="button"
                class="clear-btn"
                title="清除快捷键"
                onclick={handleClearPickupHotkey}
              >
                ✕
              </button>
            {/if}
          </div>
          <span class="sub-hint">游戏内按此键可无缝切换开启/关闭状态</span>
        </div>

        <div class="control-item">
          <div class="distance-header">
            <span class="control-label">拾取距离限制</span>
            <span class="distance-val">{autoPickup.pickupDistance.toFixed(1)} 码 (Yards)</span>
          </div>
          <input
            type="range"
            min="1.0"
            max="15.0"
            step="0.5"
            class="slider"
            value={autoPickup.pickupDistance}
            oninput={(e) => handleDistanceChange(parseFloat(e.currentTarget.value) || 5.0)}
          />
          <span class="sub-hint">建议 3.0 ~ 8.0 码（1码约为人物一步距离）</span>
        </div>
      </div>

      <!-- 规则编辑区域 -->
      <div class="rules-editor-wrapper">
        <div class="rules-editor-header">
          <div class="rules-title-group">
            <span class="rules-title">拾取规则过滤表 (TOML 语法)</span>
            <span class="rules-badge">自底向上优先匹配 (Last match wins)</span>
          </div>
          <div class="rules-actions">
            <button
              type="button"
              class="btn-reset"
              onclick={handleResetRules}
              title="恢复为内置默认规则模板"
            >
              重置为默认规则
            </button>
            <button
              type="button"
              class="btn-save {rulesSaveSuccess ? 'save-success' : ''}"
              onclick={handleSaveRules}
              disabled={isSavingRules}
            >
              {rulesSaveSuccess ? '✓ 已保存并生效' : isSavingRules ? '保存中…' : '保存并应用规则'}
            </button>
          </div>
        </div>

        <textarea
          class="rules-textarea"
          rows="14"
          spellcheck="false"
          bind:value={pickupRulesText}
          placeholder="在此编写自定义自动拾取规则..."
        ></textarea>

        <!-- 语法参考帮助 (折叠面板) -->
        <details class="rules-help-details">
          <summary class="rules-help-summary">
            <span>📖 点击展开：自动拾取规则语法与属性说明</span>
          </summary>
          <div class="rules-help-content">
            <div class="help-item">
              <strong class="help-title">拾取行为 (pickup)：</strong>
              <ul class="help-list">
                <li>
                  <code>pickup = 0</code>：<strong>不捡 / 黑名单</strong
                  >（如特定高价值大号符文防误捡保护，需手动拾取）
                </li>
                <li>
                  <code>pickup = 1</code>：<strong>直接入背包 (Inventory)</strong
                  >（背包放不下静默跳过）
                </li>
                <li>
                  <code>pickup = 2</code>：<strong>优先放入赫拉迪克方块 (Cube)</strong
                  >（方块满或未携带方块时自动降级入包）
                </li>
                <li>
                  <code>pickup = 3</code>：<strong>自动补入腰带药水栏 (AutoBelt)</strong
                  >（腰带存在空槽才捡，满后静默跳过）
                </li>
              </ul>
            </div>
            <div class="help-item">
              <strong class="help-title">常用匹配字段：</strong>
              <ul class="help-list">
                <li>
                  <code>match_name = 1, prop = "关键词"</code>：匹配物品名称或包含的字词（如
                  <code>"完美的"</code>、<code>"符文"</code>、<code>"神秘晶体"</code>）
                </li>
                <li><code>match_name = 0, prop = "词条正则"</code>：匹配装备掉落词条属性</li>
                <li>
                  <code>quality = "..."</code>：物品品质（可选 <code>normal</code>,
                  <code>superior</code>, <code>magic</code>, <code>rare</code>, <code>set</code>,
                  <code>unique</code>）
                </li>
                <li><code>id = 603</code>：匹配具体的物品 Class ID（数字）</li>
                <li><code>socks = 4</code>：匹配开孔数量</li>
                <li><code>eth = true / false</code>：匹配是否为无形装备</li>
              </ul>
            </div>
            <div class="help-item">
              <strong class="help-title">匹配机制：</strong>
              <span class="help-desc"
                >规则采用从后往前（自底向上）倒序匹配，最后一条命中的规则决定最终行为。</span
              >
            </div>
          </div>
        </details>
      </div>
    </div>
  </div>

  <!-- 怪物信息与抗性免疫显示 (Monster Info & Resistances) -->
  <div class="settings-section">
    <div class="section-header-row">
      <div>
        <h2 class="section-title">怪物信息与抗性免疫显示 (Monster Info & Resistances)</h2>
      </div>
      <Toggle checked={monsterInfo.enabled} onchange={handleMonsterInfoMasterToggle} />
    </div>

    <div class="feature-container {monsterInfo.enabled ? '' : 'is-disabled'}">
      <div class="feature-controls-grid">
        <div class="control-item">
          <span class="control-label">游戏内实时开关快捷键</span>
          <div class="hotkey-wrapper">
            <button
              type="button"
              class="hotkey-btn {recordingMonsterHotkey ? 'recording' : ''}"
              onclick={startRecordMonsterHotkey}
            >
              {recordingMonsterHotkey
                ? '按下按键… (Esc取消)'
                : monsterInfo.hotkey?.display || '未设置'}
            </button>
            {#if monsterInfo.hotkey}
              <button
                type="button"
                class="clear-btn"
                title="清除快捷键"
                onclick={handleClearMonsterHotkey}
              >
                ✕
              </button>
            {/if}
          </div>
          <span class="sub-hint"
            >游戏内按此键可无缝切换开启/关闭状态（默认快捷键为 <code>]</code>）</span
          >
        </div>

        <div class="control-item">
          <span class="control-label">显示怪物类别 ID (Class ID)</span>
          <div class="sub-toggle-wrapper">
            <Toggle
              checked={monsterInfo.showClassId}
              onchange={handleMonsterInfoShowClassIdToggle}
            />
            <span class="sub-toggle-text"
              >开启后在怪物名称旁显示数字编号，方便对照暗黑2/MXL怪物数据</span
            >
          </div>
        </div>
      </div>

      <div class="info-banner">
        <span class="info-icon">💡</span>
        <div class="info-content">
          <strong>悬停显示抗性与免疫说明：</strong>
          鼠标悬停在怪物顶部血条时，自动提取六系抗性：
          <span class="res-tag res-phys">物 (Phys)</span>
          <span class="res-tag res-magic">魔 (Magic)</span>
          <span class="res-tag res-fire">火 (Fire)</span>
          <span class="res-tag res-ltng">电 (Ltng)</span>
          <span class="res-tag res-cold">冰 (Cold)</span>
          <span class="res-tag res-pois">毒 (Pois)</span>。 抗性达到或超过 100%
          时自动以特殊高亮彩色标明“免疫”，全分辨率/宽屏无缝适配。
        </div>
      </div>
    </div>
  </div>

  <!-- 物品额外信息显示 (Item Extra Info) -->
  <div class="settings-section">
    <div class="section-header-row">
      <div>
        <h2 class="section-title">物品额外信息显示 (Item Extra Info)</h2>
      </div>
      <Toggle checked={itemExtraInfo.enabled} onchange={handleItemExtraInfoMasterToggle} />
    </div>

    <div class="feature-container {itemExtraInfo.enabled ? '' : 'is-disabled'}">
      <div class="feature-controls-grid">
        <div class="control-item">
          <span class="control-label">游戏内实时开关快捷键</span>
          <div class="hotkey-wrapper">
            <button
              type="button"
              class="hotkey-btn {recordingItemHotkey ? 'recording' : ''}"
              onclick={startRecordItemHotkey}
            >
              {recordingItemHotkey
                ? '按下按键… (Esc取消)'
                : itemExtraInfo.hotkey?.display || '未设置'}
            </button>
            {#if itemExtraInfo.hotkey}
              <button
                type="button"
                class="clear-btn"
                title="清除快捷键"
                onclick={handleClearItemHotkey}
              >
                ✕
              </button>
            {/if}
          </div>
          <span class="sub-hint"
            >游戏内按此键可无缝切换开启/关闭状态（默认快捷键为 <code>[</code>）</span
          >
        </div>

        <div class="control-item">
          <span class="control-label">名称后标注孔数与无形状态</span>
          <div class="sub-toggle-wrapper">
            <Toggle
              checked={itemExtraInfo.showSocketsAndEth}
              onchange={handleItemExtraInfoShowSocketsAndEthToggle}
            />
            <span class="sub-toggle-text"
              >开启后在物品名后自动追加 <code>(4s)</code> 孔数与 <code>(eth)</code> 无形标记</span
            >
          </div>
        </div>
      </div>

      <div class="info-banner">
        <span class="info-icon">🔍</span>
        <div class="info-content">
          <strong>物品 ID 显示说明：</strong>
          在地面的物品名牌与鼠标悬停物品 Tooltip 上，首行前置显示物品唯一识别码
          <code>UID:0x...</code> 与类别编号
          <code>CID:...</code>，方便确认物品底层代码与精准编写拾取/过滤规则。
        </div>
      </div>
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
</section>

<style>
  .assist-tab {
    display: flex;
    flex-direction: column;
    padding-right: var(--space-1);
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

  /* 自动拾取面板样式 */
  .pickup-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    margin-top: var(--space-2);
    transition: opacity 0.2s ease;
  }

  .pickup-container.is-disabled {
    opacity: 0.45;
    pointer-events: none;
  }

  .pickup-controls-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: var(--space-3);
  }

  .control-item {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .control-label {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-secondary);
  }

  .distance-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .distance-val {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--accent-primary, #60a5fa);
  }

  .sub-hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .hotkey-wrapper {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }

  .hotkey-btn {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    padding: var(--space-2);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: center;
  }

  .hotkey-btn:hover {
    border-color: var(--border-accent);
  }

  .hotkey-btn.recording {
    background: rgba(239, 68, 68, 0.15);
    border-color: #ef4444;
    color: #ef4444;
    animation: pulse 1s infinite;
  }

  .clear-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    padding: var(--space-2) var(--space-3);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .clear-btn:hover {
    color: #ef4444;
    border-color: #ef4444;
  }

  /* 规则编辑器 */
  .rules-editor-wrapper {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: var(--space-3);
  }

  .rules-editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .rules-title-group {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .rules-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
  }

  .rules-badge {
    font-size: var(--text-xs);
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    color: var(--text-muted);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .rules-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .btn-reset {
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    padding: var(--space-1) var(--space-3);
    font-size: var(--text-xs);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-reset:hover {
    color: var(--text-primary);
    border-color: var(--border-accent);
  }

  .btn-save {
    background: var(--accent-primary, #3b82f6);
    border: none;
    border-radius: var(--radius-sm);
    color: #ffffff;
    padding: var(--space-1) var(--space-3);
    font-size: var(--text-xs);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-save:hover:not(:disabled) {
    opacity: 0.9;
    filter: brightness(1.1);
  }

  .btn-save:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-save.save-success {
    background: #10b981;
  }

  .rules-textarea {
    width: 100%;
    box-sizing: border-box;
    background: var(--bg-primary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-family: var(--font-mono, 'Consolas', 'Courier New', monospace);
    font-size: 13px;
    line-height: 1.5;
    padding: var(--space-3);
    resize: vertical;
    white-space: pre;
    overflow-x: auto;
  }

  .rules-textarea:focus {
    outline: none;
    border-color: var(--accent-primary, #3b82f6);
  }

  /* 帮助折叠 */
  .rules-help-details {
    background: var(--bg-primary);
    border: 1px dashed var(--border-primary);
    border-radius: var(--radius-sm);
    padding: var(--space-2) var(--space-3);
  }

  .rules-help-summary {
    cursor: pointer;
    font-size: var(--text-xs);
    color: var(--text-secondary);
    user-select: none;
  }

  .rules-help-summary:hover {
    color: var(--accent-primary, #60a5fa);
  }

  .rules-help-content {
    margin-top: var(--space-3);
    padding-top: var(--space-2);
    border-top: 1px solid var(--border-primary);
    font-size: var(--text-xs);
    color: var(--text-secondary);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .help-title {
    color: var(--text-primary);
  }

  .help-list {
    margin: var(--space-1) 0 0 var(--space-4);
    padding: 0;
    line-height: 1.6;
  }

  .help-list code {
    background: var(--bg-secondary);
    padding: 1px 4px;
    border-radius: 3px;
    font-family: var(--font-mono);
    color: var(--accent-primary, #60a5fa);
  }

  /* 通用功能卡片与说明样式 */
  .feature-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-top: var(--space-2);
    transition: opacity 0.2s ease;
  }

  .feature-container.is-disabled {
    opacity: 0.45;
    pointer-events: none;
  }

  .feature-controls-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-md);
    padding: var(--space-3);
  }

  .sub-toggle-wrapper {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-top: var(--space-1);
  }

  .sub-toggle-text {
    font-size: var(--text-xs);
    color: var(--text-muted);
    line-height: 1.4;
  }

  .info-banner {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    background: var(--bg-primary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    padding: var(--space-3);
    font-size: var(--text-xs);
    line-height: 1.6;
    color: var(--text-secondary);
  }

  .info-icon {
    font-size: 16px;
    flex-shrink: 0;
  }

  .info-content {
    flex: 1;
  }

  .info-content strong {
    color: var(--text-primary);
    margin-right: var(--space-1);
  }

  .info-content code {
    background: var(--bg-secondary);
    padding: 1px 5px;
    border-radius: 3px;
    font-family: var(--font-mono);
    color: var(--accent-primary, #60a5fa);
  }

  /* 游戏原生六系抗性彩色标签 */
  .res-tag {
    display: inline-block;
    padding: 0 5px;
    margin: 0 2px;
    border-radius: 3px;
    font-weight: 600;
    font-size: 11px;
  }

  .res-tag.res-phys {
    background: rgba(178, 190, 195, 0.2);
    color: #dfe6e9;
    border: 1px solid rgba(178, 190, 195, 0.4);
  }

  .res-tag.res-magic {
    background: rgba(232, 67, 147, 0.2);
    color: #fd79a8;
    border: 1px solid rgba(232, 67, 147, 0.4);
  }

  .res-tag.res-fire {
    background: rgba(225, 112, 85, 0.2);
    color: #ff7675;
    border: 1px solid rgba(225, 112, 85, 0.4);
  }

  .res-tag.res-ltng {
    background: rgba(253, 203, 110, 0.2);
    color: #ffeaa7;
    border: 1px solid rgba(253, 203, 110, 0.4);
  }

  .res-tag.res-cold {
    background: rgba(116, 185, 255, 0.2);
    color: #74b9ff;
    border: 1px solid rgba(116, 185, 255, 0.4);
  }

  .res-tag.res-pois {
    background: rgba(85, 239, 196, 0.2);
    color: #55efc4;
    border: 1px solid rgba(85, 239, 196, 0.4);
  }
</style>
