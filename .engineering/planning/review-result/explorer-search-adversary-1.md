---
format: aep.planning-md/3
id: review-result:explorer-search-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:explorer-search
relations:
- reviews: story:explorer-search
revision: 1
---
I broke it in two places. The acceptance URL itself is correct, but two edge cases fail. Both defects are in new code, so their origin is `introduced`.

```
unit: story:explorer-search, uncommitted working tree on 1845f70 (the explorer-search worktree)
verdict: CONFIRMED
cases: executed 38→47, red 5
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none
```

**1. `git --no-pager diff --stat`**

It shows the implementor's 17 tracked files, the same as the brief and none of them mine. My only write is the untracked test file `explorer/tests/adversary-search.test.ts`. No non-test path is mine.

**2. Cases added** (`the explorer-search worktree/explorer/tests/adversary-search.test.ts`)

I ran this file alone, before the suite: 9 cases, 5 red, 4 green.

- **Red: "the controls show the filter in effect"** (4 cases: `status=internet standard`, `level=must`, `format=XML`, `stream=ietf`). Each opens the list from a link and asserts that the printed count matches what the visible selects would filter.
  ```
  AssertionError: expected '0 of 12 RFCs' to be '12 of 12 RFCs'
   ❯ tests/adversary-search.test.ts:115:21
  ```
- **Red: "reads every number ?rfc= reads, leading zeros included".**
  ```
  AssertionError: 006455: expected 'invalid' to deeply equal { from: 6455, to: 6455 }
   ❯ tests/adversary-search.test.ts:125:39
  ```
- **Green:**
  - Every row of `rfc-index.json` matches its summary row field by field: rfc, title, status, stream, area, date, format and counts. The profiles match a separate reading of `profiles/*/rfcspec.lock`, and the only marked RFCs are 868, 1035, 5905 and 6455.
  - For every status crossed with every level plus `none`, the list equals what a jq-style filter keeps from `bulk/rfc-summary.jsonl`.
  - The acceptance URL, opened in the App on the committed index, prints the jq count and links each row to `?rfc=<n>`.
  - The query string round-trips for 2000 random filters (fixed seed) containing `+ % & = # ? \n`, emoji, `ü` and `€`.

**3. Suite run** (after the cases existed)

`pnpm exec vitest run` in `explorer/`:
```
 Test Files  1 failed | 7 passed (8)
      Tests  5 failed | 42 passed (47)
EXIT=1
```
With `--exclude tests/adversary-search.test.ts`: `Tests  38 passed (38)`.

**4. Findings**

| file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|
| `explorer/src/components/ListView.vue:46` (selects), `explorer/src/lib/search.ts:68-74` | CONFIRMED | introduced | A query value that no option offers still filters the list, here down to 0. The select shows "any" and no message appears (`adversary-search.test.ts:115`). | A hand-written or edited link such as `?format=XML` (the table prints "XML") or `?status=Internet%20Standard`. There is no in-app path to it. Possible fixes: show a "not a known value" notice, as the number filter already does, or match case-insensitively. |
| `explorer/src/lib/search.ts:48` | CONFIRMED | introduced | `numberRange` takes 1 to 5 digits, so `006455` is "invalid", while `requestedRfc` accepts any leading zeros (`adversary-search.test.ts:125`). | Only someone typing 6 or more digits. The range pages pad to 5 digits (`06455`), and that form works. Possible fix: strip leading zeros before the length check. |

Judgement notes, with no failing case:
- **3 MiB cap headroom:** `rfc-index.json` is 2,655,752 bytes, not the 2,025,670 the story measured. The cap leaves about 490 KB of headroom.
- **Unfiltered list size:** the unfiltered list renders 9847 `<tr>` and re-renders on every keystroke. This is a performance note only.

**5. What I attacked and could not break**
- **Ranges:** reversed, garbage, `1e3`, `-5` and huge values are all refused. Spaces around `-` are accepted. Open ranges are not supported, and the story does not ask for them.
- **Title words:** they are AND-ed and case-insensitive (`ü` included).
- **Equality filters:** `+` and `%20` decode the same way.
- **Level filters:** `none` matches empty counts and zero counts; any other level means a count above 0.
- **Parameters:** `?rfc=` takes precedence over the list filters, and `?rfc=` with an empty value falls through to the list.
- **Back/forward:** `replaceState` creates no history entries, and the back button from `?rfc=` restores the filtered URL.
- **Drift checks:** they cover the index (the `main.rs` drift test and `crosscheck.test.ts`), and the fixture-equals-generated check compares real data.
- **XSS:** no `v-html` anywhere in `explorer/src`.
- **Content rules:** no `/home/` paths, no comments in config files, no Python in the diff.

**6. Paths written outside the worktree:** none.

```findings
- file: explorer/src/components/ListView.vue
  line: 46
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a query value no option offers (status=internet standard, level=must, format=XML, stream=ietf) filters the list to 0 while the select shows "any" and no notice is printed
- file: explorer/src/lib/search.ts
  line: 48
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: numberRange refuses 006455 as invalid while requestedRfc reads it as 6455, so the two number inputs disagree on leading zeros
```
