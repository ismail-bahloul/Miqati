"""Generate `src/cities.json` — the city list behind the settings picker.

Source: GeoNames (https://www.geonames.org/), files `cities5000.zip` and
`admin1CodesASCII.txt`, licensed CC BY 4.0
(https://creativecommons.org/licenses/by/4.0/). The output is a derived
database: see the attribution in README.md.

Kept: **every Moroccan city** (the primary audience must always find its town)
plus cities of 50 000 inhabitants or more worldwide (so the picker can resolve
any plausible choice to coordinates + timezone + country).

Output shape — compact, and indexed so a country/timezone never repeats:

    {
      "tz": ["Africa/Casablanca", ...],
      "cities": [
        ["Casablanca", "MA", 33.5883, -7.6114, 12],
        ["Springfield", "US", 39.7817, -89.6501, 90, "Illinois"],
        ...
      ]
    }

Fields: name, ISO country code, latitude, longitude, timezone index, and the
region name **only** when the same city name occurs several times in that
country (to disambiguate). Entries are sorted by population descending, so the
picker can rank by importance without the population being shipped at all.

Run from the repository root:

    python3 scripts/gen_cities.py
"""

import io
import json
import urllib.request
import zipfile

BASE = "https://download.geonames.org/export/dump/"
MIN_POPULATION = 50_000
ALWAYS_KEEP_COUNTRY = "MA"  # Morocco: keep every city, whatever its size
OUT = "src/cities.json"

# GeoNames uses the English form for a few major Moroccan cities; French is the
# app's default language, so prefer the standard French spelling for those.
NAME_OVERRIDES = {
    "Fes": "Fès",
    "Meknes": "Meknès",
    "Kenitra": "Kénitra",
    "Marrakesh": "Marrakech",
    "Tangier": "Tanger",
    "Temara": "Témara",
    "Khemisset": "Khémisset",
    "Khenifra": "Khénifra",
}


def fetch(path: str) -> bytes:
    req = urllib.request.Request(BASE + path, headers={"User-Agent": "miqati-build/1.0"})
    with urllib.request.urlopen(req, timeout=120) as resp:
        return resp.read()


def load_rows() -> list[list[str]]:
    with zipfile.ZipFile(io.BytesIO(fetch("cities5000.zip"))) as zf:
        name = zf.namelist()[0]
        text = zf.read(name).decode("utf-8")
    return [line.split("\t") for line in text.splitlines() if line]


def load_regions() -> dict[str, str]:
    """admin1 code ("MA.05") -> region name ("Casablanca-Settat")."""
    out = {}
    for line in fetch("admin1CodesASCII.txt").decode("utf-8").splitlines():
        parts = line.split("\t")
        if len(parts) >= 2:
            out[parts[0]] = parts[1]
    return out


def main() -> None:
    regions = load_regions()
    rows = []
    for r in load_rows():
        cc = r[8]
        population = int(r[14] or 0)
        if cc != ALWAYS_KEEP_COUNTRY and population < MIN_POPULATION:
            continue
        tz = r[17]
        if not tz:
            continue
        rows.append(
            {
                "name": NAME_OVERRIDES.get(r[1], r[1]) if cc == "MA" else r[1],
                "cc": cc,
                "lat": round(float(r[4]), 4),
                "lon": round(float(r[5]), 4),
                "tz": tz,
                "pop": population,
                "region": regions.get(f"{cc}.{r[10]}", ""),
            }
        )

    # Sort by importance: the picker relies on this order to rank its matches.
    rows.sort(key=lambda c: c["pop"], reverse=True)

    # Only disambiguate names that actually collide inside a country.
    seen: dict[tuple[str, str], int] = {}
    for c in rows:
        key = (c["cc"], c["name"].casefold())
        seen[key] = seen.get(key, 0) + 1

    tz_index: dict[str, int] = {}
    tz_list: list[str] = []
    cities: list[list] = []
    for c in rows:
        if c["tz"] not in tz_index:
            tz_index[c["tz"]] = len(tz_list)
            tz_list.append(c["tz"])
        entry = [c["name"], c["cc"], c["lat"], c["lon"], tz_index[c["tz"]]]
        if seen[(c["cc"], c["name"].casefold())] > 1 and c["region"]:
            entry.append(c["region"])
        cities.append(entry)

    with open(OUT, "w", encoding="utf-8") as fh:
        json.dump({"tz": tz_list, "cities": cities}, fh, ensure_ascii=False, separators=(",", ":"))

    morocco = sum(1 for c in cities if c[1] == "MA")
    print(f"{len(cities)} cities ({morocco} in Morocco), {len(tz_list)} timezones -> {OUT}")


if __name__ == "__main__":
    main()
