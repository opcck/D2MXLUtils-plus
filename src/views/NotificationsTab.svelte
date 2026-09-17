<script lang="ts">
  import { settingsStore } from '../stores';
  import { Toggle, Notification } from '../components';

  const previewPlain = {
    unit_id: 0,
    class: 0,
    quality: 'Unique',
    name: "Tyrael's Might SU (泰瑞尔之力)",
    base_name: 'Sacred Armor (神圣战甲)',
    stats: '',
    is_ethereal: true,
    is_identified: true,
    unique_kind: 'su' as const,
    filter: { display_stats: false },
  };

  const previewWithStats = {
    unit_id: 1,
    class: 0,
    quality: 'Unique',
    name: "Tyrael's Might SU (泰瑞尔之力)",
    base_name: 'Sacred Armor (神圣战甲)',
    stats: [
      '无法破坏',
      '+150% 强化防御',
      '+20% 快速奔跑/行走',
      '装备需求 -100%',
      '+2 所有技能',
    ].join('\n'),
    is_ethereal: true,
    is_identified: true,
    unique_kind: 'su' as const,
    filter: { display_stats: true },
  };

  const previewWithMatch = {
    unit_id: 2,
    class: 0,
    quality: 'Rare',
    name: 'Rune Turn (符文指环)',
    base_name: 'Sacred Ring (神圣戒指)',
    stats: ['+15% 快速施法速度', '+1 所有技能', '+25 法力'].join('\n'),
    is_ethereal: false,
    is_identified: true,
    unique_kind: null,
    filter: { display_stats: true, matched_stat_lines: [0, 1] },
  };

  let duration = $derived(settingsStore.settings.notificationDuration);
  let fontSize = $derived(settingsStore.settings.notificationFontSize);
  let opacity = $derived(settingsStore.settings.notificationOpacity);
  let compactName = $derived(settingsStore.settings.compactName);
  let showOnlyMatchedStats = $derived(settingsStore.settings.showOnlyMatchedStats);

  function setDuration(value: number) {
    const clamped = Math.max(1000, Math.min(30000, value));
    settingsStore.set('notificationDuration', clamped);
  }

  function setFontSize(value: number) {
    const clamped = Math.max(10, Math.min(36, value));
    settingsStore.set('notificationFontSize', clamped);
  }

  function setOpacity(value: number) {
    const clamped = Math.max(0, Math.min(1, value));
    settingsStore.set('notificationOpacity', clamped);
  }

  function setCompactName(value: boolean) {
    settingsStore.set('compactName', value);
  }

  function setShowOnlyMatchedStats(value: boolean) {
    settingsStore.set('showOnlyMatchedStats', value);
  }
</script>

<section class="tab-content">
  <div class="settings-section">
    <h2 class="section-title">通知设置</h2>
    <p class="section-description">自定义游戏悬浮窗中掉落通知的显示样式。</p>

    <div class="settings-grid">
      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label" for="duration">停留时长</label>
          <span class="setting-hint">通知在屏幕上停留显示的时间 (1-30 秒)</span>
        </div>
        <div class="setting-control">
          <input
            type="range"
            id="duration-slider"
            min="1000"
            max="30000"
            step="500"
            value={duration}
            oninput={(e) => setDuration(parseInt(e.currentTarget.value))}
            class="slider"
          />
          <span class="setting-value">{(duration / 1000).toFixed(1)}s</span>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label" for="font-size">尺寸缩放</label>
          <span class="setting-hint">缩放通知面板整体大小 (10-36 像素)</span>
        </div>
        <div class="setting-control">
          <input
            type="range"
            id="font-size-slider"
            min="10"
            max="36"
            step="1"
            value={fontSize}
            oninput={(e) => setFontSize(parseInt(e.currentTarget.value))}
            class="slider"
          />
          <span class="setting-value">{fontSize}px</span>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label" for="opacity">背景不透明度</label>
          <span class="setting-hint">通知弹窗背景的不透明度 (0-100%)</span>
        </div>
        <div class="setting-control">
          <input
            type="range"
            id="opacity-slider"
            min="0"
            max="1"
            step="0.05"
            value={opacity}
            oninput={(e) => setOpacity(parseFloat(e.currentTarget.value))}
            class="slider"
          />
          <span class="setting-value">{Math.round(opacity * 100)}%</span>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label" for="compact-name">精简物品名称</label>
          <span class="setting-hint">
            对套装及各级暗金 (TU/SU/SSU/SSSU) 隐藏专属名称行，仅显示底模类型。带有 <code>stat</code> 标记的规则会忽略此项。
          </span>
        </div>
        <div class="setting-control">
          <Toggle id="compact-name" checked={compactName} onchange={setCompactName} />
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <label class="setting-label" for="show-only-matched-stats">仅显示命中词条</label>
          <span class="setting-hint">
            对于带有 <code>&#123;stat 属性正则&#125;</code> 的规则，仅显示匹配成功的词条行。多行正则匹配将退回显示全部属性。
          </span>
        </div>
        <div class="setting-control">
          <Toggle
            id="show-only-matched-stats"
            checked={showOnlyMatchedStats}
            onchange={setShowOnlyMatchedStats}
          />
        </div>
      </div>
    </div>
  </div>

  <div class="preview-section">
    <h3 class="preview-title">实时预览效果</h3>
    <div class="preview-container">
      <Notification item={previewPlain} {fontSize} {opacity} {compactName} {showOnlyMatchedStats} />
      <Notification
        item={previewWithStats}
        {fontSize}
        {opacity}
        {compactName}
        {showOnlyMatchedStats}
      />
      <Notification
        item={previewWithMatch}
        {fontSize}
        {opacity}
        {compactName}
        {showOnlyMatchedStats}
      />
    </div>
  </div>
</section>

<style>
  .tab-content {
    padding: var(--space-4);
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(340px, 520px);
    gap: var(--space-5);
    align-items: start;
    scrollbar-gutter: stable;
  }

  @media (max-width: 820px) {
    .tab-content {
      grid-template-columns: 1fr;
    }
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .section-title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .section-description {
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin: 0;
  }

  .settings-grid {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .setting-label {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-primary);
  }

  .setting-hint {
    font-size: var(--text-xs);
    color: var(--text-muted);
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

  .preview-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    position: sticky;
    top: var(--space-4);
  }

  .preview-title {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-muted);
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .preview-container {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-4);
    background: repeating-linear-gradient(
      45deg,
      var(--bg-tertiary),
      var(--bg-tertiary) 10px,
      var(--bg-secondary) 10px,
      var(--bg-secondary) 20px
    );
    border-radius: var(--radius-md);
    min-height: 100px;
  }

  .setting-hint code {
    background: var(--bg-tertiary);
    padding: 0 4px;
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 0.95em;
  }
</style>
