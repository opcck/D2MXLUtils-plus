<script lang="ts">
  import { playSound } from '../lib/sound-player';
  import { settingsStore } from '../stores';
  import {
    ITEM_CATEGORIES,
    QUALITY_OPTIONS,
    TIER_OPTIONS,
    SOCKET_OPTIONS,
    CLASS_OPTIONS,
    COLOR_OPTIONS,
    ACTION_OPTIONS,
    SOUND_OPTIONS,
    STAT_OPTIONS,
    FILTER_PRESETS,
    buildStatThresholdRegex,
    type PresetOption,
    type StatOption,
  } from '../lib/filter-generator-data';

  interface Props {
    open?: boolean;
    oninsert?: (rule: string, mode: 'cursor' | 'append') => void;
    onclose?: () => void;
  }

  let { open = $bindable(false), oninsert, onclose }: Props = $props();

  // 14 维度状态定义
  let namePattern = $state('');
  let quality = $state('');
  let tier = $state('');
  let sockets = $state('');
  let clvlEnabled = $state(false);
  let minClvl = $state<number | null>(null);
  let maxClvl = $state<number | null>(null);
  let ilvlEnabled = $state(false);
  let minIlvl = $state<number | null>(null);
  let maxIlvl = $state<number | null>(null);
  let charClass = $state('');
  let eth = $state(false);
  let quest = $state(false);

  interface StatRow {
    id: number;
    statId: string;
    minValue: number | null;
    customRegex: string;
  }

  let nextRowId = 1;
  let stats = $state<StatRow[]>([]);

  let color = $state('');
  let action = $state('');
  let sound = $state('');
  let notify = $state(false);
  let stat = $state(false);
  let map = $state(false);

  // 交互辅助状态
  let copySuccess = $state(false);
  let isPlayingSound = $state(false);
  let selectedCategory = $state('');

  // 处理大类选择联动
  function handleCategoryChange(pattern: string) {
    selectedCategory = pattern;
    if (pattern) {
      namePattern = pattern;
    }
  }

  // 预设应用
  function applyPreset(preset: PresetOption) {
    const r = preset.rule;
    namePattern = r.namePattern ?? '';
    quality = r.quality ?? '';
    tier = r.tier ?? '';
    sockets = r.sockets ?? '';
    clvlEnabled = r.minClvl != null || r.maxClvl != null;
    minClvl = r.minClvl ?? null;
    maxClvl = r.maxClvl ?? null;
    ilvlEnabled = r.minIlvl != null || r.maxIlvl != null;
    minIlvl = r.minIlvl ?? null;
    maxIlvl = r.maxIlvl ?? null;
    charClass = r.charClass ?? '';
    eth = !!r.eth;
    quest = false;

    if (r.stats && r.stats.length > 0) {
      stats = r.stats.map((s) => ({
        id: nextRowId++,
        statId: s.statId,
        minValue: s.minValue ?? null,
        customRegex: s.customRegex ?? '',
      }));
    } else {
      stats = [];
    }

    color = r.color ?? '';
    action = r.action ?? '';
    sound = r.sound ?? '';
    notify = !!r.notify;
    stat = !!r.stat;
    map = !!r.map;
  }

  // 属性词条增删
  function addStatRow() {
    stats = [
      ...stats,
      {
        id: nextRowId++,
        statId: 'all_skills',
        minValue: null,
        customRegex: '',
      },
    ];
  }

  function removeStatRow(id: number) {
    stats = stats.filter((s) => s.id !== id);
  }

  // 试听音效
  async function testSound() {
    if (!sound || sound === 'sound_none') return;
    const match = sound.match(/^sound(\d+)$/);
    if (!match) return;
    const slotIndex = parseInt(match[1], 10);
    isPlayingSound = true;
    try {
      await playSound(slotIndex, settingsStore.settings.soundVolume);
    } catch (e) {
      console.warn('[FilterRuleGenerator] Sound playback failed:', e);
    } finally {
      setTimeout(() => {
        isPlayingSound = false;
      }, 500);
    }
  }

  // 重置清空所有字段
  function resetAll() {
    namePattern = '';
    selectedCategory = '';
    quality = '';
    tier = '';
    sockets = '';
    clvlEnabled = false;
    minClvl = null;
    maxClvl = null;
    ilvlEnabled = false;
    minIlvl = null;
    maxIlvl = null;
    charClass = '';
    eth = false;
    quest = false;
    stats = [];
    color = '';
    action = '';
    sound = '';
    notify = false;
    stat = false;
    map = false;
  }

  // 动态组装 DSL 规则 (符合 14 维度严格语法规范)
  let generatedRule = $derived.by(() => {
    const tokens: string[] = [];

    // 1. ["名称模式"]
    if (namePattern.trim()) {
      tokens.push(`"${namePattern.trim()}"`);
    }

    // 2. [品质]
    if (quality) {
      tokens.push(quality);
    }

    // 3. [阶级]
    if (tier) {
      tokens.push(tier);
    }

    // 4. [凹槽/镶孔]
    if (sockets) {
      tokens.push(sockets);
    }

    // 5. [等级限制]
    if (clvlEnabled) {
      if (minClvl != null && minClvl > 0) tokens.push(`min_clvl${minClvl}`);
      if (maxClvl != null && maxClvl > 0) tokens.push(`max_clvl${maxClvl}`);
    }
    if (ilvlEnabled) {
      if (minIlvl != null && minIlvl > 0) tokens.push(`min_ilvl${minIlvl}`);
      if (maxIlvl != null && maxIlvl > 0) tokens.push(`max_ilvl${maxIlvl}`);
    }

    // 6. [职业]
    if (charClass) {
      tokens.push(charClass);
    }

    // 7. [eth] 与 quest
    if (eth) {
      tokens.push('eth');
    }
    if (quest) {
      tokens.push('quest');
    }

    // 8. {属性正则}*
    for (const row of stats) {
      if (row.customRegex.trim()) {
        tokens.push(`{${row.customRegex.trim()}}`);
      } else if (row.statId) {
        const statDef = STAT_OPTIONS.find((s) => s.id === row.statId);
        if (statDef) {
          const regex = buildStatThresholdRegex(statDef, row.minValue);
          tokens.push(`{${regex}}`);
        }
      }
    }

    // 9. [颜色]
    if (color) {
      tokens.push(color);
    }

    // 10. [show|hide]
    if (action) {
      tokens.push(action);
    }

    // 11. [音效]
    if (sound) {
      tokens.push(sound);
    }

    // 12. [notify]
    if (notify) {
      tokens.push('notify');
    }

    // 13. [stat]
    if (stat) {
      tokens.push('stat');
    }

    // 14. [map]
    if (map) {
      tokens.push('map');
    }

    return tokens.join(' ');
  });

  // 复制规则
  async function copyRule() {
    if (!generatedRule) return;
    try {
      await navigator.clipboard.writeText(generatedRule);
      copySuccess = true;
      setTimeout(() => {
        copySuccess = false;
      }, 2000);
    } catch (e) {
      console.error('Failed to copy rule:', e);
    }
  }

  // 插入并关闭
  function handleInsert(mode: 'cursor' | 'append') {
    if (!generatedRule) return;
    oninsert?.(generatedRule, mode);
    open = false;
  }

  function handleClose() {
    open = false;
    onclose?.();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && open) {
      handleClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- 遮罩层 -->
  <div class="modal-backdrop" onclick={handleClose} role="presentation">
    <div
      class="generator-modal"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
    >
      <!-- 头部 -->
      <header class="modal-header">
        <div class="title-wrap">
          <span class="header-icon">🛠️</span>
          <h2 id="modal-title" class="modal-title">掉落过滤规则生成器</h2>
          <span class="header-tag">14 维度官方规范</span>
        </div>
        <button type="button" class="close-btn" onclick={handleClose} title="关闭 (Esc)">✕</button>
      </header>

      <!-- 快捷预设横条 -->
      <div class="presets-section">
        <span class="preset-label">常用预设：</span>
        <div class="preset-chips">
          {#each FILTER_PRESETS as preset}
            <button
              type="button"
              class="preset-chip"
              title={preset.desc}
              onclick={() => applyPreset(preset)}
            >
              {preset.name}
            </button>
          {/each}
        </div>
      </div>

      <!-- 核心表单内容区 (双列并排) -->
      <div class="modal-body">
        <!-- 左侧列：基础特征与约束 -->
        <div class="column">
          <!-- 1. 物品名称与分类 -->
          <div class="form-group">
            <label class="group-label" for="category-select">
              1. 名称模式 (Name Pattern)
              <span class="label-hint">官方分类或输入正则表达式</span>
            </label>
            <div class="input-row">
              <select
                id="category-select"
                class="form-select category-select"
                value={selectedCategory}
                onchange={(e) => handleCategoryChange((e.target as HTMLSelectElement).value)}
              >
                <option value="">-- 选择装备/杂项预设分类 --</option>
                {#each ITEM_CATEGORIES as item}
                  <option value={item.pattern}>{item.label}</option>
                {/each}
              </select>
            </div>
            <div class="input-row">
              <input
                type="text"
                class="form-input"
                bind:value={namePattern}
                placeholder={'例如: "Ring$" 或 "Amulet" 或 "^(Ber|Jah)$"'}
              />
            </div>
          </div>

          <!-- 2. 品质 -->
          <div class="form-group">
            <label class="group-label" for="quality-select">2. 品质 (Quality)</label>
            <select id="quality-select" class="form-select" bind:value={quality}>
              {#each QUALITY_OPTIONS as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>

          <!-- 3. 品级 -->
          <div class="form-group">
            <label class="group-label" for="tier-select">3. 品级 (Tier)</label>
            <select id="tier-select" class="form-select" bind:value={tier}>
              {#each TIER_OPTIONS as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>

          <!-- 4. 镶孔 -->
          <div class="form-group">
            <label class="group-label" for="socket-select">4. 镶孔 (Sockets)</label>
            <select id="socket-select" class="form-select" bind:value={sockets}>
              {#each SOCKET_OPTIONS as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>

          <!-- 5. 职业限定 -->
          <div class="form-group">
            <label class="group-label" for="class-select">5. 专属-职业限定 (Class)</label>
            <select id="class-select" class="form-select" bind:value={charClass}>
              {#each CLASS_OPTIONS as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>

          <!-- 6. 生效等级范围 (clvl / ilvl) -->
          <div class="form-group">
            <span class="group-label">6. 生效等级范围 (Level Constraints)</span>
            <div class="level-box">
              <div class="level-row">
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={clvlEnabled} />
                  <span>角色等级范围 (clvl)</span>
                </label>
                {#if clvlEnabled}
                  <div class="range-inputs">
                    <input
                      type="number"
                      class="mini-input"
                      bind:value={minClvl}
                      placeholder="min"
                      min="1"
                      max="150"
                    />
                    <span>~</span>
                    <input
                      type="number"
                      class="mini-input"
                      bind:value={maxClvl}
                      placeholder="max"
                      min="1"
                      max="150"
                    />
                  </div>
                {/if}
              </div>

              <div class="level-row">
                <label class="checkbox-label">
                  <input type="checkbox" bind:checked={ilvlEnabled} />
                  <span>物品等级范围 (ilvl)</span>
                </label>
                {#if ilvlEnabled}
                  <div class="range-inputs">
                    <input
                      type="number"
                      class="mini-input"
                      bind:value={minIlvl}
                      placeholder="min"
                      min="1"
                      max="150"
                    />
                    <span>~</span>
                    <input
                      type="number"
                      class="mini-input"
                      bind:value={maxIlvl}
                      placeholder="max"
                      min="1"
                      max="150"
                    />
                  </div>
                {/if}
              </div>
            </div>
          </div>

          <!-- 7. 无形与任务物品 -->
          <div class="form-group">
            <span class="group-label">7. 特殊状态 (Ethereal & Quest)</span>
            <div class="checkbox-row">
              <label class="checkbox-chip" class:active={eth}>
                <input type="checkbox" bind:checked={eth} />
                <span>物品无形 (eth)</span>
              </label>
              <label class="checkbox-chip" class:active={quest}>
                <input type="checkbox" bind:checked={quest} />
                <span>任务物品 (quest)</span>
              </label>
            </div>
          </div>
        </div>

        <!-- 右侧列：属性筛选与视觉通知动作 -->
        <div class="column">
          <!-- 8. 属性正则匹配 (Stat Regex) -->
          <div class="form-group">
            <div class="group-header-flex">
              <span class="group-label">8. 属性词条筛选 &#123;属性正则&#125;</span>
              <button type="button" class="mini-btn add-btn" onclick={addStatRow}>+ 添加词条</button
              >
            </div>

            {#if stats.length === 0}
              <div class="empty-hint">尚未添加属性词条筛选（留空匹配所有词条）。</div>
            {:else}
              <div class="stats-list">
                {#each stats as row (row.id)}
                  <div class="stat-card">
                    <div class="stat-row-top">
                      <select class="form-select flex-1" bind:value={row.statId}>
                        {#each STAT_OPTIONS as statOpt}
                          <option value={statOpt.id}>{statOpt.label}</option>
                        {/each}
                      </select>
                      <button
                        type="button"
                        class="remove-stat-btn"
                        onclick={() => removeStatRow(row.id)}
                        title="删除该词条"
                      >
                        ✕
                      </button>
                    </div>

                    <div class="stat-row-bottom">
                      <label class="threshold-label">
                        <span>数值下限 &ge;</span>
                        <input
                          type="number"
                          class="threshold-input"
                          bind:value={row.minValue}
                          placeholder="不限"
                        />
                      </label>
                      <input
                        type="text"
                        class="custom-regex-input"
                        bind:value={row.customRegex}
                        placeholder="或输入自定义正则"
                      />
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- 9. 地面动作 [show|hide] -->
          <div class="form-group">
            <label class="group-label" for="action-select">9. 地面动作 (Action)</label>
            <select id="action-select" class="form-select" bind:value={action}>
              {#each ACTION_OPTIONS as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>

          <!-- 10. 提示颜色 [颜色] -->
          <div class="form-group">
            <span class="group-label">10. 提示颜色 (Color)</span>
            <div class="color-palette">
              {#each COLOR_OPTIONS as c}
                <button
                  type="button"
                  class="color-btn"
                  class:selected={color === c.value}
                  onclick={() => (color = c.value)}
                  title={c.label}
                >
                  <span class="color-dot" style:background={c.hex}></span>
                  <span class="color-name">{c.value || '默认'}</span>
                </button>
              {/each}
            </div>
          </div>

          <!-- 11. 音效 [音效] 及试听 -->
          <div class="form-group">
            <label class="group-label" for="sound-select">11. 掉落音效 (Sound)</label>
            <div class="sound-row">
              <select id="sound-select" class="form-select flex-1" bind:value={sound}>
                {#each SOUND_OPTIONS as opt}
                  <option value={opt.value}>{opt.label}</option>
                {/each}
              </select>
              <button
                type="button"
                class="sound-test-btn"
                onclick={testSound}
                disabled={!sound || sound === 'sound_none' || isPlayingSound}
                title="试听当前选中的音效"
              >
                {isPlayingSound ? '🔊 播放中' : '🔊 试听'}
              </button>
            </div>
          </div>

          <!-- 12, 13, 14. 通知与小地图 [notify] [stat] [map] -->
          <div class="form-group">
            <span class="group-label">12~14. 通知与地图动作 (Notify, Stat, Map)</span>
            <div class="checkbox-row">
              <label class="checkbox-chip" class:active={notify}>
                <input type="checkbox" bind:checked={notify} />
                <span>🔔 弹窗通知 (notify)</span>
              </label>
              <label class="checkbox-chip" class:active={stat}>
                <input type="checkbox" bind:checked={stat} />
                <span>📜 词条详情 (stat)</span>
              </label>
              <label class="checkbox-chip" class:active={map}>
                <input type="checkbox" bind:checked={map} />
                <span>📍 在地图上显示 (map)</span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部：实时预览与操作工具栏 -->
      <footer class="modal-footer">
        <div class="preview-box">
          <div class="preview-header">
            <span class="preview-title">实时 DSL 规则装配预览：</span>
            {#if copySuccess}
              <span class="copy-success-badge">✓ 已复制到剪切版</span>
            {/if}
          </div>
          <code class="preview-code">
            {generatedRule || '（当前无生效条件，请在上方勾选或输入）'}
          </code>
        </div>

        <div class="action-buttons">
          <button type="button" class="btn secondary-btn" onclick={resetAll}>清空重置</button>
          <button
            type="button"
            class="btn secondary-btn"
            onclick={copyRule}
            disabled={!generatedRule}
          >
            📋 复制规则
          </button>
          <button
            type="button"
            class="btn primary-btn"
            onclick={() => handleInsert('cursor')}
            disabled={!generatedRule}
          >
            🎯 插入到当前光标
          </button>
          <button
            type="button"
            class="btn accent-btn"
            onclick={() => handleInsert('append')}
            disabled={!generatedRule}
          >
            ➕ 追加至文件末尾
          </button>
        </div>
      </footer>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.72);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 16px;
  }

  .generator-modal {
    background: var(--bg-secondary, #1a1a1f);
    border: 1px solid var(--border-primary, #33333d);
    border-radius: var(--radius-lg, 10px);
    width: 960px;
    max-width: 95vw;
    max-height: 92vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
    overflow: hidden;
    color: var(--text-primary, #e8e6e3);
    animation: modal-appear 0.18s ease-out;
  }

  @keyframes modal-appear {
    from {
      opacity: 0;
      transform: scale(0.97) translateY(-8px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 20px;
    background: var(--bg-tertiary, #22222a);
    border-bottom: 1px solid var(--border-primary, #33333d);
  }

  .title-wrap {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .header-icon {
    font-size: 20px;
  }

  .modal-title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary, #fff);
  }

  .header-tag {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 12px;
    background: color-mix(in srgb, var(--accent-primary, #c7b377) 20%, transparent);
    color: var(--accent-primary, #c7b377);
    border: 1px solid color-mix(in srgb, var(--accent-primary, #c7b377) 40%, transparent);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary, #a0a0aa);
    font-size: 18px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
  }

  /* 预设横条 */
  .presets-section {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    background: color-mix(in srgb, var(--bg-tertiary, #22222a) 60%, transparent);
    border-bottom: 1px solid var(--border-primary, #2a2a35);
    overflow-x: auto;
    white-space: nowrap;
  }

  .preset-label {
    font-size: 12px;
    color: var(--text-secondary, #999);
    flex-shrink: 0;
  }

  .preset-chips {
    display: flex;
    gap: 8px;
    overflow-x: auto;
    scrollbar-width: thin;
  }

  .preset-chip {
    padding: 4px 10px;
    background: var(--bg-elevated, #2a2a35);
    border: 1px solid var(--border-primary, #3a3a48);
    border-radius: 4px;
    color: var(--text-primary, #ddd);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s ease;
    flex-shrink: 0;
  }

  .preset-chip:hover {
    background: var(--accent-primary, #c7b377);
    color: #000;
    border-color: var(--accent-primary, #c7b377);
  }

  /* 内容主体双列 */
  .modal-body {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
    padding: 18px 20px;
    overflow-y: auto;
    max-height: 54vh;
  }

  .column {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .group-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e8e6e3);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .label-hint {
    font-size: 11px;
    font-weight: normal;
    color: var(--text-secondary, #888);
  }

  .group-header-flex {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .input-row {
    display: flex;
    gap: 8px;
  }

  .form-input,
  .form-select {
    width: 100%;
    background: var(--bg-tertiary, #1f1f26);
    border: 1px solid var(--border-primary, #333340);
    border-radius: var(--radius-sm, 5px);
    color: var(--text-primary, #eee);
    padding: 7px 10px;
    font-size: 12px;
    outline: none;
    transition: border-color 0.15s ease;
  }

  .form-input:focus,
  .form-select:focus {
    border-color: var(--accent-primary, #c7b377);
  }

  .category-select {
    background: var(--bg-elevated, #262630);
    color: var(--accent-primary, #c7b377);
  }

  /* 等级约束卡片 */
  .level-box {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: var(--bg-tertiary, #1f1f26);
    border: 1px solid var(--border-primary, #333340);
    border-radius: 6px;
    padding: 8px 12px;
  }

  .level-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .range-inputs {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .mini-input {
    width: 58px;
    background: var(--bg-secondary, #16161c);
    border: 1px solid var(--border-primary, #3a3a48);
    border-radius: 4px;
    color: #fff;
    padding: 3px 6px;
    font-size: 12px;
    text-align: center;
  }

  /* 复选芯片组 */
  .checkbox-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    cursor: pointer;
  }

  .checkbox-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    background: var(--bg-tertiary, #1f1f26);
    border: 1px solid var(--border-primary, #333340);
    border-radius: 5px;
    font-size: 12px;
    cursor: pointer;
    user-select: none;
    transition: all 0.15s ease;
  }

  .checkbox-chip input {
    cursor: pointer;
  }

  .checkbox-chip.active {
    background: color-mix(in srgb, var(--accent-primary, #c7b377) 18%, transparent);
    border-color: var(--accent-primary, #c7b377);
    color: var(--accent-primary, #c7b377);
  }

  /* 词条筛选卡片 */
  .empty-hint {
    font-size: 12px;
    color: var(--text-secondary, #777);
    padding: 8px;
    background: var(--bg-tertiary, #1a1a20);
    border-radius: 4px;
    text-align: center;
  }

  .stats-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 180px;
    overflow-y: auto;
    padding-right: 4px;
  }

  .stat-card {
    background: var(--bg-tertiary, #1f1f26);
    border: 1px solid var(--border-primary, #333340);
    border-radius: 6px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .stat-row-top {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .stat-row-bottom {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .threshold-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--text-secondary, #aaa);
    white-space: nowrap;
  }

  .threshold-input {
    width: 65px;
    background: var(--bg-secondary, #16161c);
    border: 1px solid var(--border-primary, #3a3a48);
    border-radius: 4px;
    color: #fff;
    padding: 4px 6px;
    font-size: 11px;
  }

  .custom-regex-input {
    flex: 1;
    background: var(--bg-secondary, #16161c);
    border: 1px solid var(--border-primary, #3a3a48);
    border-radius: 4px;
    color: #fff;
    padding: 4px 6px;
    font-size: 11px;
  }

  .remove-stat-btn {
    background: none;
    border: none;
    color: #ff6b6b;
    cursor: pointer;
    font-size: 13px;
    padding: 2px 6px;
  }

  .mini-btn {
    padding: 2px 8px;
    font-size: 11px;
    border-radius: 4px;
    cursor: pointer;
    border: 1px solid var(--accent-primary, #c7b377);
    background: transparent;
    color: var(--accent-primary, #c7b377);
    transition: all 0.15s ease;
  }

  .mini-btn:hover {
    background: var(--accent-primary, #c7b377);
    color: #000;
  }

  /* 颜色色盘 */
  .color-palette {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .color-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: var(--bg-tertiary, #1f1f26);
    border: 1px solid var(--border-primary, #333340);
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    color: var(--text-primary, #ccc);
    transition: all 0.15s ease;
  }

  .color-btn:hover {
    border-color: #777;
  }

  .color-btn.selected {
    border-color: var(--accent-primary, #c7b377);
    background: color-mix(in srgb, var(--accent-primary, #c7b377) 15%, transparent);
    color: #fff;
  }

  .color-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 1px solid rgba(0, 0, 0, 0.4);
  }

  /* 音效 */
  .sound-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .sound-test-btn {
    padding: 7px 12px;
    background: var(--bg-elevated, #282834);
    border: 1px solid var(--border-primary, #3e3e4f);
    border-radius: 5px;
    color: var(--accent-primary, #c7b377);
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.15s ease;
  }

  .sound-test-btn:hover:not(:disabled) {
    background: var(--accent-primary, #c7b377);
    color: #000;
  }

  .sound-test-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* 底部预览与操作栏 */
  .modal-footer {
    padding: 14px 20px;
    background: var(--bg-tertiary, #1c1c22);
    border-top: 1px solid var(--border-primary, #2e2e38);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .preview-box {
    background: #0f0f14;
    border: 1px solid var(--border-primary, #2a2a35);
    border-radius: 6px;
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .preview-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary, #888);
  }

  .copy-success-badge {
    font-size: 11px;
    color: #4caf50;
    font-weight: 600;
  }

  .preview-code {
    font-family: var(--font-mono, 'Fira Code', Consolas, monospace);
    font-size: 13px;
    color: #dcdcaa;
    word-break: break-all;
    white-space: pre-wrap;
    line-height: 1.4;
  }

  .action-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .btn {
    padding: 8px 14px;
    border-radius: 5px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    border: 1px solid transparent;
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .secondary-btn {
    background: var(--bg-elevated, #282834);
    border-color: var(--border-primary, #3a3a48);
    color: var(--text-primary, #ddd);
  }

  .secondary-btn:hover:not(:disabled) {
    background: #333342;
    color: #fff;
  }

  .primary-btn {
    background: #2b5278;
    border-color: #3b6b9a;
    color: #fff;
  }

  .primary-btn:hover:not(:disabled) {
    background: #366596;
  }

  .accent-btn {
    background: var(--accent-primary, #c7b377);
    border-color: var(--accent-primary, #c7b377);
    color: #1a1a1f;
    font-weight: 600;
  }

  .accent-btn:hover:not(:disabled) {
    background: #dfca8f;
  }

  .flex-1 {
    flex: 1;
  }
</style>
