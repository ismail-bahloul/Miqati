// Settings window: load the current config, let the user edit it, persist.
// Changes are applied automatically (auto-save) — no submit button needed.
const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

const $ = (id) => document.getElementById(id);

const LANG = {
  fr: {
    title: "Réglages", city: "Ville", latitude: "Latitude", longitude: "Longitude",
    locate: "Utiliser ma position", method: "Méthode de calcul", school: "École (Asr)",
    highLat: "Règle hautes latitudes", language: "Langue", hourFormat: "Format de l'heure",
    timezone: "Fuseau horaire",
    autostart: "Démarrer avec Windows", startHidden: "Démarrer masqué (prochain démarrage)",
    alwaysOnTop: "Toujours au premier plan",
    notify: "Rappels",
    notifyOff: "Désactivés",
    notifyAtTime: "À l'heure de la prière",
    notifyBefore: "Avant la prière",
    notifyBoth: "À l'heure et avant",
    notifyMinutes: "Délai du rappel",
    advanced: "Réglages avancés",
    offlineMode: "Mode hors ligne (aucune requête réseau automatique)",
    offlineLocate: "Fait une requête réseau (désactivé en mode hors ligne)",
    close: "Fermer", saved: "✓ Enregistré", positionError: "Impossible de déterminer la position : ",
    cityPlaceholder: "Paris",
    methodOptions: {
      "12": "UOIF (France)", "21": "Maroc (Ministère des Habous)", "3": "Muslim World League",
      "2": "ISNA (Amérique du Nord)", "1": "Université de Karachi", "4": "Umm Al-Qura (La Mecque)",
      "5": "Égypte (EGA)", "19": "Algérie", "18": "Tunisie", "8": "Golfe", "9": "Koweït",
      "10": "Qatar", "11": "Singapour", "16": "Dubaï", "17": "JAKIM (Malaisie)",
      "20": "KEMENAG (Indonésie)", "22": "Portugal", "23": "Jordanie", "13": "Diyanet (Turquie)",
      "14": "Russie", "7": "Téhéran", "0": "Jafari (Chiite)"
    },
    schoolOptions: { "0": "Générale (Chaféite…)", "1": "Hanafite" },
    highLatOptions: { "2": "Méthode angulaire", "0": "Milieu de la nuit", "1": "Septième de la nuit" },
    hour12Options: { "false": "24 h", "true": "12 h (AM/PM)" },
  },
  en: {
    title: "Settings", city: "City", latitude: "Latitude", longitude: "Longitude",
    locate: "Use my location", method: "Calculation method", school: "School (Asr)",
    highLat: "High latitude rule", language: "Language", hourFormat: "Time format",
    timezone: "Timezone",
    autostart: "Start with Windows", startHidden: "Start hidden (next launch)",
    alwaysOnTop: "Always on top",
    notify: "Reminders",
    notifyOff: "Off",
    notifyAtTime: "At prayer time",
    notifyBefore: "Before prayer",
    notifyBoth: "At and before prayer",
    notifyMinutes: "Reminder lead time",
    advanced: "Advanced settings",
    offlineMode: "Offline mode (no automatic network request)",
    offlineLocate: "Makes a network request (disabled in offline mode)",
    close: "Close", saved: "✓ Saved", positionError: "Unable to determine position: ",
    cityPlaceholder: "Paris",
    methodOptions: {
      "12": "UOIF (France)", "21": "Morocco (Ministry of Habous)", "3": "Muslim World League",
      "2": "ISNA (North America)", "1": "University of Karachi", "4": "Umm Al-Qura (Makkah)",
      "5": "Egypt (EGA)", "19": "Algeria", "18": "Tunisia", "8": "Gulf", "9": "Kuwait",
      "10": "Qatar", "11": "Singapore", "16": "Dubai", "17": "JAKIM (Malaysia)",
      "20": "KEMENAG (Indonesia)", "22": "Portugal", "23": "Jordan", "13": "Diyanet (Turkey)",
      "14": "Russia", "7": "Tehran", "0": "Jafari (Shia)"
    },
    schoolOptions: { "0": "General (Shafi'i…)", "1": "Hanafi" },
    highLatOptions: { "2": "Angle-based method", "0": "Middle of the night", "1": "Seventh of the night" },
    hour12Options: { "false": "24 h", "true": "12 h (AM/PM)" },
  },
  ar: {
    title: "الإعدادات", city: "المدينة", latitude: "خط العرض", longitude: "خط الطول",
    locate: "استخدام موقعي", method: "طريقة الحساب", school: "المدرسة (العصر)",
    highLat: "قاعدة خطوط العرض العالية", language: "اللغة", hourFormat: "صيغة الوقت",
    timezone: "المنطقة الزمنية",
    autostart: "التشغيل مع ويندوز", startHidden: "تشغيل مخفي (عند الإقلاع)",
    alwaysOnTop: "دائمًا في المقدمة",
    notify: "التذكيرات",
    notifyOff: "معطّلة",
    notifyAtTime: "عند دخول الوقت",
    notifyBefore: "قبل الصلاة",
    notifyBoth: "عند الوقت وقبله",
    notifyMinutes: "مدة التذكير",
    advanced: "إعدادات متقدمة",
    offlineMode: "وضع عدم الاتصال (لا طلبات شبكة تلقائية)",
    offlineLocate: "يُجري طلب شبكة (معطّل في وضع عدم الاتصال)",
    close: "إغلاق", saved: "✓ تم الحفظ", positionError: "تعذر تحديد الموقع: ",
    cityPlaceholder: "الرباط",
    methodOptions: {
      "12": "UOIF (فرنسا)", "21": "المغرب (وزارة الأوقاف)", "3": "رابطة العالم الإسلامي",
      "2": "ISNA (أمريكا الشمالية)", "1": "جامعة كراتشي", "4": "أم القرى (مكة)",
      "5": "مصر (EGA)", "19": "الجزائر", "18": "تونس", "8": "الخليج", "9": "الكويت",
      "10": "قطر", "11": "سنغافورة", "16": "دبي", "17": "JAKIM (ماليزيا)",
      "20": "KEMENAG (إندونيسيا)", "22": "البرتغال", "23": "الأردن", "13": "ديانت (تركيا)",
      "14": "روسيا", "7": "طهران", "0": "جعفري (شيعي)"
    },
    schoolOptions: { "0": "عامة (الشافعية…)", "1": "حنفي" },
    highLatOptions: { "2": "الطريقة الزاوية", "0": "منتصف الليل", "1": "سبع الليل" },
    hour12Options: { "false": "24 ساعة", "true": "12 ساعة (صباحًا/مساءً)" },
  },
};
let currentLang = "fr";

