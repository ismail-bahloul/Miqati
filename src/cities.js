// City search behind the settings picker.
//
// Pure and dependency-free so it can be unit-tested with `node --test`: the
// settings window feeds it the bundled `cities.json` (see
// scripts/gen_cities.py). Entries there are `[name, cc, lat, lon, tzIndex]`
// plus an optional region, already sorted by population descending.

/// Lowercase, strip diacritics and drop separators, so "Tétouan" matches
/// "tetouan", "Fès" matches "fes" and "El Jadida" matches "eljadida".
export function normalize(value) {
  return String(value)
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "");
}

/// Build the searchable index from the raw `cities.json` payload. The
/// normalized key is precomputed because search runs on every keystroke.
export function buildIndex(data) {
  const tz = data.tz ?? [];
  const cities = (data.cities ?? []).map((c, i) => ({
    name: c[0],
    cc: c[1],
    lat: c[2],
    lon: c[3],
    tz: tz[c[4]] ?? "",
    region: c[5] ?? "",
    key: normalize(c[0]),
    rank: i,
  }));
  return { cities };
}

/// Localized country name ("MA" -> "Maroc"), falling back to the raw code.
export function countryName(cc, lang) {
  try {
    return new Intl.DisplayNames([lang], { type: "region" }).of(cc) ?? cc;
  } catch {
    return cc;
  }
}

/// Rank matches for `query`. Name-prefix matches come first, then the rest;
/// within each group the index order is kept, i.e. population descending. An
/// empty query returns nothing — the field is a city search box, not a browser.
export function search(index, query, limit = 12) {
  const q = normalize(query);
  if (!q) return [];
  const prefix = [];
  const rest = [];
  for (const city of index.cities) {
    if (city.key.startsWith(q)) {
      prefix.push(city);
      // Prefix matches always outrank substring ones, and the caller only ever
      // shows `limit` of them: no need to keep scanning.
      if (prefix.length >= limit) break;
    } else if (rest.length < limit && city.key.includes(q)) {
      rest.push(city);
    }
  }
  return prefix.concat(rest).slice(0, limit);
}
