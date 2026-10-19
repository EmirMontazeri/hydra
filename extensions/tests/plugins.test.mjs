// Copyright (C) 2026 Javad Rajabzadeh
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Plugin hand-off: which pages and links the extension offers to a Hydra
// plugin, read from the claims the app mirrors, and which build withholds
// which hosts. Driven through the real background scripts and content script.
// Run from the repository root:  node extensions/tests/plugins.test.mjs
import { readFileSync } from "node:fs";
import vm from "node:vm";
import { loadBackground } from "./load.mjs";

let fails = 0;
const eq = (label, got, want) => {
  if (JSON.stringify(got) === JSON.stringify(want)) console.log(`ok   ${label}`);
  else {
    fails++;
    console.log(`FAIL ${label}\n  got  ${JSON.stringify(got)}\n  want ${JSON.stringify(want)}`);
  }
};
const tick = (n = 4) =>
  new Promise((r) => {
    let i = 0;
    const f = () => (++i >= n ? r() : setImmediate(f));
    setImmediate(f);
  });

const PLUGINS = [
  {
    id: "hydra.torrent",
    name: "Torrent Downloader",
    claims: ["magnet:*", "*://*/*.torrent", "*://*/*.torrent?*"],
  },
  {
    id: "hydra.youtube",
    name: "YouTube Downloader",
    claims: ["*://*.youtube.com/watch*", "*://youtube.com/watch*", "*://youtu.be/*"],
  },
  { id: "rogue", name: "Rogue", claims: ["https:*", "ftp:*"] },
];

function build(browser, { enabled = true } = {}) {
  const store = { enabled };
  const sent = [];
  const ev = () => {
    const l = [];
    return { addListener: (f) => l.push(f), l };
  };
  const storage = () => ({
    async get(def) {
      if (typeof def === "string") return { [def]: store[def] };
      const out = {};
      for (const [k, v] of Object.entries(def)) out[k] = k in store ? store[k] : v;
      return out;
    },
    async set(patch) {
      Object.assign(store, patch);
    },
    async remove() {},
  });
  const onMessage = ev();
  const onUpdated = ev();
  const told = [];
  const chrome = {
    runtime: {
      id: "hydra-ext",
      onInstalled: ev(),
      onStartup: ev(),
      onMessage,
      getURL: (p) => p,
      sendNativeMessage: (_h, _m, cb) => cb(undefined),
    },
    storage: { local: storage(), session: storage() },
    action: { setBadgeBackgroundColor: async () => {}, setBadgeText: async () => {}, setTitle: async () => {} },
    downloads: { onCreated: ev(), onDeterminingFilename: ev() },
    contextMenus: { removeAll: async () => {}, create: () => {}, onClicked: ev() },
    tabs: {
      query: async () => [],
      sendMessage: async (tabId, msg) => told.push([tabId, msg.type]),
      onRemoved: ev(),
      onUpdated,
    },
    cookies: { getAll: async () => [{ name: "sid", value: "abc" }] },
    webRequest: { onResponseStarted: ev(), onHeadersReceived: ev(), onBeforeRedirect: ev() },
  };
  class FakeWS {
    constructor() {
      setImmediate(() => this.onopen && this.onopen());
    }
    send(raw) {
      const msg = JSON.parse(raw);
      sent.push(msg);
      setImmediate(
        () => this.onmessage && this.onmessage({ data: JSON.stringify({ ok: true, id: msg.id, plugins: PLUGINS }) })
      );
    }
    close() {}
  }
  const ctx = vm.createContext({
    chrome,
    navigator: { userAgent: "Mozilla/5.0 Chrome/120.0.0.0" },
    WebSocket: FakeWS,
    console,
    setTimeout,
    clearTimeout,
    setInterval,
    clearInterval,
    setImmediate,
    fetch: async () => {
      throw new Error("no net");
    },
    AbortController,
    URL,
    URLSearchParams,
    TextDecoder,
  });
  loadBackground(ctx, browser);
  const send = (msg, tab = { id: 7, url: "https://tracker.example/t/1" }) =>
    new Promise((res) => onMessage.l[0](msg, { tab }, res));
  return { ctx, store, sent, send, told, onUpdated };
}

// ------------------------------------------------- 1. the claim matcher
{
  const { ctx } = build("firefox");
  const m = (pattern, url) => ctx.claimMatches(pattern, url);
  eq("a whole-scheme claim takes a magnet", m("magnet:*", "magnet:?xt=urn:btih:abc"), true);
  eq("a whole-scheme claim is that scheme only", m("magnet:*", "https://a.example/magnet"), false);
  eq("* scheme means http", m("*://*/*.torrent", "http://a.example/f.torrent"), true);
  eq("* scheme is not ftp", m("*://*/*.torrent", "ftp://a.example/f.torrent"), false);
  eq("the path glob must reach the end", m("*://*/*.torrent", "https://a.example/f.torrent.html"), false);
  eq("the query is part of the path", m("*://*/*.torrent?*", "https://a.example/f.torrent?k=1"), true);
  eq("*.host takes a subdomain", m("*://*.youtube.com/watch*", "https://www.youtube.com/watch?v=x"), true);
  eq("*.host is not the bare host", m("*://*.youtube.com/watch*", "https://youtube.com/watch?v=x"), false);
  eq("*.host is not a lookalike", m("*://*.youtube.com/watch*", "https://evilyoutube.com/watch"), false);
  eq("a port must match when named", m("*://a.example:8080/*", "http://a.example/x"), false);
  eq("a named port matches", m("*://a.example:8080/*", "http://a.example:8080/x"), true);
  eq("a default port counts as named", m("*://a.example:443/*", "https://a.example/x"), true);
  eq("regex characters in a claim are literal", m("*://a.example/f(1).torrent", "https://a.example/f1.torrent"), false);
  eq("an unparseable address matches nothing", m("magnet:*", "not a url"), false);
}

