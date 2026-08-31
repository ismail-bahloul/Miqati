// Salaat Widget frontend.
// Talks to the Rust backend via Tauri commands and updates the UI each second.

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { PhysicalSize, PhysicalPosition } = window.__TAURI__.dpi;
const { listen } = window.__TAURI__.event;

const $ = (id) => document.getElementById(id);

const state = {
  times: null, // { Fajr, Sunrise, Dhuhr, Asr, Maghrib, Isha } as Date-comparable minutes
  hijri: "", // "21 Rajab 1447"
  city: "",
  nextName: "",
  hasLocation: false,
  language: "fr",
  hour12: false,
  tick: 0,
};

// Localized strings. Prayer keys are the backend's English identifiers.
const LANG = {
  fr: {
    Fajr: "Fajr",
    Sunrise: "Lever",
    Dhuhr: "Dhuhr",
    Asr: "Asr",
    Maghrib: "Maghrib",
    Isha: "Isha",
    remaining: "Restant",
    searching: "Recherche…",
    setup: "Configurer la position",
    settings: "Réglages",
    quit: "Fermer",
    reduce: "Réduire",
    in: "dans",
  },
  en: {
    Fajr: "Fajr",
    Sunrise: "Sunrise",
    Dhuhr: "Dhuhr",
    Asr: "Asr",
    Maghrib: "Maghrib",
    Isha: "Isha",
    remaining: "Remaining",
    searching: "Searching…",
    setup: "Configure location",
    settings: "Settings",
    quit: "Close",
    reduce: "Minimize",
    in: "in",
  },
  ar: {
    Fajr: "الفجر",
    Sunrise: "الشروق",
    Dhuhr: "الظهر",
    Asr: "العصر",
    Maghrib: "المغرب",
    Isha: "العشاء",
    remaining: "متبقٍ",
    searching: "بحث…",
    setup: "حدد موقعك",
    settings: "الإعدادات",
    quit: "إغلاق",
    reduce: "تصغير",
    in: "في",
  },
};

const strings = () => LANG[state.language] ?? LANG.fr;
const label = (key) => strings()[key] ?? key;

// Prayer keys in display order (backend identifiers).
const PRAYERS = ["Fajr", "Sunrise", "Dhuhr", "Asr", "Maghrib", "Isha"];

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

function renderTimes() {
  if (!state.times) return;
  const list = $("detail-list");
  list.innerHTML = "";
  PRAYERS.forEach((key) => {
    const isNext = key === state.nextName;
    const row = document.createElement("div");
    row.className = "prayer-row" + (isNext ? " next" : "");
    row.dataset.prayer = key;
    row.innerHTML = `
      <span class="row-name">${label(key)}</span>
      <div style="display:flex;gap:10px;align-items:center">
        ${isNext ? `<span class="row-count">${fmtDuration(state.remainingSeconds)}</span>` : ""}
        <span class="row-time">${fmtClock(state.times[key], state.hour12)}</span>
      </div>`;
    list.appendChild(row);
  });
  $("detail-hijri").textContent = state.hijri;
  $("detail-city").textContent = state.city || strings().searching;
}

async function refresh() {
  try {
    const data = await invoke("get_status");
    state.times = Object.fromEntries(
      PRAYERS.map((k, i) => [k, data.times[i]])
    );
    state.hijri = data.hijri;
    state.city = data.city;
    state.nextName = data.next_name;
    state.language = data.language;
    state.hour12 = data.hour12;
    state.hasLocation = true;
    state.remainingSeconds = data.remaining_seconds;
    applyLang();
    updateCompact();
    renderTimes();
    updateAlert();
  } catch (err) {
    state.hasLocation = false;
    $("compact-countdown").textContent = "--:--";
    updateCompact();
    // First launch (no location yet): try to detect it automatically.
    autoConfigure();
    if (typeof import.meta.env !== "undefined" && import.meta.env.MODE === "development") {
      console.error(err);
    }
  }
}

// First launch: no location configured — detect it automatically (city,
// coordinates, timezone and the country's method). Runs once; on failure the
// "Configurer la position" prompt stays and the user can use the loupe.
let autoDetectStarted = false;
async function autoConfigure() {
  if (autoDetectStarted) return;
  autoDetectStarted = true;
  try {
    const loc = await invoke("detect_location");
    const cfg = await invoke("get_config");
    cfg.city = loc.city;
    cfg.coordinates = { lat: loc.lat, lon: loc.lon };
    cfg.timezone = loc.timezone || null;
    cfg.method = loc.method;
    await invoke("set_config", { cfg });
    refresh();
  } catch {
    // Offline or detection failed: keep the prompt (retries on next refresh).
    autoDetectStarted = false;
  }
}

// Reflect the configured language on the static strings.
function applyLang() {
  const t = strings();
  $("compact-remaining").textContent = t.remaining;
  $("settings-btn").textContent = t.settings;
  $("reduce-btn").textContent = t.reduce;
  document.documentElement.lang = state.language;
  document.body.dir = state.language === "ar" ? "rtl" : "ltr";
}

function updateCompact() {
  if (!state.hasLocation) {
    $("compact-prayer-name").textContent = strings().setup;
    $("compact-prayer-time").textContent = "";
    $("compact-remaining").textContent = "";
    $("compact-countdown").textContent = "";
    return;
  }
  if (!state.times || !state.nextName) {
    $("compact-prayer-name").textContent = "—";
    $("compact-countdown").textContent = "--:--";
    return;
  }
  $("compact-prayer-name").textContent = label(state.nextName);
  $("compact-prayer-time").textContent = fmtClock(state.times[state.nextName], state.hour12);
  $("compact-countdown").textContent = fmtDuration(state.remainingSeconds);

  // Keep the tray tooltip in sync with the live countdown.
  const t = strings();
  const tooltip = `${label(state.nextName)} ${t.in} ${fmtDuration(state.remainingSeconds)}`;
  invoke("update_tray", { tooltip }).catch(() => {});
}

