// Tests for the settings city picker search (`cities.js`).
//
//   node --test src/
import { test } from "node:test";
import assert from "node:assert/strict";

import { normalize, buildIndex, countryName, search } from "./cities.js";

test("normalize folds case, accents and separators", () => {
  assert.equal(normalize("Tétouan"), "tetouan");
  assert.equal(normalize("Fès"), "fes");
  assert.equal(normalize("El Jadida"), "eljadida");
  assert.equal(normalize("  Al Hoceïma "), "alhoceima");
  assert.equal(normalize("Oued Zem"), "ouedzem");
});

test("buildIndex resolves the timezone index and keeps the region optional", () => {
  const index = buildIndex({
    tz: ["Africa/Casablanca", "Europe/Paris"],
    cities: [
      ["Casablanca", "MA", 33.5883, -7.6114, 0],
      ["Springfield", "US", 39.78, -89.65, 1, "Illinois"],
    ],
  });
  assert.equal(index.cities.length, 2);
  assert.equal(index.cities[0].tz, "Africa/Casablanca");
  assert.equal(index.cities[0].region, "");
  assert.equal(index.cities[1].tz, "Europe/Paris");
  assert.equal(index.cities[1].region, "Illinois");
});

test("search is accent- and case-insensitive", () => {
  const index = buildIndex({
    tz: ["Africa/Casablanca"],
    cities: [["Tétouan", "MA", 35.57, -5.37, 0]],
  });
  assert.equal(search(index, "tetouan").length, 1);
  assert.equal(search(index, "TÉTOUAN").length, 1);
});

test("search returns nothing for an empty query", () => {
  const index = buildIndex({ tz: [], cities: [["Rabat", "MA", 34.02, -6.83, 0]] });
  assert.deepEqual(search(index, "   "), []);
  assert.deepEqual(search(index, ""), []);
});

test("search ranks name-prefix matches before substring ones", () => {
  // "Busan" comes first in the file (bigger city) but only *contains* "san";
  // the prefix match must still win.
  const index = buildIndex({
    tz: [],
    cities: [
      ["Busan", "KR", 35.1, 129.0, 0],
      ["San Francisco", "US", 37.77, -122.42, 0],
    ],
  });
  assert.deepEqual(
    search(index, "san").map((c) => c.name),
    ["San Francisco", "Busan"],
  );
});

test("search keeps at most `limit` results", () => {
  const cities = Array.from({ length: 30 }, (_, i) => [`City${i}`, "XX", 0, 0, 0]);
  const index = buildIndex({ tz: [], cities });
  assert.equal(search(index, "city", 5).length, 5);
});

test("countryName is localized and falls back to the code", () => {
  assert.equal(countryName("MA", "fr"), "Maroc");
  assert.equal(countryName("MA", "en"), "Morocco");
  assert.equal(countryName("XX", "fr"), "XX");
});