function fillSelectOptions(selectId, map) {
  const sel = document.getElementById(selectId);
  if (!sel || !map) return;
  for (const opt of sel.options) {
    if (Object.prototype.hasOwnProperty.call(map, opt.value)) opt.textContent = map[opt.value];
  }
}

// "5 min", "5 mins", "5 دقائق"… built from the option values so the list
// stays in one place (settings.html).
function minuteOptions(lang) {
  const unit = lang === "ar" ? "دقيقة" : "min";
  const out = {};
  for (const opt of $("notify-before-minutes").options) out[opt.value] = `${opt.value} ${unit}`;
  return out;
}

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
  set("i-timezone", t.timezone);
  set("i-autostart", t.autostart);
  set("i-starthidden", t.startHidden);
  set("i-alwaysontop", t.alwaysOnTop);
  set("i-notify", t.notify);
  set("i-notifyminutes", t.notifyMinutes);
  set("i-advanced", t.advanced);
  set("i-offlinemode", t.offlineMode);
  fillSelectOptions("notify-before-minutes", minuteOptions(lang));
  fillSelectOptions("notify-mode", {
    off: t.notifyOff,
    "at-time": t.notifyAtTime,
    before: t.notifyBefore,
    both: t.notifyBoth,
  });
  $("city").placeholder = t.cityPlaceholder;
  fillSelectOptions("method", t.methodOptions);
  fillSelectOptions("school", t.schoolOptions);
  fillSelectOptions("high-lat", t.highLatOptions);
  fillSelectOptions("hour12", t.hour12Options);
  $("locate").title = t.locate;
  $("locate").setAttribute("aria-label", t.locate);
  $("cancel").textContent = t.close;
  $("saved-hint").textContent = t.saved;
  document.documentElement.lang = lang;
  document.body.dir = lang === "ar" ? "rtl" : "ltr";
  // Re-apply the offline hint in the new language (overrides the title above
  // when offline mode is on).
  syncOfflineState();
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
  $("always-on-top").checked = !!cfg.always_on_top;
  $("notify-mode").value = notifyModeFrom(cfg);
  $("notify-before-minutes").value = String(cfg.notify_before_minutes ?? 10);
  syncNotifyState();
  $("offline-mode").checked = !!cfg.offline_mode;
  syncOfflineState();
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
    always_on_top: $("always-on-top").checked,
    notify_at_time: notifyFlags().atTime,
    notify_before: notifyFlags().before,
    notify_before_minutes: Number($("notify-before-minutes").value),
    offline_mode: $("offline-mode").checked,
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

// The single "Reminders" selector maps onto the two backend booleans, so the
// config format does not change (no migration needed).
function notifyModeFrom(cfg) {
  const at = !!cfg.notify_at_time;
  const before = !!cfg.notify_before;
  if (at && before) return "both";
  if (at) return "at-time";
  if (before) return "before";
  return "off";
}

function notifyFlags() {
  const mode = $("notify-mode").value;
  return {
    atTime: mode === "at-time" || mode === "both",
    before: mode === "before" || mode === "both",
  };
}

// The lead time only means something when the early reminder is on.
function syncNotifyState() {
  const { before } = notifyFlags();
  $("notify-minutes-row").classList.toggle("hidden", !before);
}
$("notify-mode").addEventListener("change", () => {
  syncNotifyState();
  autoSave();
});

// In offline mode the location button cannot work (the backend refuses the
// request), so disable it and explain why on hover.
function syncOfflineState() {
  const offline = $("offline-mode").checked;
  const btn = $("locate");
  btn.disabled = offline;
  btn.title = offline ? LANG[currentLang].offlineLocate : LANG[currentLang].locate;
}
$("offline-mode").addEventListener("change", () => {
  syncOfflineState();
  autoSave();
});

$("cancel").addEventListener("click", () => getCurrentWindow().hide());
$("language").addEventListener("change", (e) => applyLang(e.target.value));

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
