---
format: aep.planning-md/3
id: review-result:wave6-explorer-look-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:explorer-look
relations:
- reviews: story:explorer-look
revision: 1
---
The look itself is correct. It still fails one case: a visitor in dark mode gets a light page until the app's JavaScript runs.

```
unit: story:explorer-look, uncommitted working tree of a local path (detached at 24ccbc4)
verdict: CONFIRMED
cases: executed 82→86, red 1
origin: introduced 2 / pre-existing 1 / undecided 0
wrote-outside-worktree: none by hand (package-manager caches only, see 6)
needs-coordinator: none
```

**1. `git --no-pager diff --stat`** (tracked files; everything here is the unit's own work)
```
 explorer/src/App.vue   |  28 +++-
 explorer/src/style.css | 389 +++++++++++++++++++++++++++++++++++++++++++++++--
```
I added only three untracked test files: `explorer/tests/adversary_look_flash.test.ts`, `adversary_look_contrast.test.ts` and `adversary_look_storage_key.test.ts`. No implementation file was touched, and nothing outside `explorer/` changed.

**2. The cases I added**
- **`adversary_look_flash.test.ts` (red).** It asserts that `explorer/index.html` sets `data-theme` in an inline head script, as the docs site does, or that `style.css` has a `prefers-color-scheme: dark` fallback. Red output from running it alone:
  ```
  FAIL  tests/adversary_look_flash.test.ts > ... > sets data-theme in the head, as the documentation site does, or has a dark fallback without it
  AssertionError: data-theme is only set when App.vue runs from the deferred module script; until then :root paints the light canvas, so a visitor whose stored or system mode is dark sees a light page first: expected false to be true
  ```
- **`adversary_look_contrast.test.ts` (green).** Text and background pairs meet WCAG AA (at least 4.5:1) in both themes: text, heading, muted, link and danger on the page background; text on surfaces; button text on both button colours; code on its background.
- **`adversary_look_storage_key.test.ts` (green).** It derives the key from `url` and `baseUrl` in `docusaurus.config.ts` the way Docusaurus does: `theme-` plus the first three hex digits of the md5 of `https://beyond10x.github.io/rfcspec/`. That gives `theme-cdd`, and `App.vue` uses exactly that. It stays in the suite as a drift guard.

**3. Suite run** (after the cases existed): `task explorer`. Typecheck passed; vitest gave `Test Files 1 failed | 13 passed (14)`, `Tests 1 failed | 85 passed (86)`, and the task failed with `exit status 1`. Because the tests failed, `pnpm build` did not run, so `website/build` is still the implementor's build. With my three files excluded the suite gives 82 passed.

**4. Findings**

| file:line | verdict | origin | finding | what reaches it |
|---|---|---|---|---|
| `explorer/src/App.vue:47` (and `explorer/index.html`) | CONFIRMED | introduced | The page is light by default and only switches to dark once the app's module script runs. The docs site avoids this with an inline head script that reads `theme-cdd`; the explorer has none. Fix: the same inline script in `explorer/index.html`'s `<head>`. That file is outside the story's listed scope, so the coordinator may need to widen it. | Every visitor whose saved or system setting is dark. I checked the HTML and CSS but did not watch the flash in a browser (no headless Chromium here). |
| `explorer/src/components/SiteHeader.vue:27` | CONFIRMED | introduced | The docs navbar has a light/dark switch on the right; the explorer header has only GitHub. The explorer follows the stored choice but cannot change it. | Any visitor comparing the explorer header with the docs header. |
| `website/package.json:10` (`docusaurus serve`) | CONFIRMED | pre-existing | With `trailingSlash: false`, `docusaurus serve` redirects `/rfcspec/explorer/` (302) to `/rfcspec/explorer`. There `./assets/index-BS7bt4NH.js` resolves to `/rfcspec/assets/…` and returns 404, so the whole explorer fails to load, not only the `../` links. | `npm run serve` locally. GitHub Pages instead adds the trailing slash to the directory URL, so production loads. |

**5. Checked and found working**
- **Links from `/rfcspec/explorer/` and `/rfcspec/explorer/?rfc=6455`:** they resolve to the built files (`docs.html`, `docs/design.html`, `docs/catalog/rfcs.html` and the rest), and `look.test.ts` checks them against the config.
- **The no-slash form `/rfcspec/explorer`:** there `../docs` would go to `/docs`. But the only server I found that serves that form breaks the whole page, which is the pre-existing row above.
- **`theme-cdd`:** it is stable. Docusaurus derives it from `url` plus `baseUrl` (`@docusaurus/core/lib/server/storage.js`, with `future.v4` on), and the built head script reads exactly that key. It would only change if `url` or `baseUrl` changes, and the new guard test would catch that.
- **Colours:** every token in `style.css`, light and dark, matches the docs site's built CSS (`html[data-theme=light],:root` and `html[data-theme=dark]` blocks).
- **Elements the old CSS styled:** all still have colours (`.note`, `.missing`, `.filters`, tables, code, the old `nav` now `.toolbar`). `.meta` and `.rfc` were unstyled before and still are.
- **Contrast:** all checked pairs pass AA in both themes.
- **Fonts:** the built CSS uses `url(/rfcspec/b10x-fonts/…)`, the same paths the docs site preloads, and all three files exist in `website/build/b10x-fonts/`.
- **Existing features** (search, filters, the RFC view, notes, missing states): all 82 existing cases pass.
- **The footer:** columns, links and copyright match the config, and `look.test.ts` checks them.

**6. Paths written outside the worktree:** none by hand. `npx`/`pnpm` may have touched their user caches (`~/.npm`, the pnpm store). `docusaurus serve` ran on port 8752; it is no longer listening (curl returns 000), though `kill` on the PID I recorded said "no such process".

```findings
- file: explorer/src/App.vue
  line: 47
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: data-theme is set only when the deferred app module runs, and index.html has no head script, so dark-mode visitors first see the light page
- file: explorer/src/components/SiteHeader.vue
  line: 27
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the explorer header omits the docs navbar's colour-mode switch, so the mode can be followed but not changed from the explorer
- file: website/package.json
  line: 10
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: docusaurus serve with trailingSlash false redirects /rfcspec/explorer/ to /rfcspec/explorer, where the relative ./assets/ script returns 404 and the explorer does not load
```
