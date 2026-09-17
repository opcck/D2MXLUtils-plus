<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Button, HotkeyInput, Toggle } from '../components';
  import { settingsStore, updaterStore, uniqueStatsDbStore, type HotkeyConfig } from '../stores';

  let verboseFilterLogging = $derived(settingsStore.settings.verboseFilterLogging);
  let liveMatchHighlightDurationMs = $derived(settingsStore.settings.liveMatchHighlightDurationMs);
  let autoAlwaysShowItems = $derived(settingsStore.settings.autoAlwaysShowItems);
  let autoNoPickup = $derived(settingsStore.settings.autoNoPickup);
  let showItemsHiddenIndicator = $derived(settingsStore.settings.showItemsHiddenIndicator);
  let dpsMeterEnabled = $derived(settingsStore.settings.dpsMeter?.enabled ?? false);
  let gameCreateNamePrefix = $derived(settingsStore.settings.gameCreateNamePrefix);
  let gameCreatePassword = $derived(settingsStore.settings.gameCreatePassword);
  let gameCreatePasswordPrefix = $derived(settingsStore.settings.gameCreatePasswordPrefix);
  let gameCreatePasswordUsePrefix = $derived(settingsStore.settings.gameCreatePasswordUsePrefix);
  let gameCreateDescription = $derived(settingsStore.settings.gameCreateDescription);
  let radarEnabled = $derived(settingsStore.settings.radarEnabled);
  let radarShowNormal = $derived(settingsStore.settings.radarShowNormal);
  let continuousAttack = $derived(settingsStore.settings.continuousAttack);
  let autoBelt = $derived(settingsStore.settings.autoBelt);
  let removeShadows = $derived(settingsStore.settings.removeShadows);

  const UNBOUND_HOTKEY: HotkeyConfig = { keyCode: 0, modifiers: 0, display: 'None' };

  type HotkeyId =
    | 'toggleWindow'
    | 'editOverlay'
    | 'revealHidden'
    | 'lootHistory'
    | 'itemSearch'
    | 'dpsMeterReset'
    | 'gameCreateAutofill';
  interface HotkeyRow {
    id: HotkeyId;
    label: string;
    hint: string;
    setter: (h: HotkeyConfig) => void;
  }
  const HOTKEY_ROWS: readonly HotkeyRow[] = [
    {
      id: 'toggleWindow',
      label: '切换窗口',
      hint: '在游戏上方显示/隐藏主窗口',
      setter: (h) => settingsStore.setToggleWindowHotkey(h),
    },
    {
      id: 'editOverlay',
      label: '调整界面布局',
      hint: '按住可拖拽悬浮窗各组件的锚点位置',
      setter: (h) => settingsStore.setEditOverlayHotkey(h),
    },
    {
      id: 'revealHidden',
      label: '强制显示隐藏物品',
      hint: '按住可在地面显示所有物品（临时忽略 `hide` 过滤规则）',
      setter: (h) => settingsStore.setRevealHiddenHotkey(h),
    },
    {
      id: 'lootHistory',
      label: '掉落历史',
      hint: '打开/关闭游戏内掉落日志面板（本次游戏会话的掉落）',
      setter: (h) => settingsStore.setLootHistoryHotkey(h),
    },
    {
      id: 'itemSearch',
      label: '物品检索',
      hint: '打开游戏内 MXL 物品数据库搜索悬浮窗',
      setter: (h) => settingsStore.setItemSearchHotkey(h),
    },
  ];

  const DPS_HOTKEY_ROWS: readonly HotkeyRow[] = [
    {
      id: 'dpsMeterReset',
      label: '重置 DPS 统计',
      hint: '清空当前的秒伤统计数据',
      setter: (h) => settingsStore.setDpsMeterResetHotkey(h),
    },
  ];

  const GAME_CREATE_HOTKEY_ROWS: readonly HotkeyRow[] = [
    {
      id: 'gameCreateAutofill',
      label: '自动建房快速输入',
      hint: '先点击建房界面游戏名称输入框，再按此键 — 自动填入名称、Tab、密码与描述（若已设置）',
      setter: (h) => settingsStore.setGameCreateAutofillHotkey(h),
    },
  ];

  const HOTKEY_GETTERS: Record<HotkeyId, () => HotkeyConfig> = {
    toggleWindow: () => settingsStore.settings.toggleWindowHotkey,
    editOverlay: () => settingsStore.settings.editOverlayHotkey,
    revealHidden: () => settingsStore.settings.revealHiddenHotkey,
    lootHistory: () => settingsStore.settings.lootHistoryHotkey,
    itemSearch: () => settingsStore.settings.itemSearchHotkey,
    dpsMeterReset: () => settingsStore.settings.dpsMeter?.hotkeyReset ?? UNBOUND_HOTKEY,
    gameCreateAutofill: () => settingsStore.settings.gameCreateAutofillHotkey,
  };
  let hotkeyValues = $derived(
    Object.fromEntries(
      (Object.keys(HOTKEY_GETTERS) as HotkeyId[]).map((id) => [id, HOTKEY_GETTERS[id]()]),
    ) as Record<HotkeyId, HotkeyConfig>,
  );

  function handleDpsMeterEnabledChange(enabled: boolean) {
    settingsStore.setDpsMeterEnabled(enabled);
  }

  let updaterState = $derived(updaterStore.state);
  let checkDisabled = $derived(
    updaterState.kind === 'checking' ||
      updaterState.kind === 'downloading' ||
      updaterState.kind === 'ready',
  );

  function formatBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
    return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  }

  function updateStatusText(): string {
    const s = updaterState;
    switch (s.kind) {
      case 'idle':
        return '';
      case 'checking':
        return '正在检查更新…';
      case 'up_to_date':
        return '已是最新版本';
      case 'available':
        return `发现新版本 v${s.latest} — 请点击右上角按钮更新`;
      case 'downloading':
        return `正在下载 ${formatBytes(s.downloaded)}`;
      case 'ready':
        return '准备就绪，点击右上角“重启”以安装更新';
      case 'error':
        return s.phase === 'install'
          ? '更新失败 — 可能是安全软件拦截。请使用右上角“手动下载”按钮。'
          : '检查更新失败，请检查网络连接。';
    }
  }

  let uniqueDbState = $derived(uniqueStatsDbStore.state);
  let uniqueDbButtonDisabled = $derived(
    uniqueDbState.kind === 'checking' || uniqueDbState.kind === 'downloading',
  );

  function uniqueDbStatusText(): string {
    const s = uniqueDbState;
    switch (s.kind) {
      case 'idle':
        return '';
      case 'checking':
        return '正在检查…';
      case 'not_downloaded':
        return '尚未下载 — 点击“下载”以启用暗金/套装的变量范围显示';
      case 'up_to_date':
        return '数据库已是最新';
      case 'available':
        return '发现数据库更新 — 请点击“下载”';
      case 'downloading':
        return '正在下载…';
      case 'downloaded':
        return '下载完成 — 重启软件后生效';
      case 'error':
        return `失败: ${s.message}`;
    }
  }

  function uniqueDbButtonLabel(): string {
    const s = uniqueDbState;
    return s.kind === 'not_downloaded' || s.kind === 'available' ? '下载' : '检查更新';
  }

  function handleUniqueDbButtonClick() {
    if (uniqueDbState.kind === 'not_downloaded' || uniqueDbState.kind === 'available') {
      uniqueStatsDbStore.download();
    } else {
      uniqueStatsDbStore.check();
    }
  }

  const UNBOUND: HotkeyConfig = { keyCode: 0, modifiers: 0, display: 'None' };

  function sameChord(a: HotkeyConfig, b: HotkeyConfig): boolean {
    return a.keyCode === b.keyCode && a.modifiers === b.modifiers;
  }

  function isBound(h: HotkeyConfig): boolean {
    return h.keyCode !== 0 || h.modifiers !== 0;
  }

  function handleHotkeyChange(id: HotkeyId, hotkey: HotkeyConfig) {
    const allRows = [...HOTKEY_ROWS, ...DPS_HOTKEY_ROWS, ...GAME_CREATE_HOTKEY_ROWS];
    if (isBound(hotkey)) {
      for (const row of allRows) {
        if (row.id === id) continue;
        if (sameChord(hotkeyValues[row.id], hotkey)) {
          row.setter(UNBOUND);
        }
      }
    }
    allRows.find((r) => r.id === id)!.setter(hotkey);
  }

  function handleCheckForUpdates() {
    updaterStore.check(true);
  }

  let refreshGameDataStatus = $state<'idle' | 'refreshing' | 'done' | 'error'>('idle');

  async function handleRefreshGameData() {
    refreshGameDataStatus = 'refreshing';
    try {
      await invoke('refresh_game_data_caches');
      refreshGameDataStatus = 'done';
    } catch (err) {
      console.error('Failed to refresh game data caches:', err);
      refreshGameDataStatus = 'error';
    }
  }

  function refreshGameDataStatusText(): string {
    switch (refreshGameDataStatus) {
      case 'idle':
        return '';
      case 'refreshing':
        return '正在刷新…';
      case 'done':
        return '完成 — 正在从游戏内存实时重新构建（若游戏未运行，则在下次连接时生效）。';
      case 'error':
        return '刷新失败 — 请查看 d2mxlutils.log 日志。';
    }
  }

  async function handleOpenAppFolder() {
    try {
      await invoke('open_app_folder');
    } catch (err) {
      console.error('Failed to open app folder:', err);
    }
  }

  function handleVerboseLoggingChange(enabled: boolean) {
    settingsStore.setVerboseFilterLogging(enabled);
  }

  function setLiveMatchHighlightDuration(value: number) {
    const clamped = Math.max(200, Math.min(5000, value));
    settingsStore.set('liveMatchHighlightDurationMs', clamped);
  }

  function handleAutoAlwaysShowItemsChange(enabled: boolean) {
    settingsStore.setAutoAlwaysShowItems(enabled);
  }

  function handleAutoNoPickupChange(enabled: boolean) {
    settingsStore.setAutoNoPickup(enabled);
  }

  function handleShowItemsHiddenIndicatorChange(enabled: boolean) {
    settingsStore.set('showItemsHiddenIndicator', enabled);
  }

  function handleGameCreateNamePrefixInput(e: Event) {
    settingsStore.setGameCreateNamePrefix((e.target as HTMLInputElement).value);
  }

  function handleGameCreatePasswordInput(e: Event) {
    settingsStore.setGameCreatePassword((e.target as HTMLInputElement).value);
  }

  function handleGameCreatePasswordPrefixInput(e: Event) {
    settingsStore.setGameCreatePasswordPrefix((e.target as HTMLInputElement).value);
  }

  function handleGameCreatePasswordUsePrefixChange(enabled: boolean) {
    settingsStore.setGameCreatePasswordUsePrefix(enabled);
  }

  function handleGameCreateDescriptionInput(e: Event) {
    settingsStore.setGameCreateDescription((e.target as HTMLInputElement).value);
  }

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

  let showChangelog = $state(false);
  let changelogHtml = $state('');

  async function handleOpenChangelog() {
    try {
      const md: string = await invoke('get_changelog');
      changelogHtml = renderChangelog(md);
      showChangelog = true;
    } catch (err) {
      console.error('Failed to load changelog:', err);
    }
  }

  function renderChangelog(md: string): string {
    const lines = md.split('\n');
    const out: string[] = [];
    let skipSection = false;
    let inVersion = false;

    for (const line of lines) {
      if (line.startsWith('# ') && !line.startsWith('## ')) continue;

      if (line.startsWith('## ')) {
        if (inVersion) out.push('</section>');
        inVersion = true;
        skipSection = false;
        out.push(`<section class="cl-version">`);
        out.push(`<h2>${line.slice(3)}</h2>`);
        continue;
      }

      if (line.startsWith('### ')) {
        const heading = line.slice(4);
        skipSection = heading === 'Other';
        if (!skipSection) out.push(`<h3>${heading}</h3>`);
        continue;
      }

      if (skipSection) continue;

      if (line.startsWith('- ')) {
        out.push(`<div class="cl-entry">${formatEntry(line.slice(2))}</div>`);
        continue;
      }
    }
    if (inVersion) out.push('</section>');
    return out.join('\n');
  }

  function formatEntry(text: string): string {
    text = text.replace(
      /^(?:Feat|Fix|Refactor|Perf|Chore|Docs|Style|Build|Ci|Test)(\([^)]+\)):\s*/i,
      (_, scope) => {
        return `<span class="cl-scope">${scope.slice(1, -1)}</span>`;
      },
    );
    text = text.replace(
      /\(([0-9a-f]{7})\)$/,
      '<a class="cl-hash" href="https://github.com/synonymouse/D2MXLUtils/commit/$1" target="_blank">$1</a>',
    );
    return text;
  }
