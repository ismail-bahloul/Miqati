// Update check: is a newer release published on GitHub?
//
// The network call is a plain `fetch` to the public GitHub Releases API, made
// from the webview — which ships a TLS stack — rather than from Rust, whose
// HTTP client is deliberately plain-HTTP only (see src-tauri/Cargo.toml). The
// parsing and comparison are pure so they can be unit-tested (see
// src/update.test.mjs).

export const RELEASES_API = "https://api.github.com/repos/ismail-bahloul/Miqati/releases/latest";
export const RELEASES_PAGE = "https://github.com/ismail-bahloul/Miqati/releases/latest";

/// "v0.2.2" / "0.2.2" -> [0, 2, 2]; `null` when it is not an `x.y.z` version.
export function parseVersion(value) {
  const m = /^v?(\d+)\.(\d+)\.(\d+)/.exec(String(value ?? "").trim());
  return m ? [Number(m[1]), Number(m[2]), Number(m[3])] : null;
}

/// True when `candidate` is strictly newer than `current`.
export function isNewer(candidate, current) {
  const a = parseVersion(candidate);
  const b = parseVersion(current);
  if (!a || !b) return false;
  for (let i = 0; i < 3; i++) {
    if (a[i] !== b[i]) return a[i] > b[i];
  }
  return false;
}

/// Read a GitHub "latest release" payload into `{ version, url }`, or `null`
/// when there is nothing newer than `current`.
export function updateFromRelease(release, current) {
  const tag = release?.tag_name;
  if (!tag || !isNewer(tag, current)) return null;
  return {
    version: String(tag).replace(/^v/, ""),
    url: release.html_url || RELEASES_PAGE,
  };
}

/// Ask GitHub for the latest release. Resolves to `{ version, url }` when it is
/// newer than `current`, and to `null` otherwise — no update, no published
/// release yet, offline, rate-limited… It never throws: a failed check must
/// stay invisible, never disturb the widget.
export async function checkForUpdate(current, fetchImpl = fetch) {
  try {
    const res = await fetchImpl(RELEASES_API, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!res.ok) return null;
    return updateFromRelease(await res.json(), current);
  } catch {
    return null;
  }
}