// --------------------------------- 2. what a page is offered, per build
for (const [browser, youtube] of [
  ["chrome", null],
  ["firefox", { id: "hydra.youtube", name: "YouTube Downloader" }],
]) {
  const h = build(browser);
  await tick();
  const watch = await h.send({ type: "page-plugin", url: "https://www.youtube.com/watch?v=x" });
  eq(`${browser}: the YouTube page button`, watch.page, youtube);
  eq(`${browser}: link schemes leave web schemes to the browser`, watch.schemes, ["magnet"]);
  const other = await h.send({ type: "page-plugin", url: "https://example.com/" });
  eq(`${browser}: an unclaimed page gets no button`, other.page, null);
  const hand = await h.send({ type: "plugin-url", url: "https://youtu.be/x" });
  eq(`${browser}: a YouTube hand-off`, !!hand.ok, youtube !== null);
}

// --------------------------------------- 3. the hand-off itself
{
  const h = build("chrome");
  await tick();
  eq("a reply's plugin list is mirrored", h.store.plugins?.length, PLUGINS.length);
  const reply = await h.send({ type: "plugin-url", url: "magnet:?xt=urn:btih:abc", referer: "https://tracker.example/t/1" });
  eq("a magnet hand-off is taken", reply.ok, true);
  const msg = h.sent.find((m) => m.type === "plugin-url");
  eq("it carries the address", msg?.url, "magnet:?xt=urn:btih:abc");
  eq("it carries the page", msg?.referer, "https://tracker.example/t/1");
  eq("a magnet carries no cookies", msg?.cookies, null);
  const torrent = await h.send({ type: "plugin-url", url: "https://cdn.example/f.torrent" });
  eq("a .torrent hand-off is taken", torrent.ok, true);
  eq("a web address carries its cookies", h.sent.filter((m) => m.type === "plugin-url").pop()?.cookies, "sid=abc");
  const before = h.sent.length;
  const refused = await h.send({ type: "plugin-url", url: "https://example.com/page" });
  eq("an unclaimed address is refused here", refused.ok, false);
  eq("and never reaches Hydra", h.sent.length, before);

  h.onUpdated.l[0](7, { url: "https://www.youtube.com/watch?v=next" });
  await tick();
  eq(
    "a same-page navigation is told to the tab",
    h.told.filter(([, type]) => type === "location-changed"),
    [[7, "location-changed"]]
  );
}
{
  const h = build("firefox", { enabled: false });
  h.store.plugins = PLUGINS;
  const page = await h.send({ type: "page-plugin", url: "https://www.youtube.com/watch?v=x" });
  eq("switched off: no button", page.page, null);
  eq("switched off: no links", page.schemes, []);
  eq("switched off: no hand-off", (await h.send({ type: "plugin-url", url: "magnet:?xt=1" })).ok, false);
}

// --------------------------------------- 4. which clicks the page hands over
{
  const src = readFileSync("extensions/chrome/content.js", "utf8");
  const at = src.indexOf("function pluginLinkUrl(");
  const fn = new Function(src.slice(at, src.indexOf("\n}\n", at) + 3) + "return pluginLinkUrl;")();
  const click = ({ href = "magnet:?xt=urn:btih:abc", ...rest } = {}) => ({
    button: 0,
    altKey: false,
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    defaultPrevented: false,
    target: { closest: () => (href === null ? null : { href }) },
    ...rest,
  });
  const schemes = ["magnet"];
  eq("a plain click on a magnet is handed over", fn(click(), schemes), "magnet:?xt=urn:btih:abc");
  eq("the scheme is case-blind", fn(click({ href: "MAGNET:?xt=1" }), schemes), "MAGNET:?xt=1");
  eq("an http link is the browser's", fn(click({ href: "https://a.example/" }), schemes), null);
  eq("Alt leaves it to the browser", fn(click({ altKey: true }), schemes), null);
  eq("a modified click is the browser's", fn(click({ metaKey: true }), schemes), null);
  eq("a middle-click is the browser's", fn(click({ button: 1 }), schemes), null);
  eq("the page's own handler wins", fn(click({ defaultPrevented: true }), schemes), null);
  eq("no plugin, no hand-off", fn(click(), []), null);
  eq("a click on no link is left alone", fn(click({ href: null }), schemes), null);
}

console.log(fails ? `\n${fails} FAILED` : "\nall passed");
process.exit(fails ? 1 : 0);
