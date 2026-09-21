# Reference: zipher.to (captured 2026-09-20)

Upstream's product page for Zipher, captured as a reference for our own site. We are not
contributing upstream; this folder exists so the shape can be reproduced without re-scraping.
Everything here was verified by fetching the live page and driving it in a browser on the capture
date. Nothing is inferred from memory.

## Files

| File | What |
|---|---|
| `index.html` | The page as served (single file, ~30 KB, Astro build) |
| `knowledge.json` | The 11 Q&A entries extracted from the inline `knowledgeJson` constant |
| `screenshot-home.png` | Full page at 1430 px wide, human mode, idle |
| `screenshot-answer.png` | Viewport after submitting "how do agents pay for things" |

## Hosting and build

- Host: Netlify (`Server: Netlify` response header).
- Build: Astro (`/_astro/` asset path). No framework runtime ships to the browser.
- External requests: Google Fonts only (Inter 400–700, JetBrains Mono 400–500). No analytics, no
  API calls, no third-party scripts. The whole page is one HTML document with inline CSS and JS.
- Related sites for contrast: `zipher.app` is a Framer page, `atmospherelabs.dev` is Astro on
  Netlify, `cipherscan.app` is Next.js on Vercel. None of their site sources are in a public repo.

## Structure (top to bottom)

1. Top-right segmented toggle, `Human | Agent`, `role=group` "Interface mode".
2. Centered mark: cyan "Z" glyph (a Z with a slash, reads as a stylised ZEC sign), ~64 px.
3. Wordmark `ZIPHER`, the leading Z in cyan, the rest in white, wide letter-spacing.
4. Tagline: "Zcash for humans and agents".
5. Eyebrow label `ASK ANYTHING` in mono, tracked out.
6. One pill-shaped input, search icon left, green gradient arrow button right, rotating
   placeholder text.
7. Five chips under the input: What is Zipher? · Download · Agent wallet · Privacy · Open source.
8. Answer panel (hidden until a query): mono label in cyan with the matched question, body text,
   optional CTA buttons.
9. Footer: TestFlight · Google Play · GitHub · zipher-cli · MCP · Atmosphere, then a tagline line.
10. Background: near-black with a faint radial teal glow behind the mark and a sparse dot field.

## Behaviour of the prompt box (verified by typing into it)

The box is **not an LLM and makes no network call**. The inline script does this:

1. Lower-cases the query, strips a stop-word list (`a an the is it to do i my me you your how what
   where can does are was be of in on for and or this that with about tell please`).
2. Scores each `knowledge.json` entry by keyword overlap against the entry's `keywords` array.
3. Renders the best entry's `question` as the panel label and its `answer` with a typewriter
   effect (an `AbortController` cancels the previous animation if you submit again).
4. If nothing matches it shows a fixed fallback answer.

Test performed: query "how do agents pay for things" matched the `agents` entry ("Can an AI agent
use this?") and typed out the answer about the MCP server. See `screenshot-answer.png`.

## Human vs Agent mode

The toggle sets `document.documentElement.dataset.mode = 'agent'`, persisted in
`localStorage['zipher-mode']` and also readable from `?mode=agent`. In agent mode the script swaps:

- placeholders (`AGENT_PLACEHOLDERS`) and chips (`AGENT_CHIPS`),
- the fallback answer,
- the footer tagline to `local-first · keys never leave the machine · exit 0`,
- and answers are rendered from a `CLI_COMMANDS` map, so the same question returns shell
  commands instead of prose.

## Knowledge base (11 entries)

| id | question | CTAs |
|---|---|---|
| what | What is Zipher? | TestFlight, Google Play |
| privacy | How is my money private? | — |
| safe | Is it safe? | View source |
| download | Where do I download it? | TestFlight, Google Play |
| opensource | Is it open source? | View source |
| agent-wallet | What is the agent wallet? | CLI readme |
| cli | What is zipher-cli? | npm package |
| agents | Can an AI agent use this? | MCP config |
| different | How is this different from other wallets? | — |
| action | What can Action do? | — |
| company | Who built this? | atmospherelabs.dev |

Full text and keyword lists are in `knowledge.json`. Note the `agents` answer claims "29 tools";
the June `main` code exposes 24 and the September ironwood branch fewer after the security
remediation. Their copy is ahead of their code.

## Design tokens observed

Measured from the screenshot and the inline CSS, not from a published token file.

- Background: near-black navy, roughly `#0b0f14`, with a radial teal glow centred on the mark.
- Accent: cyan `#22d3ee`-ish for the Z, labels, and the input focus ring.
- Action: green→teal gradient on the submit button.
- Text: white for the wordmark, light grey for body, dim grey for footer links.
- Type: Inter for UI, JetBrains Mono for eyebrow labels and the answer label.
- Radii: fully rounded input and chips, ~16 px on the answer panel.

## What to take and what to change

Take: the single-input "ask" pattern, the human/agent toggle, the local keyword knowledge base
(honest, zero-cost, no LLM bill), the six-link footer, the restraint.

Change: our own mark and name, our own knowledge entries written against what our code actually
does, our own links. Do not copy their copy. Do not ship a tool count our binary does not expose.
