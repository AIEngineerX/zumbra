// X kit: post-ready images generated from brand/mark.svg and the tokens in docs/BRAND.md.
// Run: node brand/tools/xkit.mjs   (repo root). Output: .private/xkit/images/ (gitignored on purpose:
// the kit is the owner's private posting material, the generator is the reproducible part).

import { Resvg } from "@resvg/resvg-js";
import { readFileSync, writeFileSync, mkdirSync, copyFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..", "..");
const out = join(root, ".private", "xkit", "images");
mkdirSync(out, { recursive: true });

const VOID = "#0B0D12", GOLD = "#F2C14E", PALE = "#F4F1EA", MIST = "#9AA3B2";

const markSvg = readFileSync(join(root, "brand", "mark.svg"), "utf8");
const markInner = markSvg.replace(/^[\s\S]*?<svg[^>]*>/, "").replace(/<\/svg>\s*$/, "");
const mark = (x, y, size) => `<g transform="translate(${x} ${y}) scale(${size / 512})">${markInner}</g>`;

const fonts = {
  fontFiles: [join(root, "brand", "fonts", "Inter.ttf"), join(root, "brand", "fonts", "JetBrains-Mono.ttf")],
  loadSystemFonts: false, defaultFontFamily: "Inter",
};
const render = (svg, width, file) => {
  const png = new Resvg(svg, { fitTo: { mode: "width", value: width }, font: fonts }).render().asPng();
  writeFileSync(join(out, file), png);
};
const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;");
const word = (x, y, size, fill = PALE) =>
  `<text x="${x}" y="${y}" font-family="Inter" font-weight="800" font-size="${size}" letter-spacing="${size * 0.16}" fill="${fill}"><tspan fill="${GOLD}">Z</tspan>UMBRA</text>`;
const mono = (x, y, size, text, fill = MIST, anchor = "start") =>
  `<text x="${x}" y="${y}" font-family="JetBrains Mono" font-weight="500" font-size="${size}" letter-spacing="${size * 0.14}" fill="${fill}" text-anchor="${anchor}">${esc(text)}</text>`;
const body = (x, y, size, text, fill = PALE, anchor = "start", weight = 800) =>
  `<text x="${x}" y="${y}" font-family="Inter" font-weight="${weight}" font-size="${size}" fill="${fill}" text-anchor="${anchor}">${esc(text)}</text>`;
const lines = (x, y, size, gap, arr, fill = PALE, anchor = "start", weight = 800) =>
  arr.map((t, i) => body(x, y + i * gap, size, t, fill, anchor, weight)).join("");
// A faint star field so cards are not flat: deterministic pseudo-random, no dependency.
const stars = (w, h, n, seed = 7) => {
  let s = seed; const rnd = () => (s = (s * 16807) % 2147483647) / 2147483647;
  let o = "";
  for (let i = 0; i < n; i++) { const r = rnd() < 0.85 ? 1 : 1.8; o += `<circle cx="${(rnd() * w).toFixed(1)}" cy="${(rnd() * h).toFixed(1)}" r="${r}" fill="${PALE}" opacity="${(0.08 + rnd() * 0.25).toFixed(2)}"/>`; }
  return o;
};
const doc = (w, h, inner, withStars = true) =>
  `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}"><rect width="${w}" height="${h}" fill="${VOID}"/>${withStars ? stars(w, h, Math.round((w * h) / 9000)) : ""}${inner}</svg>`;

// 1. Announcement card 1600×900 (16:9, X's preferred single-image ratio)
render(doc(1600, 900,
  mark(640, 150, 320) +
  word(800, 620, 120, PALE).replace(`x="800"`, `x="800" text-anchor="middle"`) +
  mono(800, 690, 26, "SHIELDED ZCASH FOR HUMANS AND AGENTS", MIST, "middle") +
  mono(800, 800, 20, "SHIELDED BY DEFAULT · KEYS ON YOUR MACHINE · POLICY THE AGENT CANNOT TOUCH", MIST, "middle")
), 1600, "announce-1600x900.png");

// 2. Square 1080×1080 (feeds, profiles)
render(doc(1080, 1080,
  mark(340, 200, 400) +
  word(540, 760, 104, PALE).replace(`x="540"`, `x="540" text-anchor="middle"`) +
  mono(540, 830, 24, "SHIELDED ZCASH FOR HUMANS AND AGENTS", MIST, "middle")
), 1080, "square-1080.png");

// 3. "What it is" card: three lines, no marketing
render(doc(1600, 900,
  mark(120, 120, 160) +
  word(320, 235, 88) +
  lines(120, 420, 56, 96, [
    "A Zcash wallet an AI agent can hold.",
    "Shielded sends only. Keys never leave the machine.",
    "Policy runs before any key is loaded.",
  ], PALE, "start", 700) +
  mono(120, 800, 20, "PRE-ALPHA · MIT · OPEN SOURCE", MIST)
), 1600, "what-it-is-1600x900.png");

// 4. "How a send works" card: the two-step protocol
render(doc(1600, 900,
  mark(120, 120, 160) +
  word(320, 235, 88) +
  body(120, 400, 40, "How an agent sends", GOLD, "start", 800) +
  lines(120, 490, 40, 70, [
    "1  Agent proposes: address, amount, memo. No seed is loaded.",
    "2  Policy runs: per-tx cap, daily cap, allowlist. Fails closed.",
    "3  Above threshold: a human approves on a channel the agent cannot call.",
    "4  Only then is the seed read, the tx signed, broadcast, and logged.",
  ], PALE, "start", 600) +
  mono(120, 820, 20, "DESIGN INTENT · SEE SECURITY.MD FOR WHAT HOLDS TODAY", MIST)
), 1600, "how-a-send-works-1600x900.png");

// 5b. Security card: what holds and what does not, verbatim register of SECURITY.md
const col = (x, y, title, items, titleFill) =>
  mono(x, y, 20, title, titleFill) + items.map((t, i) => mono(x, y + 50 + i * 44, 20, t, PALE)).join("");
render(doc(1600, 900,
  mark(96, 84, 96) + word(220, 154, 56) +
  mono(96, 300, 20, "SECURITY.MD, THE SHORT VERSION", MIST) +
  col(96, 380, "HOLDS TODAY", [
    "seed only in an encrypted vault",
    "policy before any key, on every path",
    "missing or corrupt policy: nothing spends",
    "above threshold: you sign at a terminal",
    "every refusal logged",
  ], GOLD) +
  col(860, 380, "DOES NOT HOLD YET", [
    "no independent audit",
    "zero in the policy means unlimited",
    "policy file: only as safe as your OS user",
    "rate limit is per process on the CLI",
    "so: testnet, and nothing you cannot lose",
  ], MIST) +
  mono(96, 840, 18, "GITHUB.COM/AIENGINEERX/ZUMBRA/BLOB/MAIN/SECURITY.MD", MIST)
), 1600, "security-holds-1600x900.png");

// 5c. Agent card: what the agent can and cannot do over MCP
render(doc(1600, 900,
  mark(96, 84, 96) + word(220, 154, 56) +
  mono(96, 300, 20, "WHAT THE AGENT CAN AND CANNOT DO", MIST) +
  col(96, 380, "CAN", [
    "read balance, addresses, history",
    "propose a send within the policy",
    "confirm it, seed read last",
    "pay an x402 paywall within the policy",
    "lock the wallet",
  ], GOLD) +
  col(860, 380, "CANNOT", [
    "read or change the policy",
    "approve its own above-threshold send",
    "unlock a locked wallet",
    "see, print or export the seed",
    "raise a limit, ever",
  ], MIST) +
  mono(96, 840, 18, "15 MCP TOOLS · STDIO · HERMES, OPENCLAW, CLAUDE, CURSOR", MIST)
), 1600, "agent-can-cannot-1600x900.png");

// 5d. Policy card: the real default policy.toml a fresh wallet gets
render(doc(1600, 900,
  mark(96, 84, 96) + word(220, 154, 56) +
  mono(96, 300, 20, "THE POLICY IS A FILE YOU WRITE. THE AGENT HAS NO TOOL FOR IT.", MIST) +
  mono(96, 380, 22, "$ cat ~/.zumbra/testnet/policy.toml", GOLD) +
  [
    "max_per_tx = 1000000          # 0.01 ZEC",
    "daily_limit = 10000000        # 0.1 ZEC per rolling day",
    "approval_threshold = 5000000  # above this, you sign it",
    "min_spend_interval_ms = 0",
    "require_context_id = false",
    "allowlist = []                # empty = any shielded address",
  ].map((t, i) => mono(96, 450 + i * 50, 26, t, PALE)).join("") +
  mono(96, 840, 18, "VALUES IN ZATOSHI · 1 ZEC = 100 000 000 ZAT · WRITTEN BY ZUMBRA WALLET INIT", MIST)
), 1600, "policy-file-1600x900.png");

// 6. Quote/blank card: mark and wordmark small top-left, room for text you type in the post
render(doc(1600, 900, mark(96, 96, 112) + word(240, 180, 64) + mono(96, 820, 20, "ZUMBRA", MIST), true), 1600, "blank-1600x900.png");

// 7. Copies of the profile assets from brand/dist so the kit is one folder
for (const f of ["x-avatar-400.png", "x-header-1500x500.png", "og-1200x630.png"]) {
  const src = join(root, "brand", "dist", f);
  if (existsSync(src)) copyFileSync(src, join(out, f));
}

console.log("xkit/images written: announce, square, what-it-is, how-a-send-works, security-holds, agent-can-cannot, policy-file, blank, plus avatar/header/og copies");
