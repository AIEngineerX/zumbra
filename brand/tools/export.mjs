// Export every brand asset from brand/mark.svg and the tokens below.
// Run: node brand/tools/export.mjs   (from the repo root; needs `npm install` in brand/tools once)
// Output: brand/dist/. Deterministic: same inputs, same bytes. No image models anywhere.

import { Resvg } from "@resvg/resvg-js";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..", "..");
const out = join(root, "brand", "dist");
mkdirSync(out, { recursive: true });

// ── Tokens (mirror docs/BRAND.md; that file is the human-readable source) ──
const VOID = "#0B0D12";   // ground
const GOLD = "#F2C14E";   // the corona; the only accent
const PALE = "#F4F1EA";   // wordmark
const MIST = "#9AA3B2";   // secondary text

const markSvg = readFileSync(join(root, "brand", "mark.svg"), "utf8");
// The mark's own <svg> wrapper is replaced by a <g> so it can be placed inside compositions.
const markInner = markSvg.replace(/^[\s\S]*?<svg[^>]*>/, "").replace(/<\/svg>\s*$/, "");
const mark = (x, y, size) =>
  `<g transform="translate(${x} ${y}) scale(${size / 512})">${markInner}</g>`;

const fonts = {
  fontFiles: [join(root, "brand", "fonts", "Inter.ttf"), join(root, "brand", "fonts", "JetBrains-Mono.ttf")],
  loadSystemFonts: false,
  defaultFontFamily: "Inter",
};

function render(svg, width, file) {
  const r = new Resvg(svg, { fitTo: { mode: "width", value: width }, font: fonts, background: "transparent" });
  const png = r.render().asPng();
  writeFileSync(join(out, file), png);
  return png;
}

const wordmark = (x, y, size, fill = PALE) =>
  `<text x="${x}" y="${y}" font-family="Inter" font-weight="800" font-size="${size}" letter-spacing="${size * 0.16}" fill="${fill}"><tspan fill="${GOLD}">Z</tspan>UMBRA</text>`;
const tagline = (x, y, size, fill = MIST) =>
  `<text x="${x}" y="${y}" font-family="JetBrains Mono" font-weight="500" font-size="${size}" letter-spacing="${size * 0.14}" fill="${fill}">SHIELDED ZCASH FOR YOU AND YOUR AGENTS</text>`;

const doc = (w, h, body) =>
  `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">
  <rect width="${w}" height="${h}" fill="${VOID}"/>${body}</svg>`;

// ── 1. Mark alone, transparent ground, every size a platform asks for ──
for (const s of [16, 32, 48, 64, 128, 180, 192, 256, 512, 1024]) render(markSvg, s, `mark-${s}.png`);

// Favicon .ico: PNG-compressed entries for 16, 32, 48 (valid ICO since Vista; every browser reads it).
{
  const sizes = [16, 32, 48];
  const pngs = sizes.map((s) => readFileSync(join(out, `mark-${s}.png`)));
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); header.writeUInt16LE(1, 2); header.writeUInt16LE(sizes.length, 4);
  const dir = Buffer.alloc(16 * sizes.length);
  let offset = 6 + dir.length;
  sizes.forEach((s, i) => {
    const b = pngs[i];
    dir.writeUInt8(s === 256 ? 0 : s, i * 16 + 0);
    dir.writeUInt8(s === 256 ? 0 : s, i * 16 + 1);
    dir.writeUInt8(0, i * 16 + 2); dir.writeUInt8(0, i * 16 + 3);
    dir.writeUInt16LE(1, i * 16 + 4); dir.writeUInt16LE(32, i * 16 + 6);
    dir.writeUInt32LE(b.length, i * 16 + 8); dir.writeUInt32LE(offset, i * 16 + 12);
    offset += b.length;
  });
  writeFileSync(join(out, "favicon.ico"), Buffer.concat([header, dir, ...pngs]));
}

// ── 2. Mark on void (avatars) ──
render(doc(512, 512, mark(56, 56, 400)), 400, "x-avatar-400.png");
render(doc(512, 512, mark(56, 56, 400)), 512, "avatar-512.png");

// ── 3. X header 1500×500. The avatar overlaps the bottom-left ~ (0..400, 250..500) on desktop;
//       keep the lockup right of x=460 and vertically centred. ──
render(doc(1500, 500, `${mark(520, 150, 200)}${wordmark(760, 285, 112)}${tagline(766, 340, 22)}`), 1500, "x-header-1500x500.png");

// ── 4. GitHub social preview 1280×640 and Open Graph 1200×630: centred lockup ──
render(doc(1280, 640, `${mark(300, 208, 224)}${wordmark(560, 350, 120)}${tagline(566, 410, 24)}`), 1280, "github-social-1280x640.png");
render(doc(1200, 630, `${mark(260, 203, 224)}${wordmark(520, 345, 120)}${tagline(526, 405, 24)}`), 1200, "og-1200x630.png");

// ── 5. README banner 1280×320 ──
render(doc(1280, 320, `${mark(96, 64, 192)}${wordmark(330, 195, 108)}${tagline(336, 250, 22)}`), 1280, "readme-banner-1280x320.png");

// ── 6. Lockup SVG (vector, for the site) ──
writeFileSync(
  join(out, "lockup.svg"),
  `<svg xmlns="http://www.w3.org/2000/svg" width="720" height="160" viewBox="0 0 720 160">${mark(0, 8, 144)}${wordmark(176, 112, 96)}</svg>\n`
);

console.log("brand/dist written:", [
  "mark-{16..1024}.png", "favicon.ico", "x-avatar-400.png", "avatar-512.png",
  "x-header-1500x500.png", "github-social-1280x640.png", "og-1200x630.png",
  "readme-banner-1280x320.png", "lockup.svg",
].join(", "));
