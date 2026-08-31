// Settings window: load the current config, let the user edit it, persist.
// Changes are applied automatically (auto-save) — no submit button needed.
const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

const $ = (id) => document.getElementById(id);

const LANG = {
  fr: { title: "Réglages", city: "Ville", latitude: "Latitude", longitude: "Longitude", locate: "Utiliser ma position", method: "Méthode de calcul", school: "École (Asr)", highLat: "Règle hautes latitudes", language: "Langue", hourFormat: "Format de l'heure", autostart: "Démarrer avec Windows", startHidden: "Démarrer masqué (prochain démarrage)", close: "Fermer", saved: "✓ Enregistré", positionError: "Impossible de déterminer la position : " },
  en: { title: "Settings", city: "City", latitude: "Latitude", longitude: "Longitude", locate: "Use my location", method: "Calculation method", school: "School (Asr)", highLat: "High latitude rule", language: "Language", hourFormat: "Time format", autostart: "Start with Windows", startHidden: "Start hidden (next launch)", close: "Close", saved: "✓ Saved", positionError: "Unable to determine position: " },
  ar: { title: "الإعدادات", city: "المدينة", latitude: "خط العرض", longitude: "خط الطول", locate: "استخدام موقعي", method: "طريقة الحساب", school: "المدرسة (العصر)", highLat: "قاعدة خطوط العرض العالية", language: "اللغة", hourFormat: "صيغة الوقت", autostart: "التشغيل مع ويندوز", startHidden: "تشغيل مخفي (عند الإقلاع)", close: "إغلاق", saved: "✓ تم الحفظ", positionError: "تعذر تحديد الموقع: " },
};
let currentLang = "fr";

function applyLang(lang) {
  const t = LANG[lang] || LANG.fr;
  currentLang = lang;
  const set = (id, v) => { const el = document.getElementById(id); if (el) el.textContent = v; };
  set("i-title", t.title);
  set("i-city", t.city);
  set("i-latitude", t.latitude);
  set("i-longitude", t.longitude);
  set("i-method", t.method);
  set("i-school", t.school);
  set("i-highlat", t.highLat);
  set("i-language", t.language);
  set("i-hour", t.hourFormat);
  set("i-autostart", t.autostart);
  set("i-starthidden", t.startHidden);
  $("locate").title = t.locate;
  $("locate").setAttribute("aria-label", t.locate);
  $("cancel").textContent = t.close;
  $("saved-hint").textContent = t.saved;
  document.documentElement.lang = lang;
  document.body.dir = lang === "ar" ? "rtl" : "ltr";
}

function fill(cfg) {
  applyLang(cfg.language || "fr");
  $("city").value = cfg.city || "";
  if (cfg.coordinates) {
    $("lat").value = cfg.coordinates.lat;
    $("lon").value = cfg.coordinates.lon;
  }
  $("method").value = String(cfg.method);
  $("school").value = String(cfg.school);
  $("high-lat").value = String(cfg.high_lat_rule);
  $("language").value = cfg.language || "fr";
  $("hour12").value = String(cfg.hour12);
  $("autostart").checked = !!cfg.autostart;
  $("start-hidden").checked = !!cfg.start_hidden;
  $("timezone").value = cfg.timezone || "";
}

function collect() {
  return {
    method: Number($("method").value),
    school: Number($("school").value),
    high_lat_rule: Number($("high-lat").value),
    language: $("language").value,
    hour12: $("hour12").value === "true",
    coordinates: {
      lat: Number($("lat").value),
      lon: Number($("lon").value),
    },
    city: $("city").value.trim(),
    autostart: $("autostart").checked,
    start_hidden: $("start-hidden").checked,
    timezone: $("timezone").value || null,
  };
}

function showError(message) {
  const err = $("error");
  err.textContent = message;
  err.classList.remove("hidden");
}

function showSaved() {
  const h = $("saved-hint");
  h.classList.remove("hidden");
  clearTimeout(h._t);
  h._t = setTimeout(() => h.classList.add("hidden"), 1200);
}

// Debounced auto-save: apply the current form values to the config.
let saveTimer = null;
function autoSave() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    const latV = $("lat").value.trim();
    const lonV = $("lon").value.trim();
    if (latV === "" || lonV === "" || !Number.isFinite(Number(latV)) || !Number.isFinite(Number(lonV))) return;
    try {
      await invoke("set_config", { cfg: collect() });
      showSaved();
    } catch (err) {
      showError(String(err));
    }
  }, 250);
}

$("cancel").addEventListener("click", () => getCurrentWindow().hide());

async function detectLocation() {
  const btn = $("locate");
  btn.disabled = true;
  try {
    const loc = await invoke("detect_location");
    $("city").value = loc.city;
    $("lat").value = loc.lat;
    $("lon").value = loc.lon;
    // Auto-select the country's official method (Maroc → Maroc, etc.).
    $("method").value = String(loc.method);
    $("timezone").value = loc.timezone || "";
    autoSave();
  } catch (err) {
    showError(LANG[currentLang].positionError + (err.message || ""));
  } finally {
    btn.disabled = false;
  }
}
$("locate").addEventListener("click", detectLocation);

document.querySelector("#settings-form").addEventListener("change", autoSave);
$("lat").addEventListener("input", autoSave);
$("lon").addEventListener("input", autoSave);
document.querySelector("#settings-form").addEventListener("submit", (e) => e.preventDefault());

invoke("get_config")
  .then(fill)
  .catch((err) => showError(String(err)));
