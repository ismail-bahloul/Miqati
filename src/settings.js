// Settings window: load the current config, let the user edit it, persist.
const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

const $ = (id) => document.getElementById(id);

function fill(cfg) {
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
    // window_position is deliberately omitted: it is managed by the widget
    // drag and preserved by the backend on save.
  };
}

function showError(message) {
  const err = $("error");
  err.textContent = message;
  err.classList.remove("hidden");
}

$("cancel").addEventListener("click", () => getCurrentWindow().hide());

document.querySelector("#settings-form").addEventListener("submit", async (e) => {
  e.preventDefault();
  try {
    await invoke("set_config", { cfg: collect() });
    getCurrentWindow().hide();
  } catch (err) {
    showError(String(err));
  }
});

invoke("get_config")
  .then(fill)
  .catch((err) => showError(String(err)));
