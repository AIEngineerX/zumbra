/* Render docs/how-a-send-works.svg (SMIL-animated) to PNG frames by evaluating its own
   keyframes per frame, then ffmpeg turns the frames into a GIF and an MP4 for X and the site.
   Output: .private/xkit/images/how-a-send-works.{gif,mp4}. Run: node brand/tools/rail-frames.mjs */
import { Resvg } from "@resvg/resvg-js";
import { readFileSync, writeFileSync, mkdirSync, rmSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..", "..");
const src = join(root, "docs", "how-a-send-works.svg");
const outDir = join(root, ".private", "xkit", "images");
const frameDir = join(root, ".private", "xkit", "frames");
const FPS = 24, DUR = 10, WIDTH = 1600;

const fonts = {
  fontFiles: [join(root, "brand", "fonts", "Inter.ttf"), join(root, "brand", "fonts", "JetBrains-Mono.ttf")],
  loadSystemFonts: false,
  defaultFontFamily: "JetBrains Mono",
};

// cubic-bezier easing as SMIL keySplines / CSS define it: solve x(t)=p for t, return y(t)
function bezier(x1, y1, x2, y2) {
  const cx = 3 * x1, bx = 3 * (x2 - x1) - cx, ax = 1 - cx - bx;
  const cy = 3 * y1, by = 3 * (y2 - y1) - cy, ay = 1 - cy - by;
  const X = (t) => ((ax * t + bx) * t + cx) * t;
  const Y = (t) => ((ay * t + by) * t + cy) * t;
  return (p) => {
    let lo = 0, hi = 1, t = p;
    for (let i = 0; i < 40; i++) { const x = X(t); if (Math.abs(x - p) < 1e-5) break; if (x < p) lo = t; else hi = t; t = (lo + hi) / 2; }
    return Y(t);
  };
}

// one <animate> element -> function of time (seconds) -> value
function track(el) {
  const attr = (n) => (el.match(new RegExp(`${n}="([^"]*)"`)) || [])[1];
  const values = attr("values").split(";").map(Number);
  const keyTimes = attr("keyTimes") ? attr("keyTimes").split(";").map(Number) : values.map((_, i) => i / (values.length - 1));
  const splines = attr("keySplines") ? attr("keySplines").split(";").map((s) => bezier(...s.trim().split(/\s+/).map(Number))) : null;
  const dur = parseFloat(attr("dur"));
  return (t) => {
    const u = (t % dur) / dur;
    if (values.length === 1) return values[0];
    let i = 0;
    while (i < keyTimes.length - 2 && u >= keyTimes[i + 1]) i++;
    const span = keyTimes[i + 1] - keyTimes[i];
    let p = span > 0 ? (u - keyTimes[i]) / span : 1;
    p = Math.min(1, Math.max(0, p));
    if (splines) p = splines[i](p);
    return values[i] + (values[i + 1] - values[i]) * p;
  };
}

const svg = readFileSync(src, "utf8");
// Every <animate ...> belongs to the element that contains it; replace each element's animated
// attribute with the evaluated value and drop the <animate> nodes.
const animRe = /<animate\s[^>]*\/>/g;
const anims = [...svg.matchAll(animRe)].map((m) => ({ text: m[0], index: m.index, name: (m[0].match(/attributeName="([^"]+)"/) || [])[1], f: track(m[0]) }));

function frame(t) {
  let out = svg;
  // consume <animate> nodes from the last to the first, re-locating each in the current text
  // (editing an owning tag shifts everything after it), and set the evaluated attribute on the
  // nearest preceding <circle> or <rect>
  for (let k = anims.length - 1; k >= 0; k--) {
    const a = anims[k];
    const idx = out.lastIndexOf("<animate");
    const end = out.indexOf("/>", idx) + 2;
    const before = out.slice(0, idx);
    const tagStart = Math.max(before.lastIndexOf("<circle"), before.lastIndexOf("<rect"));
    const tagEnd = before.indexOf(">", tagStart);
    let tag = before.slice(tagStart, tagEnd);
    const val = a.f(t).toFixed(3);
    tag = tag.includes(` ${a.name}="`) ? tag.replace(new RegExp(` ${a.name}="[^"]*"`), ` ${a.name}="${val}"`) : `${tag} ${a.name}="${val}"`;
    out = before.slice(0, tagStart) + tag + before.slice(tagEnd) + out.slice(end);
  }
  return out;
}

if (process.env.DUMP) { writeFileSync(process.env.DUMP, frame(0.5)); process.exit(0); }
if (existsSync(frameDir)) rmSync(frameDir, { recursive: true });
mkdirSync(frameDir, { recursive: true });
mkdirSync(outDir, { recursive: true });
const n = FPS * DUR;
for (let i = 0; i < n; i++) {
  const png = new Resvg(frame(i / FPS), { fitTo: { mode: "width", value: WIDTH }, font: fonts }).render().asPng();
  writeFileSync(join(frameDir, `f${String(i).padStart(4, "0")}.png`), png);
}
console.log(`${n} frames at ${WIDTH}px in ${frameDir}`);

const ff = process.env.FFMPEG || "ffmpeg";
const input = ["-framerate", String(FPS), "-i", join(frameDir, "f%04d.png")];
execFileSync(ff, ["-y", ...input, "-vf", `fps=${FPS},scale=${WIDTH}:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=64[p];[s1][p]paletteuse=dither=bayer:bayer_scale=5`, "-loop", "0", join(outDir, "how-a-send-works.gif")], { stdio: "inherit" });
execFileSync(ff, ["-y", ...input, "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "20", "-movflags", "+faststart", "-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2", join(outDir, "how-a-send-works.mp4")], { stdio: "inherit" });
console.log("wrote how-a-send-works.gif and .mp4");
