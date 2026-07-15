//! 錄音開始／結束的提示音。
//!
//! 播放**內嵌**的 WAV（`include_bytes!`），透過 `PlaySoundW` 的 `SND_MEMORY` 從記憶體
//! 播放。之所以不用系統事件音別名（`SND_ALIAS`），是因為那會依賴使用者的「音效方案」
//! 設定——若該事件被設成「(無)」就完全沒聲音（實測踩到）；內嵌固定 WAV 則不論機器
//! 設定、也不論 `C:\Windows\Media` 檔案是否存在，都保證播得出來。
//!
//! `SND_ASYNC` 立即返回、不阻塞 controller 執行緒（規格 6.1：背景工作執行緒不可卡住）。
//! 內嵌資料為 `'static`，`SND_ASYNC` 播放期間記憶體恆有效。音效屬輔助回饋，播放失敗
//! 一律靜默忽略，絕不影響主流程。
//!
//! 僅 Windows；對外只暴露 `play_start`/`play_stop` 兩個平台無關的介面。

use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY};

// 溫暖、低沉、不刺耳的內建音（使用者逐一試聽後選定）：開始用 Windows Unlock、
// 結束用 Windows Logoff（下降感）。音量已在資產階段烤成 20%（PlaySound 無音量參數）。
// 內嵌進執行檔，不依賴系統音效方案或 C:\Windows\Media 檔案是否存在。
static START_WAV: &[u8] = include_bytes!("../assets/start.wav");
static STOP_WAV: &[u8] = include_bytes!("../assets/stop.wav");

/// 錄音開始提示音（Windows Unlock，溫暖上揚）。
pub fn play_start() {
    play(START_WAV);
}

/// 錄音結束提示音（Windows Logoff，溫暖下降）。
pub fn play_stop() {
    play(STOP_WAV);
}

/// 從記憶體非同步播放一段 WAV；失敗靜默忽略。
fn play(wav: &'static [u8]) {
    // SND_MEMORY：第一參數為指向記憶體中 WAV 資料的指標（型別上仍是寬字串指標，
    // 直接轉型傳位址即可）。
    // SAFETY: wav 為 'static、播放期間恆有效；hmod 傳 null（記憶體播放不需模組控制代碼）。
    unsafe {
        PlaySoundW(
            wav.as_ptr() as *const u16,
            std::ptr::null_mut(),
            SND_MEMORY | SND_ASYNC,
        );
    }
}
