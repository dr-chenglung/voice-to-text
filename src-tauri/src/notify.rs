//! 背景執行緒失敗的系統通知（規格 6.2 容錯的補強）。
//! 托盤圖示短暫變色／tooltip 容易被忽略（需滑鼠移過去才看得到，且只顯示約 1.2 秒），
//! 用 Windows 系統通知確保使用者一定能看到發生了什麼錯誤或降級。

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub fn show(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}
