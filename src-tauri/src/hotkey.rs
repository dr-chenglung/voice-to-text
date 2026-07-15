//! 熱鍵監聽（規格第 3.1 節）。
//! 在獨立執行緒以 rdev::listen 監聽鍵盤，偵測目標鍵（預設右 Alt）的「按下」transition，
//! 每次按下送出一個 toggle 訊號。按住自動重複的 KeyPress 會被 `down` 旗標濾掉。

use rdev::{listen, Event, EventType, Key};
use std::sync::mpsc::Sender;

/// 把設定字串轉成 rdev::Key。Windows 上右 Alt 回報為 AltGr、右 Ctrl 回報為 ControlRight。
pub fn parse_key(s: &str) -> Option<Key> {
    let norm = s.to_lowercase().replace([' ', '-'], "_");
    match norm.as_str() {
        "right_alt" | "alt_right" | "altgr" | "ralt" => Some(Key::AltGr),
        "right_ctrl" | "ctrl_right" | "right_control" | "rctrl" => Some(Key::ControlRight),
        _ => None,
    }
}

/// 阻塞式監聽迴圈，應在獨立執行緒呼叫。每次目標鍵按下送一個 `()` 到 channel。
pub fn run(target: Key, tx: Sender<()>) {
    let mut down = false;
    let callback = move |event: Event| match event.event_type {
        EventType::KeyPress(k) if k == target => {
            if !down {
                down = true;
                let _ = tx.send(());
            }
        }
        EventType::KeyRelease(k) if k == target => {
            down = false;
        }
        _ => {}
    };
    if let Err(e) = listen(callback) {
        eprintln!("[hotkey] rdev listen 失敗: {:?}", e);
    }
}
