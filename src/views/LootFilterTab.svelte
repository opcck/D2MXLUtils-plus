<script lang="ts">
  import { onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { RulesEditor, type ValidationResult } from '../editor';
  import { ProfileSelector, Toggle } from '../components';
  import { settingsStore } from '../stores';

  type SaveState = 'saved' | 'unsaved' | 'invalid' | 'saving' | 'error';

  let dslText = $state('');
  let selectedProfile = $state(settingsStore.settings.activeProfile || '');
  let validationStatus = $state<'idle' | 'valid' | 'error'>('idle');
  let errorCount = $state(0);
  let ruleCount = $state(0);
  // Default-mode state derived from the parsed DSL (mirrors FilterConfig.hide_all).
  let hideAll = $state(false);

  let saveState = $state<SaveState>('saved');
  let saveError = $state<string | null>(null);
  let lastSavedText = $state('');
  let inflightSave: Promise<void> | null = null;

  // "Show matches" live highlight — not persisted, resets to off whenever
  // this tab (re)mounts.
  let showMatches = $state(false);
  let rulesEditorRef: RulesEditor;

  async function handleShowMatchesChange(enabled: boolean) {
    showMatches = enabled;
    try {
      await invoke('set_live_match_highlight', { enabled });
    } catch (e) {
      console.error('[LootFilterTab] Failed to toggle live match highlight:', e);
    }
  }

  $effect(() => {
    if (!showMatches) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listen<number[]>('filter-rule-matched', (event) => {
      rulesEditorRef?.flashLines(
        event.payload,
        settingsStore.settings.liveMatchHighlightDurationMs,
      );
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  });

  onDestroy(() => {
    if (showMatches) {
      invoke('set_live_match_highlight', { enabled: false }).catch(() => {});
    }
  });

  async function syncFilterConfig() {
    try {
      const config = await invoke<any>('parse_filter_dsl', { text: dslText });
      hideAll = !!config.hide_all;
      await invoke('set_filter_config', { config });
    } catch (e) {
      console.error('[LootFilterTab] Failed to sync filter config:', e);
    }
  }

  // Saves are dispatched from handleValidation, not here, so we piggy-back on
  // the linter's debounce and never race a stale validation verdict.
  function updateSaveState() {
    if (saveState === 'saving' || saveState === 'error') return;
    if (!selectedProfile) return;
    if (dslText === lastSavedText) {
      saveState = 'saved';
      return;
    }
    // Treat "idle" optimistically — don't flash "fix errors" before the linter runs.
    saveState = validationStatus === 'error' ? 'invalid' : 'unsaved';
  }

  async function doSave(profileName: string, text: string) {
    if (inflightSave) {
      try {
        await inflightSave;
      } catch {
        /* ignore */
      }
    }
    saveState = 'saving';
    saveError = null;
    const p = (async () => {
      try {
        await invoke('save_profile', { name: profileName, rulesText: text });
        if (selectedProfile === profileName) {
          lastSavedText = text;
          if (dslText === text) {
            saveState = 'saved';
            await syncFilterConfig();
          } else {
            // Edits landed mid-save; next handleValidation pass will re-save.
            saveState = validationStatus === 'valid' ? 'unsaved' : 'invalid';
          }
        }
      } catch (e) {
        saveError = String(e);
        if (selectedProfile === profileName) saveState = 'error';
        console.error('[LootFilterTab] auto-save failed:', e);
      }
    })();
    inflightSave = p;
    try {
      await p;
    } finally {
      if (inflightSave === p) inflightSave = null;
    }
  }

  function retrySave() {
    if (saveState !== 'error') return;
    saveError = null;
    if (!selectedProfile) return;
    if (dslText === lastSavedText) {
      saveState = 'saved';
      return;
    }
    if (validationStatus !== 'valid') {
      saveState = 'invalid';
      return;
    }
    void doSave(selectedProfile, dslText);
  }

  // Only hard errors block save; warnings/info are advisory. This is also
  // the sole kick-off point for auto-save — the linter's 500 ms debounce
  // doubles as the save debounce, so the save always sees a fresh verdict.
  function handleValidation(result: ValidationResult) {
    const hardErrors = result.errors.filter((e) => e.severity === 'error');
    if (hardErrors.length > 0) {
      validationStatus = 'error';
      errorCount = hardErrors.length;
    } else {
      validationStatus = 'valid';
      ruleCount = result.ruleCount;
    }

    if (!selectedProfile) return;
    if (dslText === lastSavedText) {
      if (saveState !== 'saving' && saveState !== 'error') saveState = 'saved';
      return;
    }
    if (saveState === 'saving' || saveState === 'error') return;
    if (validationStatus === 'valid') {
      void doSave(selectedProfile, dslText);
    } else {
      saveState = 'invalid';
    }
  }

  function handleChange(_newValue: string) {
    // Do NOT reset validationStatus here — it makes the rule-count badge
    // flicker to "—" on every keystroke.
    updateSaveState();
  }

  // Ctrl+S: flush the save now instead of waiting for the linter debounce.
  async function handleSave(newValue: string) {
    dslText = newValue;
    if (!selectedProfile) return;
    if (dslText === lastSavedText) {
      saveState = 'saved';
      return;
    }
    if (validationStatus !== 'valid') {
      saveState = 'invalid';
      return;
    }
    await doSave(selectedProfile, dslText);
  }

  async function handleProfileLoad(name: string, rulesText: string) {
    dslText = rulesText;
    lastSavedText = rulesText;
    selectedProfile = name;
    saveState = 'saved';
    saveError = null;
    validationStatus = 'idle';

    if (name) {
      settingsStore.set('activeProfile', name);
    }

    // Push the loaded filter to the scanner immediately. The backend is
    // authoritative for parsing, so there's no need to wait for the linter.
    await syncFilterConfig();
  }

  // handleProfileLoad covers everything; this callback is just API plumbing.
  function handleProfileSelect(_profile: { name: string } | null) {}

  function handleFoldsChange(lines: number[]) {
    if (!selectedProfile) return;
    settingsStore.set('foldedLines', {
      ...settingsStore.settings.foldedLines,
      [selectedProfile]: lines,
    });
  }
</script>

<section class="loot-filter-tab">
  <header class="tab-header">
    <div class="header-left">
      <span
        class="status-badge"
        class:valid={validationStatus === 'valid'}
        class:error={validationStatus === 'error'}
      >
        {#if validationStatus === 'valid'}
          ✓ {ruleCount} 条规则
        {:else if validationStatus === 'error'}
          ✗ {errorCount} 处错误
        {:else}
          —
        {/if}
      </span>
      <span
        class="default-mode-badge"
        class:hide={hideAll}
        title="在文件顶部添加 'hide default' 可默认隐藏所有未匹配物品。"
      >
        默认：{hideAll ? '隐藏' : '显示'}未匹配物品
      </span>
    </div>

    <div class="header-actions">
      <span class="show-matches-toggle" title="掉落发生时，闪烁高亮匹配到该物品的规则行">
        <Toggle checked={showMatches} label="显示实时匹配" onchange={handleShowMatchesChange} />
      </span>

      {#if saveState === 'error'}
        <button
          type="button"
          class="save-status error"
          onclick={retrySave}
          title={saveError ?? '保存失败'}
        >
          ⚠ 保存失败 — 重试
        </button>
      {:else}
        <span
          class="save-status"
          class:saved={saveState === 'saved'}
          class:unsaved={saveState === 'unsaved'}
          class:invalid={saveState === 'invalid'}
          class:saving={saveState === 'saving'}
        >
          {#if saveState === 'saved'}
            ✓ 已保存
          {:else if saveState === 'unsaved'}
            ● 未保存
          {:else if saveState === 'invalid'}
            ⚠ 未保存 — 存在语法错误
          {:else if saveState === 'saving'}
            … 正在保存
          {/if}
        </span>
      {/if}

      <ProfileSelector
        bind:selectedProfile
        onselect={handleProfileSelect}
        onload={handleProfileLoad}
      />
    </div>
  </header>

  <div class="editor-container">
    <RulesEditor
      bind:this={rulesEditorRef}
      bind:value={dslText}
      onchange={handleChange}
      onsave={handleSave}
      onvalidate={handleValidation}
      initialFoldedLines={settingsStore.settings.foldedLines[selectedProfile] ?? []}
      onFoldsChange={handleFoldsChange}
    />
  </div>

  <div class="syntax-help">
    <details>
      <summary>语法参考指南</summary>
      <div class="help-content">
        <p>规则语法格式（所有部分均为可选 — 规则按从上到下最后命中的生效）：</p>
        <code
          >["名称"] [品质] [阶级] [凹槽] [等级限制] [职业] [eth] &#123;属性正则&#125; [颜色]
          [show|hide] [音效] [notify] [stat] [map]</code
        >

        <div class="help-columns">
          <div class="help-column">
            <h4>品质 (Quality)</h4>
            <ul>
              <li>
                <span class="kw-quality">unique</span> (暗金), <span class="kw-quality">set</span>
                (套装),
                <span class="kw-quality">rare</span> (亮金)
              </li>
              <li>
                <span class="kw-quality">magic</span> (蓝色魔法),
                <span class="kw-quality">craft</span>
                (手工),
                <span class="kw-quality">honor</span> (荣誉)
              </li>
              <li>
                <span class="kw-quality">normal</span> (普通), <span class="kw-quality">low</span>
                (劣质),
                <span class="kw-quality">superior</span> (超强)
              </li>
              <li>
                <span class="kw-quality">tu</span>, <span class="kw-quality">su</span>,
                <span class="kw-quality">ssu</span>, <span class="kw-quality">sssu</span>
                (暗金稀有阶级)
              </li>
            </ul>
          </div>

          <div class="help-column">
            <h4>阶级 (Tier)</h4>
            <ul>
              <li>
                <span class="kw-tier">sacred</span> (神圣),
                <span class="kw-tier">angelic</span> (天使),
                <span class="kw-tier">master</span> (宗师)
              </li>
              <li>
                <span class="kw-tier">0</span>,
                <span class="kw-tier">1</span>,
                <span class="kw-tier">2</span>,
                <span class="kw-tier">3</span>,
                <span class="kw-tier">4</span> (阶级1~4)
              </li>
            </ul>
          </div>

          <div class="help-column">
            <h4>凹槽 (Sockets)</h4>
            <ul>
              <li><span class="kw-socket">sockets0</span> (无凹槽)</li>
              <li>
                <span class="kw-socket">sockets1</span> -
                <span class="kw-socket">sockets6</span> (1~6孔)
              </li>
            </ul>
          </div>

          <div class="help-column">
            <h4>等级限制 (Level)</h4>
            <ul>
              <li>
                <span class="kw-level">min_clvl20</span>,
                <span class="kw-level">max_clvl99</span> (角色等级)
              </li>
              <li>
                <span class="kw-level">min_ilvl40</span>,
                <span class="kw-level">max_ilvl99</span> (物品等级)
              </li>
            </ul>
          </div>

          <div class="help-column">
            <h4>职业限定 (Class)</h4>
            <ul>
              <li>
                <span class="kw-class">amazon</span> (<span class="kw-class">zon</span> 亚马逊),
                <span class="kw-class">sorceress</span> (<span class="kw-class">sorc</span> 法师)
              </li>
              <li>
                <span class="kw-class">necromancer</span> (<span class="kw-class">necro</span>
                死灵),
                <span class="kw-class">paladin</span> (<span class="kw-class">pal</span>,
                <span class="kw-class">pally</span> 圣骑士)
              </li>
              <li>
                <span class="kw-class">barbarian</span> (<span class="kw-class">barb</span> 野蛮人),
                <span class="kw-class">druid</span> (<span class="kw-class">dru</span> 德鲁伊)
              </li>
              <li>
                <span class="kw-class">assassin</span> (<span class="kw-class">sin</span> 刺客)
              </li>
            </ul>
          </div>

          <div class="help-column">
            <h4>通知颜色 (Colors)</h4>
            <ul>
              <li>
                <span class="kw-c-gold">gold</span> (金色),
                <span class="kw-c-lime">lime</span> (亮绿),
                <span class="kw-c-red">red</span> (红色),
                <span class="kw-c-blue">blue</span> (蓝色)
              </li>
              <li>
                <span class="kw-c-white">white</span> (白色),
                <span class="kw-c-yellow">yellow</span> (黄色),
                <span class="kw-c-orange">orange</span> (橙色),
                <span class="kw-c-pink">pink</span> (粉色)
              </li>
              <li>
                <span class="kw-c-grey">grey</span> (灰色),
                <span class="kw-c-black">black</span> (黑色),
                <span class="kw-c-purple">purple</span> (紫色),
                <span class="kw-c-green">green</span> (绿色)
              </li>
            </ul>
          </div>

          <div class="help-column">
            <h4>动作 / 通知 (Action / Notification)</h4>
            <ul>
              <li>
                <span class="kw-action">show</span> (显示), <span class="kw-action">hide</span> (隐藏)
              </li>
              <li><span class="kw-notification">notify</span> (触发掉落通知弹窗)</li>
              <li><span class="kw-notification">map</span> (在游戏小地图标记红十字)</li>
            </ul>
          </div>

          <div class="help-column">
            <h4>掉落音效 (Sounds)</h4>
            <ul>
              <li>
                <span class="kw-notification">sound1</span> -
                <span class="kw-notification">sound7</span> (音效槽位1~7)
              </li>
              <li><span class="kw-notification">sound_none</span> (静音)</li>
            </ul>
          </div>
        </div>

        <p class="help-note">
          <strong><span class="kw-ethereal">eth</span></strong> — 仅匹配无形物品<br />
          <strong><span class="kw-ethereal">quest</span></strong> — 仅匹配任务物品<br />
          <strong><span class="kw-notification">stat</span></strong> — 在通知弹窗中包含物品词条属性<br
          />
          <strong><span class="kw-notification">map</span></strong> —
          在游戏内小地图掉落位置绘制红十字标记（独立于
          <span class="kw-notification">notify</span>）<br />
          <strong><span class="kw-stat">&#123;pattern&#125;</span></strong> —
          正则表达式匹配物品属性词条<br />
          <strong>规则分组：</strong>
          <code class="inline-code"
            >[<span class="kw-quality">unique</span> <span class="kw-c-gold">gold</span>
            <span class="kw-notification">notify</span>] &#123;
            <span class="kw-name">"Jordan"</span> <span class="kw-name">"Mara"</span> &#125;</code
          >
          — 大括号内每条规则继承方括号中的默认属性<br />
          <strong>默认模式：</strong> 在文件顶部单独写一行
          <code class="inline-code">hide default</code>（或
          <code class="inline-code">show default</code>）。 设置为
          <code class="inline-code">hide default</code>
          时，只有显式指定了
          <span class="kw-action">show</span> 的规则才会显示在地面上。
        </p>
      </div>
    </details>
  </div>
</section>

<style>
  .loot-filter-tab {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0; /* Important: allows flex child to shrink below content size */
    gap: var(--space-3, 12px);
    overflow: hidden;
  }

  .tab-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
    flex-wrap: wrap;
    gap: var(--space-2, 8px);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--space-3, 12px);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
  }

  .show-matches-toggle {
    display: inline-flex;
    align-items: center;
    font-size: var(--text-xs, 12px);
    color: var(--text-secondary);
  }

  .default-mode-badge {
    display: inline-flex;
    align-items: center;
    padding: 4px 10px;
    border-radius: var(--radius-full, 9999px);
    font-size: var(--text-xs, 12px);
    font-weight: 500;
    background: color-mix(in srgb, var(--text-secondary) 10%, transparent);
    color: var(--text-secondary);
    cursor: help;
    user-select: none;
  }

  .default-mode-badge.hide {
    background: color-mix(in srgb, var(--accent) 18%, transparent);
    color: var(--accent);
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-1, 4px);
    padding: 4px 10px;
    border-radius: var(--radius-full, 9999px);
    font-size: var(--text-xs, 12px);
    font-weight: 500;
    /* Neutral state */
    background: color-mix(in srgb, var(--text-secondary) 10%, transparent);
    color: var(--text-secondary);
    /* Reserve enough width so "✓ 0 rules" / "✗ 3 errors" / "—" don't shove
       the surrounding badges as their text length changes. */
    min-width: 84px;
  }

  .status-badge.valid {
    /* Use theme status colors for better contrast in light & dark themes */
    background: color-mix(in srgb, var(--status-success-text) 16%, transparent);
    color: var(--status-success-text);
  }

  .status-badge.error {
    background: color-mix(in srgb, var(--status-error-text) 18%, transparent);
    color: var(--status-error-text);
  }

  .save-status {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1, 4px);
    padding: 4px 10px;
    border-radius: var(--radius-full, 9999px);
    font-size: var(--text-xs, 12px);
    font-weight: 500;
    background: color-mix(in srgb, var(--text-secondary) 14%, transparent);
    color: var(--text-secondary);
    white-space: nowrap;
    user-select: none;
    line-height: 1.5;
    border: none;
    font-family: inherit;
  }

  .save-status.saved {
    background: color-mix(in srgb, var(--status-success-text) 16%, transparent);
    color: var(--status-success-text);
  }

  .save-status.unsaved,
  .save-status.saving {
    /* Transient neutral state — stay calm so typing doesn't feel noisy. */
    background: color-mix(in srgb, var(--text-secondary) 14%, transparent);
    color: var(--text-secondary);
  }

  .save-status.invalid {
    background: color-mix(in srgb, var(--status-error-text) 16%, transparent);
    color: var(--status-error-text);
  }

  .save-status.error {
    background: color-mix(in srgb, var(--status-error-text) 22%, transparent);
    color: var(--status-error-text);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .save-status.error:hover {
    background: color-mix(in srgb, var(--status-error-text) 30%, transparent);
  }

  .editor-container {
    flex: 1;
    min-height: 0; /* Important: allows flex child to shrink below content size */
    overflow: hidden;
  }

  .syntax-help {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .syntax-help details {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
    background: var(--bg-tertiary, #12121a);
    border: 1px solid var(--border-primary, #2a2a35);
    border-radius: var(--radius-md, 8px);
  }

  .syntax-help summary {
    padding: var(--space-2, 8px) var(--space-3, 12px);
    font-size: var(--text-sm, 13px);
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    user-select: none;
    flex-shrink: 0;
  }

  .syntax-help summary:hover {
    color: var(--text-primary);
  }

  .help-content {
    padding: var(--space-3, 12px);
    padding-top: 0;
    font-size: var(--text-sm, 13px);
    color: var(--text-secondary);
    overflow-y: auto;
    min-height: 0;
    max-height: 55vh;
  }

  .help-content p {
    margin: 0 0 var(--space-2, 8px);
  }

  .help-content code {
    display: block;
    padding: var(--space-2, 8px);
    background: var(--bg-secondary, #1a1a1f);
    border-radius: var(--radius-sm, 4px);
    font-family: var(--font-mono);
    margin-bottom: var(--space-3, 12px);
  }

  .help-columns {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: var(--space-3, 12px);
    margin-bottom: var(--space-3, 12px);
  }

  .help-column h4 {
    margin: 0 0 var(--space-1, 4px);
    font-size: var(--text-xs, 12px);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .help-column ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .help-column li {
    padding: 2px 0;
  }

  .kw-quality {
    color: #888888;
    font-weight: 600;
  }
  .kw-tier {
    color: #bd93f9;
    font-weight: 600;
  }
  .kw-ethereal {
    color: #56d4b6;
    font-weight: 600;
    font-style: italic;
  }
  .kw-socket {
    color: #6aa9ff;
    font-weight: 600;
  }
  .kw-action {
    color: #e53935;
    font-weight: 600;
  }
  .kw-notification {
    color: #c4b870;
    font-weight: 600;
  }
  .kw-name {
    color: #e09956;
    font-weight: 600;
  }
  .kw-stat {
    color: #7caa70;
    font-weight: 600;
  }
  .kw-class {
    color: #f07178;
    font-weight: 600;
  }
  .kw-level {
    color: #82aaff;
    font-weight: 600;
  }

  :global([data-theme='light']) .kw-quality {
    color: #555555;
  }
  :global([data-theme='light']) .kw-tier {
    color: #7b1fa2;
  }
  :global([data-theme='light']) .kw-ethereal {
    color: #00838f;
  }
  :global([data-theme='light']) .kw-socket {
    color: #1565c0;
  }
  :global([data-theme='light']) .kw-action {
    color: #d32f2f;
  }
  :global([data-theme='light']) .kw-notification {
    color: #ad1457;
  }
  :global([data-theme='light']) .kw-name {
    color: #b35900;
  }
  :global([data-theme='light']) .kw-stat {
    color: #116611;
  }
  :global([data-theme='light']) .kw-class {
    color: #c2185b;
  }
  :global([data-theme='light']) .kw-level {
    color: #1565c0;
  }

  /* Literal color swatches: each color name is rendered in its own color
     so the reference doubles as a color palette. Values picked to stay
     readable on both the parchment light theme and the deep-black dark
     theme; black/white/grey use mid-tones instead of pure values for
     visibility on both backgrounds. */
  .kw-c-gold {
    color: #c7a84c;
    font-weight: 600;
  }
  .kw-c-lime {
    color: #4fbf2e;
    font-weight: 600;
  }
  .kw-c-red {
    color: #d83a3a;
    font-weight: 600;
  }
  .kw-c-blue {
    color: #4a6fe0;
    font-weight: 600;
  }
  .kw-c-white {
    color: #bfbfbf;
    font-weight: 600;
  }
  .kw-c-yellow {
    color: #d6a517;
    font-weight: 600;
  }
  .kw-c-orange {
    color: #d97518;
    font-weight: 600;
  }
  .kw-c-pink {
    color: #d56bb0;
    font-weight: 600;
  }
  .kw-c-grey {
    color: #8a8a8a;
    font-weight: 600;
  }
  .kw-c-black {
    color: #303030;
    font-weight: 600;
  }
  .kw-c-purple {
    color: #8a5fb8;
    font-weight: 600;
  }
  .kw-c-green {
    color: #3d9050;
    font-weight: 600;
  }

  .inline-code {
    display: inline;
    padding: 1px 4px;
    font-family: var(--font-mono);
    background: var(--bg-secondary, #1a1a1f);
    border-radius: var(--radius-sm, 3px);
  }

  .help-note {
    margin-top: var(--space-2, 8px);
    padding: var(--space-2, 8px);
    background: var(--bg-secondary, #1a1a1f);
    border-radius: var(--radius-sm, 4px);
    font-size: var(--text-xs, 12px);
  }
</style>
