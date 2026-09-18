#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
pub mod app_paths;
mod auto_belt;
mod auto_pickup;
mod auto_potion;
mod breakpoints;
mod combat_tweaks;
mod d2types;
mod damage_stats;
mod dps;
mod game_create;
mod hotkeys;
mod injection;
mod item_extra_info;
mod item_search;
mod logger;
mod loot_history;
mod map_markers;
mod migrations;
mod monster_info;
mod monster_radar;
mod notifier;
mod offsets;
mod process;
mod profiles;
mod remote_io;
mod rules;
mod scanner_state;
mod settings;
mod shadow_tweak;
mod sounds;
mod stat_telemetry;
mod stats_panel;
mod tick_clock;
mod unique_stats_db;
#[cfg(any(target_os = "windows", target_os = "linux"))]
mod unit_stats_reader;
mod updater;

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tauri::{AppHandle, Manager, WindowEvent};

use crate::app::{EditModeState, HotkeyState};
use crate::dps::DpsMeterResetHotkeyState;
use crate::item_search::ItemSearchHotkeyState;
use crate::logger::{error as log_error, info as log_info};
use crate::loot_history::{LootHistory, LootHistoryHotkeyState};

use notifier::{ItemsDictionary, RevealHiddenState};

/// Shared state for controlling the scanner
struct AppState {
    is_scanning: Arc<AtomicBool>,
    should_auto_scan: Arc<AtomicBool>,
    /// Filter configuration shared with scanner thread
    filter_config: Arc<RwLock<Option<rules::FilterConfig>>>,
    /// When true, scanner logs per-item filter decisions (noisy; opt-in for debugging).
    verbose_filter_logging: Arc<AtomicBool>,
    /// When true, scanner reports which rule line decided each drop so the
    /// Loot Filter tab can flash it live ("show matches" mode).
    live_match_highlight: Arc<AtomicBool>,
    auto_always_show_items: Arc<AtomicBool>,
    auto_no_pickup: Arc<AtomicBool>,
    /// Driven by the reveal-hidden hotkey watcher; mirrored into the hook.
    reveal_hidden_active: Arc<AtomicBool>,
    filter_config_generation: Arc<AtomicU64>,
    // Joined on shutdown so DropScanner::drop → loot_hook.eject runs before exit.
    scanner_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    game_status: Arc<AtomicU8>,
    items_dictionary: Arc<RwLock<Option<ItemsDictionary>>>,
    /// Session loot history shared with scanner thread.
    loot_history: Arc<RwLock<LootHistory>>,
    breakpoints_polling: Arc<AtomicBool>,
    stats_polling: Arc<AtomicBool>,
    speedcalc_table: Arc<RwLock<Option<breakpoints::SpeedcalcTable>>>,
    weapon_base_catalog: Arc<RwLock<Option<breakpoints::WeaponBaseCatalog>>>,
    dps_reset_pending: Arc<AtomicBool>,
    /// Lets `refresh_game_data_caches` signal a currently-attached scanner
    /// to rebuild its class/unique/set caches without an app restart.
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    scanner_shared_state: Arc<RwLock<Option<Arc<crate::scanner_state::SharedScannerState>>>>,
    /// Consecutive `D2Injector::new` failures since the last successful
    /// attach. The Linux backend hijacks a live thread via ptrace, which
    /// only succeeds when it catches that thread outside a syscall — while
    /// the player is actively in-game (vs. idle at a menu) essentially
    /// every thread is busy most of the time, so this can fail for real,
    /// non-transient reasons. `spawn_auto_scanner` backs off its retry
    /// interval based on this counter instead of retrying every ~300ms —
    /// a live session logged 100+ failed ptrace attach attempts in under a
    /// minute against an actively-running game process, which is the
    /// leading suspect for a game crash that immediately followed.
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    attach_failure_streak: Arc<AtomicU32>,
}

const GAME_STATUS_UNKNOWN: u8 = 0;
const GAME_STATUS_INGAME: u8 = 1;
pub(crate) const GAME_STATUS_MENU: u8 = 2;