</script>

<section class="tab-content">
  <div class="settings-section">
    <h2 class="section-title">快捷键设置</h2>

    {#each HOTKEY_ROWS as row (row.id)}
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{row.label}</span>
          <span class="setting-hint">{@html row.hint.replace(/`([^`]+)`/g, '<code>$1</code>')}</span
          >
        </div>
        <HotkeyInput value={hotkeyValues[row.id]} onchange={(h) => handleHotkeyChange(row.id, h)} />
      </div>
    {/each}
  </div>

  <div class="settings-section">
    <h2 class="section-title">秒伤统计 (DPS Meter)</h2>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">显示 DPS 悬浮窗</span>
      </div>
      <Toggle checked={dpsMeterEnabled} onchange={handleDpsMeterEnabledChange} />
    </div>

    {#each DPS_HOTKEY_ROWS as row (row.id)}
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{row.label}</span>
          <span class="setting-hint">{row.hint}</span>
        </div>
        <HotkeyInput value={hotkeyValues[row.id]} onchange={(h) => handleHotkeyChange(row.id, h)} />
      </div>
    {/each}
  </div>

  <div class="settings-section">
    <h2 class="section-title">自动建房填充</h2>

    {#each GAME_CREATE_HOTKEY_ROWS as row (row.id)}
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{row.label}</span>
          <span class="setting-hint">{row.hint}</span>
        </div>
        <HotkeyInput value={hotkeyValues[row.id]} onchange={(h) => handleHotkeyChange(row.id, h)} />
      </div>
    {/each}

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">游戏名称前缀</span>
        <span class="setting-hint">房间名称 = 前缀 + 自动递增序号（重启软件后不保留）</span>
      </div>
      <input
        type="text"
        class="text-input"
        value={gameCreateNamePrefix}
        oninput={handleGameCreateNamePrefixInput}
        placeholder="例如：MyGame"
      />
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">密码也自动递增</span>
        <span class="setting-hint">使用与本次游戏名称相同的序号</span>
      </div>
      <Toggle
        checked={gameCreatePasswordUsePrefix}
        onchange={handleGameCreatePasswordUsePrefixChange}
      />
    </div>

    {#if gameCreatePasswordUsePrefix}
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">密码前缀</span>
        </div>
        <input
          type="text"
          class="text-input"
          value={gameCreatePasswordPrefix}
          oninput={handleGameCreatePasswordPrefixInput}
          placeholder="例如：pw"
        />
      </div>
    {:else}
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">固定密码</span>
        </div>
        <input
          type="text"
          class="text-input"
          value={gameCreatePassword}
          oninput={handleGameCreatePasswordInput}
        />
      </div>
    {/if}

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">房间描述</span>
        <span class="setting-hint">留空则保持默认不填</span>
      </div>
      <input
        type="text"
        class="text-input"
        value={gameCreateDescription}
        oninput={handleGameCreateDescriptionInput}
      />
    </div>
  </div>

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

  <div class="settings-section">
    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">新建游戏时自动开启地面物品常显 (Alt)</span>
        <span class="setting-hint">无需按 Alt 键，自动在地面显示掉落物品。</span>
      </div>
      <Toggle checked={autoAlwaysShowItems} onchange={handleAutoAlwaysShowItemsChange} />
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">显示“物品已隐藏”提醒标识</span>
        <span class="setting-hint">当物品高亮关闭时，在屏幕左上角提示按 Alt。</span>
      </div>
      <Toggle checked={showItemsHiddenIndicator} onchange={handleShowItemsHiddenIndicatorChange} />
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">新建游戏时自动开启 /nopickup 命令</span>
        <span class="setting-hint">防止鼠标意外捡起地面垃圾物品；修改后游戏内立即生效。</span>
      </div>
      <Toggle checked={autoNoPickup} onchange={handleAutoNoPickupChange} />
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">详细过滤日志记录</span>
        <span class="setting-hint"
          >将每件物品的过滤判定过程记录到 d2mxlutils.log，用于调试规则。</span
        >
      </div>
      <Toggle checked={verboseFilterLogging} onchange={handleVerboseLoggingChange} />
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">规则实时匹配高亮持续时间</span>
        <span class="setting-hint"
          >在“掉落过滤”标签页开启“显示实时匹配”时，命中规则行的闪烁持续时间 (0.2-5秒)。</span
        >
      </div>
      <div class="setting-control">
        <input
          type="range"
          id="live-match-highlight-duration-slider"
          min="200"
          max="5000"
          step="100"
          value={liveMatchHighlightDurationMs}
          oninput={(e) => setLiveMatchHighlightDuration(parseInt(e.currentTarget.value))}
          class="slider"
        />
        <span class="setting-value">{(liveMatchHighlightDurationMs / 1000).toFixed(1)}s</span>
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">应用数据目录</span>
        <span class="setting-hint">存放配置文件、过滤规则、运行日志</span>
      </div>
      <div class="update-control">
        <Button variant="secondary" size="sm" onclick={handleOpenAppFolder}>打开目录</Button>
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">当前版本</span>
        <span class="setting-hint">
          v{__APP_VERSION__}
          <button type="button" class="link-button" onclick={handleOpenChangelog}>更新日志</button>
        </span>
      </div>
      <div class="update-control">
        <Button
          variant="secondary"
          size="sm"
          disabled={checkDisabled}
          onclick={handleCheckForUpdates}
        >
          检查更新
        </Button>
      </div>
    </div>

    {#if updateStatusText()}
      <div class="update-status" class:is-error={updaterState.kind === 'error'}>
        {updateStatusText()}
      </div>
    {/if}

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">暗金/套装变量属性范围数据库</span>
        <span class="setting-hint">
          为暗金和套装物品提供变量属性范围说明。下载离线数据库后无需客户端重复爬取 API。
        </span>
      </div>
      <div class="update-control">
        <Button
          variant="secondary"
          size="sm"
          disabled={uniqueDbButtonDisabled}
          onclick={handleUniqueDbButtonClick}
        >
          {uniqueDbButtonLabel()}
        </Button>
      </div>
    </div>

    {#if uniqueDbStatusText()}
      <div class="update-status" class:is-error={uniqueDbState.kind === 'error'}>
        {uniqueDbStatusText()}
      </div>
    {/if}

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">刷新游戏数据缓存</span>
        <span class="setting-hint">
          从游戏内存重新构建物品名、暗金/套装名与武器底模。在 MXL
          内容补丁后若掉落名称异常可使用此项。无需重启软件。
        </span>
      </div>
      <div class="update-control">
        <Button
          variant="secondary"
          size="sm"
          disabled={refreshGameDataStatus === 'refreshing'}
          onclick={handleRefreshGameData}
        >
          刷新
        </Button>
      </div>
    </div>

    {#if refreshGameDataStatusText()}
      <div class="update-status" class:is-error={refreshGameDataStatus === 'error'}>
        {refreshGameDataStatusText()}
      </div>
    {/if}
  </div>
</section>

{#if showChangelog}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="changelog-backdrop"
    role="dialog"
    aria-modal="true"
    onkeydown={(e) => e.key === 'Escape' && (showChangelog = false)}
    onclick={() => (showChangelog = false)}
  >
    <div class="changelog-modal" onclick={(e) => e.stopPropagation()}>
      <div class="changelog-header">
        <h2 class="changelog-title">更新日志</h2>
        <button type="button" class="changelog-close" onclick={() => (showChangelog = false)}
          >&times;</button
        >
      </div>
      <div
        class="changelog-body"
        onclick={(e) => {
          const a = (e.target as HTMLElement).closest('a.cl-hash');
          if (a) {
            e.preventDefault();
            invoke('open_external_url', { url: (a as HTMLAnchorElement).href });
          }
        }}
      >
        {@html changelogHtml}
      </div>
    </div>
  </div>
{/if}

<style>
  .text-input {
    padding: var(--space-1) var(--space-2);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font: inherit;
    min-width: 180px;
  }

  .update-control {
    display: flex;
    align-items: center;
  }

  .setting-control {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .slider {
    width: 160px;
    height: 6px;
    appearance: none;
    background: var(--bg-tertiary);
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
    transition: transform 0.1s ease;
  }

  .slider::-webkit-slider-thumb:hover {
    transform: scale(1.1);
  }

  .slider::-moz-range-thumb {
    width: 16px;
    height: 16px;
    background: var(--accent-primary);
    border: none;
    border-radius: var(--radius-full);
    cursor: pointer;
  }

  .setting-value {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--text-primary);
    min-width: 50px;
    text-align: right;
  }

  .update-status {
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    color: var(--text-secondary, var(--text-primary));
  }

  .update-status.is-error {
    color: var(--status-error-text);
  }

  .link-button {
    margin-left: var(--space-2);
    padding: 0;
    background: none;
    border: none;
    color: var(--accent-primary);
    font: inherit;
    cursor: pointer;
    text-decoration: underline;
  }

  .link-button:hover {
    opacity: 0.85;
  }

  .changelog-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.6);
  }

  .changelog-modal {
    display: flex;
    flex-direction: column;
    width: 92%;
    max-width: 640px;
    max-height: 85vh;
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
  }

  .changelog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border-primary);
  }

  .changelog-title {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--text-primary);
  }

  .changelog-close {
    padding: 0;
    background: none;
    border: none;
    font-size: var(--text-2xl);
    line-height: 1;
    color: var(--text-muted);
    cursor: pointer;
  }

  .changelog-close:hover {
    color: var(--text-primary);
  }

  .changelog-body {
    padding: var(--space-3) var(--space-4);
    overflow-y: auto;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    line-height: 1.6;
  }

  .changelog-body :global(.cl-version) {
    padding-bottom: var(--space-3);
    margin-bottom: var(--space-3);
    border-bottom: 1px solid var(--border-primary);
  }

  .changelog-body :global(.cl-version:last-child) {
    border-bottom: none;
    margin-bottom: 0;
  }

  .changelog-body :global(h2) {
    margin: 0 0 var(--space-2);
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--accent-primary);
  }

  .changelog-body :global(h3) {
    margin: var(--space-2) 0 var(--space-1);
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .changelog-body :global(.cl-entry) {
    padding: 1px 0 1px var(--space-3);
    color: var(--text-primary);
  }

  .changelog-body :global(.cl-scope) {
    font-family: var(--font-mono);
    font-size: 0.9em;
    color: var(--text-secondary);
    opacity: 0.85;
  }

  .changelog-body :global(.cl-scope::after) {
    content: ':  ';
  }

  .changelog-body :global(.cl-hash) {
    font-family: var(--font-mono);
    font-size: 0.85em;
    color: var(--text-muted);
    text-decoration: underline;
    opacity: 0.5;
    margin-left: var(--space-1);
    cursor: pointer;
  }

  .changelog-body :global(.cl-hash:hover) {
    opacity: 1;
  }
</style>
