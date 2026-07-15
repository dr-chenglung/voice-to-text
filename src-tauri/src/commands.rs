//! 設定視窗用的 IPC commands（規格新增功能）。
//! API key 不可明文回傳前端：`get_config` 只回報「是否已設定」，`save_config` 的
//! `api_key` 留空代表「不變更」，避免明文塞進 DOM／devtools 可見。

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::config::Config;
use crate::history::{self, HistoryEntry};

#[derive(Serialize)]
pub struct ConfigView {
    pub has_stt_api_key: bool,
    pub stt_api_url: String,
    pub stt_model: String,

    pub has_llm_api_key: bool,
    pub llm_api_url: String,
    pub llm_model: String,

    pub enable_correction: bool,
    pub hotkey: String,

    pub vocabulary: String,
    pub enable_formatting: bool,
}

#[derive(Deserialize)]
pub struct ConfigUpdate {
    /// 留空字串＝不變更現有 API key。
    pub stt_api_key: String,
    pub stt_api_url: String,
    pub stt_model: String,

    /// 留空字串＝不變更現有 API key。
    pub llm_api_key: String,
    pub llm_api_url: String,
    pub llm_model: String,

    pub enable_correction: bool,
    pub hotkey: String,

    pub vocabulary: String,
    pub enable_formatting: bool,
}

#[tauri::command]
pub fn get_config(cfg: State<'_, Arc<Mutex<Config>>>) -> ConfigView {
    let c = cfg.lock().unwrap();
    ConfigView {
        has_stt_api_key: !c.stt_api_key.trim().is_empty(),
        stt_api_url: c.stt_api_url.clone(),
        stt_model: c.stt_model.clone(),
        has_llm_api_key: !c.llm_api_key.trim().is_empty(),
        llm_api_url: c.llm_api_url.clone(),
        llm_model: c.llm_model.clone(),
        enable_correction: c.enable_correction,
        hotkey: c.hotkey.clone(),
        vocabulary: c.vocabulary.clone(),
        enable_formatting: c.enable_formatting,
    }
}

#[tauri::command]
pub fn save_config(
    app: AppHandle,
    cfg: State<'_, Arc<Mutex<Config>>>,
    update: ConfigUpdate,
) -> Result<(), String> {
    let snapshot = {
        let mut c = cfg.lock().unwrap();
        if !update.stt_api_key.trim().is_empty() {
            c.stt_api_key = update.stt_api_key.trim().to_string();
        }
        c.stt_api_url = update.stt_api_url;
        c.stt_model = update.stt_model;
        if !update.llm_api_key.trim().is_empty() {
            c.llm_api_key = update.llm_api_key.trim().to_string();
        }
        c.llm_api_url = update.llm_api_url;
        c.llm_model = update.llm_model;
        c.enable_correction = update.enable_correction;
        c.hotkey = update.hotkey;
        c.vocabulary = update.vocabulary;
        c.enable_formatting = update.enable_formatting;
        c.clone()
    };
    snapshot.save(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_history(app: AppHandle) -> Vec<HistoryEntry> {
    history::load(&app)
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), String> {
    history::clear(&app).map_err(|e| e.to_string())?;
    let _ = app.emit_to("history", "history-cleared", ());
    Ok(())
}
