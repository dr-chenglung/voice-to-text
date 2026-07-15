//! 文字輸出到焦點視窗。
//!
//! 採「剪貼簿貼上」而非逐字模擬輸入：把文字放進剪貼簿 → 模擬 Ctrl+V → 還原原本剪貼簿。
//! 原因：中文輸入法處於組字狀態時，逐字模擬的 Unicode 會被輸入法組字緩衝攔截，
//! 導致最後的標點（如「。」）懸在組字狀態、需再按 Enter 才上字。貼上可完全繞過此問題。
//! 若剪貼簿流程失敗，退回 enigo 直接打字作為備援。

use std::time::Duration;

use anyhow::{anyhow, Result};
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

/// 把文字輸出到目前焦點視窗。
pub fn type_text(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }
    match paste_text(text) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("[typer] 剪貼簿貼上失敗，改用模擬打字備援: {e}");
            direct_type(text)
        }
    }
}

/// 剪貼簿貼上：暫存原內容 → 寫入文字 → Ctrl+V → 還原原內容。
fn paste_text(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().map_err(|e| anyhow!("開啟剪貼簿失敗: {e}"))?;
    // 先保存原本的文字內容（若原本不是文字，還原時略過）。
    let previous = clipboard.get_text().ok();

    clipboard
        .set_text(text.to_string())
        .map_err(|e| anyhow!("寫入剪貼簿失敗: {e}"))?;
    drop(clipboard); // 釋放剪貼簿，讓目標視窗能讀取

    // 給系統一點時間讓剪貼簿就緒，再送出貼上。
    std::thread::sleep(Duration::from_millis(40));
    send_paste()?;
    // 等貼上動作完成再還原剪貼簿，避免太早覆蓋。
    std::thread::sleep(Duration::from_millis(120));

    if let Some(prev) = previous {
        if let Ok(mut cb) = Clipboard::new() {
            let _ = cb.set_text(prev);
        }
    }
    Ok(())
}

/// 模擬 Ctrl+V。enigo 對英文字母的 Key::Unicode 會送出對應的虛擬鍵，故組合鍵有效。
fn send_paste() -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| anyhow!("enigo 初始化失敗: {e}"))?;
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| anyhow!("按下 Ctrl 失敗: {e}"))?;
    let v_result = enigo.key(Key::Unicode('v'), Direction::Click);
    // 無論 V 是否成功，務必放開 Ctrl，避免卡住修飾鍵。
    let release = enigo.key(Key::Control, Direction::Release);
    v_result.map_err(|e| anyhow!("按下 V 失敗: {e}"))?;
    release.map_err(|e| anyhow!("放開 Ctrl 失敗: {e}"))?;
    Ok(())
}

/// 備援：直接以 enigo 模擬逐字輸入（可能受中文輸入法組字影響）。
fn direct_type(text: &str) -> Result<()> {
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| anyhow!("enigo 初始化失敗: {e}"))?;
    enigo
        .text(text)
        .map_err(|e| anyhow!("enigo 模擬輸入失敗: {e}"))?;
    Ok(())
}