// Pre-prayer glow during the last 5 minutes.
function updateAlert() {
  const alert = state.remainingSeconds >= 0 && state.remainingSeconds <= 300;
  document.body.classList.toggle("alert", alert);
}

// Compact -> detail toggle on click. The window is resized to fit the detail
// view while keeping the bottom edge anchored (the widget grows upward, so it
// stays docked against the taskbar).
const WIDGET_WIDTH = 240;
const COMPACT_HEIGHT = 60;
const DETAIL_HEIGHT = 292;

async function toggleView() {
  const compact = $("compact");
  const detail = $("detail");
  const goingDetail = !compact.classList.contains("hidden");
  compact.classList.toggle("hidden", goingDetail);
  detail.classList.toggle("hidden", !goingDetail);
  if (goingDetail) renderTimes();

  const win = getCurrentWindow();
  try {
    const before = await win.outerSize();
    const pos = await win.outerPosition();
    const factor = await win.scaleFactor();
    const targetH = goingDetail ? DETAIL_HEIGHT : COMPACT_HEIGHT;
    const size = new PhysicalSize(WIDGET_WIDTH, targetH * factor);
    await win.setSize(size);
    // Keep the bottom edge in place (grow upward in detail view).
    await win.setPosition(
      new PhysicalPosition(pos.x, pos.y - (size.height - before.height))
    );
  } catch {
    // Resize/position failures are non-fatal (e.g. permissions missing).
  }
}

// Drag to move the widget, while keeping click-to-expand. A press is a
// "click" unless the pointer moves beyond a small threshold, in which case
// the gesture is handed over to the OS window drag (tao posts WM_NCLBUTTONDOWN
// asynchronously, so the drag promise resolves BEFORE the window moves — the
// final position is captured through move events instead). Pointer capture
// keeps the gesture working even when the cursor leaves the widget.
let pressStart = null;
let dragging = false; // an OS window drag is (or just was) running
let lastDragPos = null; // latest position during the drag (physical px)
let dragSaveTimer = null;

// Save the last dragged position (physical -> logical) to the config.
async function saveDragPosition() {
  if (!lastDragPos) return;
  try {
    const factor = await getCurrentWindow().scaleFactor();
    const logical = lastDragPos.toLogical(factor);
    await invoke("save_window_position", { x: logical.x, y: logical.y });
  } catch {}
  // The drag is finished: never let later programmatic moves (view resize,
  // re-dock, show) be mistaken for a user drag and re-saved.
  dragging = false;
}

// Both views are draggable the same way (the 5 px threshold keeps button
// clicks intact).
function armDrag(element) {
  element.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    // Buttons keep their own click handling: never capture their pointer.
    if (e.target.closest("button")) return;
    // A new press while a post-drag save is pending: flush it before resetting.
    if (dragSaveTimer) {
      clearTimeout(dragSaveTimer);
      dragSaveTimer = null;
      saveDragPosition();
    }
    pressStart = { x: e.screenX, y: e.screenY };
    dragging = false;
    lastDragPos = null;
    element.setPointerCapture(e.pointerId);
  });

  element.addEventListener("pointermove", (e) => {
    if (!pressStart) return;
    const dx = Math.abs(e.screenX - pressStart.x);
    const dy = Math.abs(e.screenY - pressStart.y);
    if (dx + dy > 5) {
      pressStart = null;
      dragging = true;
      getCurrentWindow().startDragging().catch(() => {});
    }
  });
}

armDrag($("compact"));
armDrag($("detail"));

// Clicking the visible view toggles it; clicks landing on the detail buttons
// bubble up but are ignored (the buttons handle themselves).
$("compact").addEventListener("click", () => {
  if (!state.hasLocation) { invoke("open_settings"); return; }
  if (!pressStart) return; // was a drag, not a click
  pressStart = null;
  if (!$("compact").classList.contains("hidden")) toggleView();
});

$("detail").addEventListener("click", (e) => {
  if (!pressStart) return; // was a drag, not a click
  pressStart = null;
  if (e.target.closest("button")) return;
  if (!$("detail").classList.contains("hidden")) toggleView();
});

// While an OS drag runs, the window reports its position through move
// events; persist the last one shortly after the movement stops.
getCurrentWindow()
  .onMoved(({ payload }) => {
    if (!dragging) return; // ignore programmatic moves (dock, view resize)
    lastDragPos = payload;
    if (dragSaveTimer) clearTimeout(dragSaveTimer);
    dragSaveTimer = setTimeout(saveDragPosition, 250);
  })
  .catch(() => {});

function init() {
  $("settings-btn").addEventListener("click", () => {
    invoke("open_settings");
  });
  $("reduce-btn").addEventListener("click", () => {
    invoke("hide_window");
  });

  // Refresh right away when the settings window saves new values.
  listen("config-changed", () => refresh()).catch(() => {});

  // Smooth fade when the tray toggles the window: fade out, then ask the
  // backend to hide; fade back in when it re-shows.
  let hidePending = null;
  listen("animate-out", () => {
    clearTimeout(hidePending);
    document.body.style.transition = "opacity 180ms ease-out";
    document.body.style.opacity = "0";
    hidePending = setTimeout(() => invoke("hide_window").catch(() => {}), 190);
  });
  listen("animate-in", () => {
    clearTimeout(hidePending);
    document.body.style.transition = "opacity 200ms ease-in";
    document.body.style.opacity = "1";
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


