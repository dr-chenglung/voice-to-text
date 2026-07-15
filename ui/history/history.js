// 歷史紀錄視窗：讀取/清除 history.json（經由 Rust 端 get_history/clear_history command）。
// 後端已用新到舊排序回傳，這裡不重新排序。

const { core, event } = window.__TAURI__;

const listEl = document.getElementById("list");
const emptyEl = document.getElementById("empty");
const clearBtn = document.getElementById("clear");

function formatTimestamp(unixSeconds) {
  return new Date(unixSeconds * 1000).toLocaleString();
}

function render(entries) {
  listEl.innerHTML = "";
  emptyEl.classList.toggle("hidden", entries.length > 0);
  for (const entry of entries) {
    const li = document.createElement("li");

    const ts = document.createElement("span");
    ts.className = "timestamp";
    ts.textContent = formatTimestamp(entry.timestamp);

    const text = document.createElement("span");
    text.className = "text";
    text.textContent = entry.text;

    li.appendChild(ts);
    li.appendChild(text);
    listEl.appendChild(li);
  }
}

async function load() {
  const entries = await core.invoke("get_history");
  render(entries);
}

clearBtn.addEventListener("click", async () => {
  if (!confirm("確定要清除全部歷史紀錄嗎？此動作無法復原。")) {
    return;
  }
  await core.invoke("clear_history");
  await load();
});

event.listen("history-cleared", () => load());

load();