/// Pre-populate the scanner's filter config from the last-used profile on
/// startup
fn load_initial_filter_config(app: &AppHandle) -> Option<rules::FilterConfig> {
    let settings = settings::load_settings(app.clone()).ok()?;
    let name = settings.active_profile.filter(|s| !s.is_empty())?;
    let text = match profiles::load_profile(app.clone(), name.clone()) {
        Ok(t) => t,
        Err(e) => {
            log_error(&format!(
                "Startup: failed to read active profile '{}': {}",
                name, e
            ));
            return None;
        }
    };
    match rules::parse_dsl(&text) {
        Ok(cfg) => {
            log_info(&format!(
                "Startup: loaded filter config from active profile '{}' ({} rules)",
                name,
                cfg.rules.len()
            ));
            Some(cfg)
        }
        Err(errors) => {
            log_error(&format!(
                "Startup: failed to parse active profile '{}': {}",
                name,
                errors
                    .iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            None
        }
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        log_error(&format!("PANIC: {}", info));
    }));

    app::prepare_environment();

    // Enable SeDebugPrivilege so OpenProcess has the same behavior as legacy tools.
    app::enable_debug_privilege();

    // Configure WebView2 data folder for elevated processes BEFORE Tauri init
    app::setup_webview2_for_elevation();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let cached_items = notifier::load_items_cache(app.handle());
            let cached_weapon_bases = breakpoints::load_weapon_base_cache(app.handle());

            // First-run: if the settings file has never been written, drop a
            // ready-to-use Default profile and mark it active
            let dir = crate::app_paths::get_app_dir();
            let settings_path = dir.join("settings.json");
            if !settings_path.exists() {
                match profiles::seed_default_profile(app.handle()) {
                    Ok(name) => {
                        let mut s =
                            settings::load_settings(app.handle().clone()).unwrap_or_default();
                        s.active_profile = Some(name);
                        if let Err(e) = settings::save_settings(app.handle().clone(), s) {
                            log_error(&format!(
                                "First-run seed: failed to persist active profile: {}",
                                e
                            ));
                        }
                    }
                    Err(e) => log_error(&format!(
                        "First-run seed: failed to create Default profile: {}",
                        e
                    )),
                }
            }

            let initial_filter_config = load_initial_filter_config(app.handle());

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            let scanner_shared_state = Arc::new(RwLock::new(None));

            // Shared scanner state
            let state = AppState {
                is_scanning: Arc::new(AtomicBool::new(false)),
                should_auto_scan: Arc::new(AtomicBool::new(true)),
                filter_config: Arc::new(RwLock::new(initial_filter_config)),
                verbose_filter_logging: Arc::new(AtomicBool::new(false)),
                live_match_highlight: Arc::new(AtomicBool::new(false)),
                auto_always_show_items: Arc::new(AtomicBool::new(true)),
                auto_no_pickup: Arc::new(AtomicBool::new(true)),
                reveal_hidden_active: Arc::new(AtomicBool::new(false)),
                filter_config_generation: Arc::new(AtomicU64::new(0)),
                scanner_thread: Arc::new(Mutex::new(None)),
                game_status: Arc::new(AtomicU8::new(GAME_STATUS_UNKNOWN)),
                items_dictionary: Arc::new(RwLock::new(cached_items)),
                loot_history: Arc::new(RwLock::new(LootHistory::new())),
                breakpoints_polling: Arc::new(AtomicBool::new(false)),
                stats_polling: Arc::new(AtomicBool::new(false)),
                speedcalc_table: Arc::new(RwLock::new(None)),
                weapon_base_catalog: Arc::new(RwLock::new(cached_weapon_bases)),
                dps_reset_pending: Arc::new(AtomicBool::new(false)),
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                scanner_shared_state: scanner_shared_state.clone(),
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                attach_failure_streak: Arc::new(AtomicU32::new(0)),
            };
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            let attach_failure_streak = state.attach_failure_streak.clone();
            let is_scanning = state.is_scanning.clone();
            let should_auto_scan = state.should_auto_scan.clone();
            let filter_config = state.filter_config.clone();
            let verbose_filter_logging = state.verbose_filter_logging.clone();
            let live_match_highlight = state.live_match_highlight.clone();
            let auto_always_show_items = state.auto_always_show_items.clone();
            let auto_no_pickup = state.auto_no_pickup.clone();
            let reveal_hidden_active = state.reveal_hidden_active.clone();
            let filter_config_generation = state.filter_config_generation.clone();
            let scanner_thread = state.scanner_thread.clone();
            let game_status = state.game_status.clone();
            let items_dictionary = state.items_dictionary.clone();
            let loot_history = state.loot_history.clone();
            let breakpoints_polling = state.breakpoints_polling.clone();
            let stats_polling = state.stats_polling.clone();
            let speedcalc_table_for_cache = state.speedcalc_table.clone();
            let weapon_base_catalog = state.weapon_base_catalog.clone();
            let dps_reset_pending = state.dps_reset_pending.clone();
            app.manage(state);
            app.manage(item_search::MxlItemApiState::new(
                verbose_filter_logging.clone(),
            ));

            let dir = crate::app_paths::get_app_dir();
            if let Some(table) = breakpoints::load_speedcalc_cache(&dir) {
                if let Ok(mut guard) = speedcalc_table_for_cache.write() {
                    *guard = Some(table);
                }
            }

            // Initialize hotkey state
            let hotkey_state = HotkeyState::new();
            let edit_mode_state = EditModeState::new();
            let reveal_hidden_state = RevealHiddenState::new(reveal_hidden_active.clone());
            let loot_history_hotkey_state = LootHistoryHotkeyState::new();
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            let item_search_hotkey_state = ItemSearchHotkeyState::new(
                scanner_shared_state.clone(),
                verbose_filter_logging.clone(),
            );
            #[cfg(not(any(target_os = "windows", target_os = "linux")))]
            let item_search_hotkey_state =
                ItemSearchHotkeyState::new(verbose_filter_logging.clone());
            let dps_meter_reset_state = DpsMeterResetHotkeyState::new();
            let game_create_autofill_state =
                game_create::GameCreateAutofillHotkeyState::new(game_status.clone());
            let auto_belt_state = auto_belt::AutoBeltState::new();
            let auto_pickup_state = auto_pickup::AutoPickupState::new();
            let auto_pickup_hotkey_state = auto_pickup::AutoPickupHotkeyState::new();
            let auto_potion_state = auto_potion::AutoPotionState::new();
            let combat_tweaks_state = combat_tweaks::CombatTweaksState::new();
            let monster_radar_state = monster_radar::MonsterRadarState::new();
            let shadow_tweak_state = shadow_tweak::ShadowTweakState::new();
            let monster_info_state = monster_info::MonsterInfoState::new();
            let monster_info_hotkey_state = monster_info::MonsterInfoHotkeyState::new();
            let item_extra_info_state = item_extra_info::ItemExtraInfoState::new();
            let item_extra_info_hotkey_state = item_extra_info::ItemExtraInfoHotkeyState::new();

            // Load settings and start hotkey listener
            let app_handle_for_hotkeys = app.handle().clone();
            let app_handle_for_edit_mode = app.handle().clone();
            let app_handle_for_reveal = app.handle().clone();
            let app_handle_for_loot_history = app.handle().clone();
            let app_handle_for_item_search = app.handle().clone();
            let app_handle_for_dps_reset = app.handle().clone();
            let app_handle_for_auto_pickup = app.handle().clone();
            let app_handle_for_monster_info = app.handle().clone();
            let app_handle_for_item_extra_info = app.handle().clone();
            match settings::load_settings(app.handle().clone()) {
                Ok(loaded_settings) => {
                    hotkey_state
                        .start(app_handle_for_hotkeys, loaded_settings.toggle_window_hotkey);
                    edit_mode_state.start(
                        app_handle_for_edit_mode,
                        loaded_settings.edit_overlay_hotkey,
                    );
                    reveal_hidden_state
                        .start(app_handle_for_reveal, loaded_settings.reveal_hidden_hotkey);
                    loot_history_hotkey_state.start(
                        app_handle_for_loot_history,
                        loaded_settings.loot_history_hotkey,
                    );
                    item_search_hotkey_state.start(
                        app_handle_for_item_search,
                        loaded_settings.item_search_hotkey,
                    );
                    if let Some(hk) = loaded_settings.dps_meter.hotkey_reset.clone() {
                        dps_meter_reset_state.start(app_handle_for_dps_reset, hk);
                    }
                    game_create_autofill_state.start(game_create::GameCreateAutofillConfig {
                        hotkey: loaded_settings.game_create_autofill_hotkey.clone(),
                        name_prefix: loaded_settings.game_create_name_prefix.clone(),
                        password: loaded_settings.game_create_password.clone(),
                        password_prefix: loaded_settings.game_create_password_prefix.clone(),
                        password_use_prefix: loaded_settings.game_create_password_use_prefix,
                        description: loaded_settings.game_create_description.clone(),
                    });
                    auto_pickup_hotkey_state.start(
                        app_handle_for_auto_pickup.clone(),
                        loaded_settings.auto_pickup.hotkey.clone(),
                    );
                    verbose_filter_logging
                        .store(loaded_settings.verbose_filter_logging, Ordering::SeqCst);
                    auto_always_show_items
                        .store(loaded_settings.auto_always_show_items, Ordering::SeqCst);
                    auto_no_pickup.store(loaded_settings.auto_no_pickup, Ordering::SeqCst);
                    auto_belt_state
                        .enabled
                        .store(loaded_settings.auto_belt, Ordering::SeqCst);
                    auto_pickup_state
                        .enabled
                        .store(loaded_settings.auto_pickup.enabled, Ordering::SeqCst);
                    if let Ok(mut dist) = auto_pickup_state.pickup_distance.lock() {
                        *dist = loaded_settings.auto_pickup.pickup_distance;
                    }
                    let _ = auto_pickup_state.set_rules(&loaded_settings.auto_pickup.rules_text);
                    if let Ok(mut tweaks) = combat_tweaks_state.tweaks.lock() {
                        tweaks.enabled = loaded_settings.continuous_attack;
                    }
                    if let Ok(mut radar) = monster_radar_state.hook.lock() {
                        radar.enabled = loaded_settings.radar_enabled;
                        radar.show_normal = loaded_settings.radar_show_normal;
                    }
                    if let Ok(mut tweak) = shadow_tweak_state.tweak.lock() {
                        tweak.enabled = loaded_settings.remove_shadows;
                    }
                    monster_info_hotkey_state.start(
                        app_handle_for_monster_info.clone(),
                        loaded_settings.monster_info.hotkey.clone(),
                    );
                    if let Ok(mut mi) = monster_info_state.hook.lock() {
                        mi.enabled = loaded_settings.monster_info.enabled;
                        mi.show_id = loaded_settings.monster_info.show_id;
                    }
                    item_extra_info_hotkey_state.start(
                        app_handle_for_item_extra_info.clone(),
                        loaded_settings.item_extra_info.hotkey.clone(),
                    );
                    if let Ok(mut iei) = item_extra_info_state.hook.lock() {
                        iei.enabled = loaded_settings.item_extra_info.enabled;
                        iei.show_sockets_and_eth =
                            loaded_settings.item_extra_info.show_sockets_and_eth;
                    }
                    auto_potion_state.update_settings(loaded_settings.auto_potion);
                }
                Err(e) => {
                    log_error(&format!("Failed to load settings for hotkeys: {}", e));
                    // Start with default hotkeys
                    hotkey_state.start(app_handle_for_hotkeys, hotkeys::HotkeyConfig::default());
                    let defaults = settings::AppSettings::default();
                    edit_mode_state.start(app_handle_for_edit_mode, defaults.edit_overlay_hotkey);
                    reveal_hidden_state.start(app_handle_for_reveal, defaults.reveal_hidden_hotkey);
                    loot_history_hotkey_state
                        .start(app_handle_for_loot_history, defaults.loot_history_hotkey);
                    item_search_hotkey_state
                        .start(app_handle_for_item_search, defaults.item_search_hotkey);
                    let _ = app_handle_for_dps_reset;
                    game_create_autofill_state.start(game_create::GameCreateAutofillConfig {
                        hotkey: defaults.game_create_autofill_hotkey,
                        name_prefix: defaults.game_create_name_prefix,
                        password: defaults.game_create_password,
                        password_prefix: defaults.game_create_password_prefix,
                        password_use_prefix: defaults.game_create_password_use_prefix,
                        description: defaults.game_create_description,
                    });
                    auto_pickup_hotkey_state
                        .start(app_handle_for_auto_pickup, defaults.auto_pickup.hotkey);
                    auto_belt_state
                        .enabled
                        .store(defaults.auto_belt, Ordering::SeqCst);
                    auto_pickup_state
                        .enabled
                        .store(defaults.auto_pickup.enabled, Ordering::SeqCst);
                    if let Ok(mut dist) = auto_pickup_state.pickup_distance.lock() {
                        *dist = defaults.auto_pickup.pickup_distance;
                    }
                    let _ = auto_pickup_state.set_rules(&defaults.auto_pickup.rules_text);
                    if let Ok(mut tweaks) = combat_tweaks_state.tweaks.lock() {
                        tweaks.enabled = defaults.continuous_attack;
                    }
                    if let Ok(mut radar) = monster_radar_state.hook.lock() {
                        radar.enabled = defaults.radar_enabled;
                        radar.show_normal = defaults.radar_show_normal;
                    }
                    if let Ok(mut tweak) = shadow_tweak_state.tweak.lock() {
                        tweak.enabled = defaults.remove_shadows;
                    }
                    monster_info_hotkey_state
                        .start(app_handle_for_monster_info, defaults.monster_info.hotkey);
                    if let Ok(mut mi) = monster_info_state.hook.lock() {
                        mi.enabled = defaults.monster_info.enabled;
                        mi.show_id = defaults.monster_info.show_id;
                    }
                    item_extra_info_hotkey_state.start(
                        app_handle_for_item_extra_info,
                        defaults.item_extra_info.hotkey,
                    );
                    if let Ok(mut iei) = item_extra_info_state.hook.lock() {
                        iei.enabled = defaults.item_extra_info.enabled;
                        iei.show_sockets_and_eth = defaults.item_extra_info.show_sockets_and_eth;
                    }
                    auto_potion_state.update_settings(defaults.auto_potion);
                }
            }

            app.manage(hotkey_state);
            app.manage(edit_mode_state);
            app.manage(reveal_hidden_state);
            app.manage(loot_history_hotkey_state);
            app.manage(item_search_hotkey_state);
            app.manage(dps_meter_reset_state);
            app.manage(game_create_autofill_state);
            app.manage(auto_belt_state);
            app.manage(auto_pickup_state);
            app.manage(auto_pickup_hotkey_state);
            app.manage(auto_potion_state);
            app.manage(combat_tweaks_state);
            app.manage(monster_radar_state);
            app.manage(shadow_tweak_state);
            app.manage(monster_info_state);
            app.manage(monster_info_hotkey_state);
            app.manage(item_extra_info_state);
            app.manage(item_extra_info_hotkey_state);

            // Spawn auto-scanner monitor
            let app_handle = app.handle().clone();
            app::spawn_auto_scanner(
                is_scanning.clone(),
                should_auto_scan.clone(),
                filter_config.clone(),
                verbose_filter_logging.clone(),
                live_match_highlight.clone(),
                auto_always_show_items.clone(),
                auto_no_pickup.clone(),
                reveal_hidden_active.clone(),
                filter_config_generation.clone(),
                scanner_thread.clone(),
                game_status.clone(),
                items_dictionary.clone(),
                loot_history.clone(),
                breakpoints_polling.clone(),
                stats_polling.clone(),
                weapon_base_catalog.clone(),
                dps_reset_pending.clone(),
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                scanner_shared_state.clone(),
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                attach_failure_streak.clone(),
                app_handle,
            );

            // When the main window is closed, stop everything, close overlay windows
            // and terminate the application.
            if let Some(main_window) = app.get_webview_window("main") {
                let is_scanning_clone = is_scanning.clone();
                let should_auto_scan_clone = should_auto_scan.clone();
                let scanner_thread_clone = scanner_thread.clone();
                let app_handle_clone = app.handle().clone();
                main_window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { .. } = event {
                        should_auto_scan_clone.store(false, Ordering::SeqCst);
                        is_scanning_clone.store(false, Ordering::SeqCst);

                        if let Some(overlay) = app_handle_clone.get_webview_window("overlay") {
                            if let Err(e) = overlay.close() {
                                log_error(&format!(
                                    "Failed to close overlay window on main close: {}",
                                    e
                                ));
                            }
                        }
                        let handle_opt = scanner_thread_clone.lock().unwrap().take();
                        let ah = app_handle_clone.clone();
                        thread::spawn(move || {
                            let watchdog_fired = Arc::new(AtomicBool::new(false));
                            let wf_w = watchdog_fired.clone();
                            let ah_w = ah.clone();
                            thread::spawn(move || {
                                thread::sleep(Duration::from_millis(2500));
                                wf_w.store(true, Ordering::SeqCst);
                                log_error("scanner join watchdog fired after 2.5s; exiting");
                                ah_w.exit(0);
                            });
                            if let Some(h) = handle_opt {
                                let _ = h.join();
                            }
                            if !watchdog_fired.load(Ordering::SeqCst) {
                                ah.exit(0);
                            }
                        });
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::open_devtools,
            app::get_scanner_status,
            app::get_game_status,
            notifier::get_items_dictionary,
            loot_history::get_loot_history,
            loot_history::clear_loot_history,
            notifier::set_filter_config,
            notifier::set_verbose_filter_logging,
            notifier::set_live_match_highlight,
            notifier::set_auto_always_show_items,
            notifier::set_auto_no_pickup,
            breakpoints::set_breakpoints_polling,
            stats_panel::set_stats_polling,
            breakpoints::get_speedcalc_data,
            breakpoints::refresh_speedcalc_data,
            unique_stats_db::check_unique_stats_db_update,
            unique_stats_db::download_unique_stats_db,
            breakpoints::get_weapon_base_catalog,
            app::refresh_game_data_caches,
            app::sync_overlay_with_game,
            app::set_overlay_interactive,
            app::set_overlay_edit_mode,
            rules::parse_filter_dsl,
            rules::validate_filter_dsl,
            rules::explain_filter_line,
            rules::get_item_filter_action,
            settings::load_settings,
            settings::save_settings,
            settings::get_window_state,
            settings::save_window_state,
            sounds::import_sound_file,
            sounds::delete_sound_file,
            sounds::play_audio_bytes_native,
            sounds::should_use_native_audio,
            app::update_hotkey,
            app::update_edit_mode_hotkey,
            notifier::update_reveal_hidden_hotkey,
            loot_history::update_loot_history_hotkey,
            item_search::update_item_search_hotkey,
            dps::update_dps_meter_reset_hotkey,
            game_create::update_game_create_autofill_hotkey,
            dps::reset_dps_session,
            item_search::search_mxl_items,
            profiles::list_profiles,
            profiles::load_profile,
            profiles::save_profile,
            profiles::delete_profile,
            profiles::rename_profile,
            profiles::duplicate_profile,
            profiles::create_profile,
            updater::check_for_updates,
            updater::start_update,
            updater::restart_app,
            app::open_app_folder,
            app::open_external_url,
            app::get_changelog,
            app::get_app_config_dir,
            monster_radar::toggle_monster_radar,
            monster_radar::set_radar_show_normal,
            combat_tweaks::toggle_continuous_attack,
            auto_belt::toggle_auto_belt,
            shadow_tweak::toggle_remove_shadows,
            auto_potion::update_auto_potion_settings,
            auto_pickup::toggle_auto_pickup,
            auto_pickup::update_auto_pickup_rules,
            auto_pickup::update_auto_pickup_hotkey,
            monster_info::toggle_monster_info,
            monster_info::set_monster_info_show_id,
            monster_info::update_monster_info_hotkey,
            item_extra_info::toggle_item_extra_info,
            item_extra_info::set_item_extra_info_show_sockets_and_eth,
            item_extra_info::update_item_extra_info_hotkey
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
