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
    autostart: "Démarrer avec Windows", startHidden: "Démarrer masqué (prochain démarrage)",
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
    autostart: "Start with Windows", startHidden: "Start hidden (next launch)",
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
    autostart: "التشغيل مع ويندوز", startHidden: "تشغيل مخفي (عند الإقلاع)",
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
