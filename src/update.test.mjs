// Tests for the update check (`update.js`).
//
//   node --test "src/*.test.mjs"
import { test } from "node:test";
import assert from "node:assert/strict";

import { parseVersion, isNewer, updateFromRelease, checkForUpdate, RELEASES_API } from "./update.js";

test("parseVersion reads plain and v-prefixed versions", () => {
  assert.deepEqual(parseVersion("0.2.1"), [0, 2, 1]);
  assert.deepEqual(parseVersion("v1.10.3"), [1, 10, 3]);
  assert.deepEqual(parseVersion(" v2.0.0-rc.1"), [2, 0, 0]);
  assert.equal(parseVersion("nightly"), null);
  assert.equal(parseVersion(""), null);
  assert.equal(parseVersion(undefined), null);
});

test("isNewer compares major, minor and patch", () => {
  assert.equal(isNewer("0.2.2", "0.2.1"), true);
  assert.equal(isNewer("0.3.0", "0.2.9"), true);
  assert.equal(isNewer("1.0.0", "0.9.9"), true);
  // Equal, older, or unparsable: never "newer".
  assert.equal(isNewer("0.2.1", "0.2.1"), false);
  assert.equal(isNewer("0.1.9", "0.2.0"), false);
  assert.equal(isNewer("nope", "0.2.1"), false);
  assert.equal(isNewer("0.2.2", "nope"), false);
});

test("updateFromRelease reports only a strictly newer tag", () => {
  const release = { tag_name: "v0.2.2", html_url: "https://github.com/x/y/releases/tag/v0.2.2" };
  assert.deepEqual(updateFromRelease(release, "0.2.1"), {
    version: "0.2.2",
    url: "https://github.com/x/y/releases/tag/v0.2.2",
  });
  assert.equal(updateFromRelease(release, "0.2.2"), null);
  assert.equal(updateFromRelease(release, "0.3.0"), null);
  assert.equal(updateFromRelease({}, "0.2.1"), null);
  assert.equal(updateFromRelease(null, "0.2.1"), null);
});

test("checkForUpdate returns the newer release from the API", async () => {
  const fetchImpl = async (url) => {
    assert.equal(url, RELEASES_API);
    return { ok: true, json: async () => ({ tag_name: "v0.2.3" }) };
  };
  assert.deepEqual(await checkForUpdate("0.2.1", fetchImpl), {
    version: "0.2.3",
    url: "https://github.com/ismail-bahloul/Miqati/releases/latest",
  });
});

test("checkForUpdate stays quiet when there is nothing to say", async () => {
  // No published release yet (404), same version, or a network failure.
  const notFound = async () => ({ ok: false, status: 404 });
  assert.equal(await checkForUpdate("0.2.1", notFound), null);

  const same = async () => ({ ok: true, json: async () => ({ tag_name: "v0.2.1" }) });
  assert.equal(await checkForUpdate("0.2.1", same), null);

  const offline = async () => {
    throw new Error("offline");
  };
  assert.equal(await checkForUpdate("0.2.1", offline), null);
});
