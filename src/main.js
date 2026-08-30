// Salaat Widget frontend.
// Talks to the Rust backend via Tauri commands and updates the UI each second.

import { invoke } from "@tauri-apps/api/core";

const $ = (id) => document.getElementById(id);

const state = {
  times: null, // { Fajr, Sunrise, Dhuhr, Asr, Maghrib, Isha } as Date-comparable minutes
  hijri: "", // "21 Rajab 1447"
  city: "Recherche…",
  nextName: "",
  tick: 0,
};

// Map prayer key -> localized display name.
const PRAYERS = ["Fajr", "Sunrise", "Dhuhr", "Asr", "Maghrib", "Isha"];
const LABELS_FR = {
  Fajr: "Fajr",
  Sunrise: "Lever",
  Dhuhr: "Dhuhr",
  Asr: "Asr",
  Maghrib: "Maghrib",
  Isha: "Isha",
};

function fmtClock(minutes, hour12 = false) {
  let h = Math.floor(minutes / 60) % 24;
  let m = Math.round(minutes - Math.floor(minutes / 60) * 60);
  if (m >= 60) {
    m -= 60;
    h = (h + 1) % 24;
  }
  if (hour12) {
    const ampm = h < 12 ? "AM" : "PM";
    const hh = h % 12 || 12;
    return `${hh}:${String(m).padStart(2, "0")} ${ampm}`;
  }
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

function fmtDuration(totalSeconds) {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

function highlightNext(name) {
  document.querySelectorAll(".prayer-row").forEach((row) => {
    row.classList.toggle("next", row.dataset.prayer === name);
  });
}

function renderTimes() {
  if (!state.times) return;
  const list = $("detail-list");
  list.innerHTML = "";
  PRAYERS.forEach((key, i) => {
    const isNext = key === state.nextName;
    const row = document.createElement("div");
    row.className = "prayer-row" + (isNext ? " next" : "");
    row.dataset.prayer = key;
    row.innerHTML = `
      <span class="row-name">${LABELS_FR[key] ?? key}</span>
      <div style="display:flex;gap:10px;align-items:center">
        ${isNext ? `<span class="row-count">${fmtDuration(state.remainingSeconds)}</span>` : ""}
        <span class="row-time">${fmtClock(state.times[key])}</span>
      </div>`;
    list.appendChild(row);
  });
  $("detail-hijri").textContent = state.hijri;
  $("detail-city").textContent = state.city;
}

async function refresh() {
  try {
    const data = await invoke("get_status");
    state.times = data.times;
    state.hijri = data.hijri;
    state.city = data.city || "Recherche…";
    state.nextName = data.nextName;
    state.remainingSeconds = data.remainingSeconds;
    updateCompact();
    renderTimes();
    updateAlert();
  } catch (err) {
    $("compact-countdown").textContent = "--:--";
    if (import.meta.env.MODE === "development") console.error(err);
  }
}

function updateCompact() {
  if (!state.times || !state.nextName) {
    $("compact-prayer-name").textContent = "—";
    $("compact-countdown").textContent = "--:--";
    return;
  }
  $("compact-prayer-name").textContent = LABELS_FR[state.nextName] ?? state.nextName;
  $("compact-prayer-time").textContent = fmtClock(state.times[state.nextName]);
  $("compact-countdown").textContent = fmtDuration(state.remainingSeconds);

  // Keep the tray tooltip in sync with the live countdown.
  const label = LABELS_FR[state.nextName] ?? state.nextName;
  const t = fmtDuration(state.remainingSeconds);
  invoke("update_tray", { tooltip: `${label} dans ${t}` }).catch(() => {});
}

// Pre-prayer glow during the last 5 minutes.
function updateAlert() {
  const alert = state.remainingSeconds >= 0 && state.remainingSeconds <= 300;
  document.body.classList.toggle("alert", alert);
}

// Compact -> detail toggle on click.
function toggleView() {
  const compact = $("compact");
  const detail = $("detail");
  const goingDetail = !compact.classList.contains("hidden");
  compact.classList.toggle("hidden", goingDetail);
  detail.classList.toggle("hidden", !goingDetail);
  if (goingDetail) renderTimes();
}

function init() {
  document.querySelector("#compact").addEventListener("click", toggleView);

  $("settings-btn").addEventListener("click", () => {
    invoke("open_settings");
  });
  $("quit-btn").addEventListener("click", () => {
    invoke("quit_app");
  });

  // Kick off + tick per-second countdown (UI side) and refresh full data.
  refresh();
  setInterval(() => {
    // Decrement local remainingSeconds each second for a smooth countdown.
    if (state.remainingSeconds !== undefined && state.remainingSeconds > 0) {
      state.remainingSeconds -= 1;
      updateCompact();
    } else if (state.remainingSeconds === 0) {
      refresh(); // roll over to next prayer / next day
    }
  }, 1000);

  // Full hourly refresh to catch DST / date change.
  setInterval(refresh, 3600_000);
}

init();
