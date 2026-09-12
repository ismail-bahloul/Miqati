// Frontend tests for the widget.
//
// `main.js` is a side-effect script with no exports, so it is driven as a black
// box: stub `window.__TAURI__` and a minimal DOM on `globalThis`, import the
// real file, then drive its timers by hand with a frozen clock. What we assert
// is what the user sees — the rendered strings — plus the IPC it emits.
//
//   node --test src/
import { test, before } from "node:test";
import assert from "node:assert/strict";

// ---------------------------------------------------------------- DOM stubs

function makeClassList(initial = []) {
  const set = new Set(initial);
  return {
    contains: (c) => set.has(c),
    add: (c) => set.add(c),
    remove: (c) => set.delete(c),
    toggle: (c, force) => {
      const on = force === undefined ? !set.has(c) : force;
      if (on) set.add(c);
      else set.delete(c);
      return on;
    },
  };
}

function makeElement(id, classes = []) {
  return {
    id,
    textContent: "",
    innerHTML: "",
    className: "",
    dataset: {},
    style: {},
    classList: makeClassList(classes),
    children: [],
    appendChild(child) {
      this.children.push(child);
    },
    addEventListener() {},
    closest: () => null,
  };
}

/// The ids `main.js` reaches for, with the initial visibility from index.html:
/// the compact bar is shown, the detail view carries `hidden`.
const elements = new Map();
function resetDom() {
  elements.clear();
  for (const id of [
    "compact-prayer-name",
    "compact-prayer-time",
    "compact-remaining",
    "compact-countdown",
    "detail-list",
    "detail-hijri",
    "detail-city",
    "settings-btn",
    "reduce-btn",
  ]) {
    elements.set(id, makeElement(id));
  }
  elements.set("compact", makeElement("compact"));
  elements.set("detail", makeElement("detail", ["hidden"]));
}

const $ = (id) => elements.get(id);

// ------------------------------------------------------------- Tauri stubs

/// Prayer times in minutes since midnight: Fajr 05:00 … Isha 22:00.
const TIMES = [300, 390, 780, 960, 1230, 1320];

const ipc = { updateTrayCalls: [], statusCalls: 0 };
// Deliberately not a round 3600: starting exactly on a minute boundary would
// flip the rendered string on the very first tick and hide the dedup.
let nextRemaining = 3630;

async function invoke(command, args) {
  switch (command) {
    case "get_status":
      ipc.statusCalls += 1;
      return {
        times: TIMES,
        next_name: "Asr",
        remaining_seconds: nextRemaining,
        hijri: "21 Rajab 1447",
        city: "Paris",
        language: "fr",
        hour12: false,
      };
    case "get_taskbar_rect":
      return null; // non-Windows: skip the compact-bar resize path
    case "update_tray":
      ipc.updateTrayCalls.push(args.tooltip);
      return null;
    default:
      return null;
  }
}

// ------------------------------------------------------------- fake clock

let now = 1_757_700_000_000; // fixed epoch ms
const realDateNow = Date.now;
/// Callbacks registered through `setInterval`, in registration order:
/// `main.js` installs the 1 s tick first, then the hourly refresh.
const intervals = [];

function installGlobals() {
  resetDom();

  globalThis.window = {
    __TAURI__: {
      core: { invoke },
      window: {
        getCurrentWindow: () => ({
          scaleFactor: async () => 1,
          outerSize: async () => ({ toLogical: () => ({ width: 300, height: 60 }) }),
          outerPosition: async () => ({ toLogical: () => ({ x: 0, y: 0 }) }),
          setSize: async () => {},
          setPosition: async () => {},
          onMoved: async () => {},
        }),
      },
      dpi: {
        LogicalSize: class {
          constructor(width, height) {
            Object.assign(this, { width, height });
          }
        },
        LogicalPosition: class {
          constructor(x, y) {
            Object.assign(this, { x, y });
          }
        },
      },
      event: { listen: async () => {} },
    },
  };

  globalThis.document = {
    getElementById: (id) => $(id) ?? null,
    createElement: () => makeElement("created"),
    documentElement: { lang: "" },
    body: { dir: "", style: {}, classList: makeClassList() },
    querySelector: () => null,
  };

  Date.now = () => now;
  globalThis.setInterval = (fn) => {
    intervals.push(fn);
    return intervals.length;
  };
  globalThis.setTimeout = () => 0;
  globalThis.clearTimeout = () => {};
}

/// Let the module's pending promises settle.
const settle = () => new Promise((resolve) => process.nextTick(resolve));

/// Advance the frozen clock and run the 1 s tick once, as the real timer would.
async function advance(seconds) {
  now += seconds * 1000;
  intervals[0]();
  await settle();
}

before(async () => {
  installGlobals();
  await import("./main.js");
  await settle(); // the initial refresh() resolves
});

// ------------------------------------------------------------------ tests

test("renders the next prayer from the backend payload", () => {
  assert.equal($("compact-prayer-name").textContent, "Asr");
  assert.equal($("compact-prayer-time").textContent, "16:00"); // 960 min
  assert.equal($("compact-countdown").textContent, "01:00"); // 3600 s
  assert.equal($("compact-remaining").textContent, "Restant");
});

test("a backend refresh forces exactly one repaint", async () => {
  // `refresh()` clears the dedup key so the fresh payload is always painted,
  // even when the countdown happens to render the same string as before.
  const before = ipc.updateTrayCalls.length;
  await advance(1);
  assert.equal(ipc.updateTrayCalls.length, before + 1);
});

test("the countdown only repaints when the displayed minute changes", async () => {
  const before = ipc.updateTrayCalls.length;

  // 3629 s down to 3600 s: twenty-nine one-second ticks all rendering "01:00".
  for (let i = 0; i < 29; i++) await advance(1);
  assert.equal(
    ipc.updateTrayCalls.length,
    before,
    "no tray IPC while the rendered string is unchanged",
  );
  assert.equal($("compact-countdown").textContent, "01:00");

  // The next one crosses the minute boundary.
  await advance(1);
  assert.equal(ipc.updateTrayCalls.length, before + 1, "exactly one repaint per minute");
  assert.equal($("compact-countdown").textContent, "00:59");
  assert.equal(ipc.updateTrayCalls.at(-1), "Asr dans 00:59");
});

test("the countdown survives the machine sleeping", async () => {
  // The old code decremented a counter, so a suspended machine froze it: after
  // a 10-minute sleep the widget stayed 10 minutes wrong until the hourly
  // refresh. Deriving from the wall clock makes the jump self-correcting.
  const before = $("compact-countdown").textContent;
  assert.equal(before, "00:59");

  await advance(10 * 60); // one tick, ten minutes later

  assert.equal($("compact-countdown").textContent, "00:49");
});

test("reaching zero asks the backend for the next prayer", async () => {
  const before = ipc.statusCalls;
  nextRemaining = 7200; // what the backend will answer for the following prayer

  await advance(2999); // run the countdown down to 0

  assert.ok(ipc.statusCalls > before, "a refresh was requested at the rollover");
  assert.equal($("compact-countdown").textContent, "02:00", "shows the new countdown");
});

test("the detail view stays collapsed until it is opened", () => {
  // renderTimes() must not paint into a hidden view.
  assert.ok($("detail").classList.contains("hidden"));
  assert.equal($("detail-list").children.length, 0);
});
