/// <reference types="node" />
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';
import App from '../src/App.vue';
import ListView from '../src/components/ListView.vue';
import type { ExplorerIndex } from '../src/generated/types.ts';
import { EMPTY, type Filter, filterRows, numberRange, readFilter, writeFilter } from '../src/lib/search.ts';
import { requestedRfc } from '../src/lib/shard.ts';
import generated from '../../website/static/data/rfc-index.json';
import fixture from './fixtures/rfc-index.json';

type SummaryRow = {
  rfc: number;
  title: string;
  status: string;
  stream: string;
  area?: string;
  date: string;
  format?: string;
  counts: Record<string, number>;
};

const root = resolve(__dirname, '../..');
const summary: SummaryRow[] = readFileSync(resolve(root, 'bulk/rfc-summary.jsonl'), 'utf8')
  .split('\n')
  .slice(1)
  .filter((line) => line !== '')
  .map((line) => JSON.parse(line) as SummaryRow);
const rows = (generated as ExplorerIndex).rfcs;

function lockedBy(): Map<number, string[]> {
  const out = new Map<number, string[]>();
  const dir = resolve(root, 'profiles');
  for (const name of readdirSync(dir).sort()) {
    const lock = resolve(dir, name, 'rfcspec.lock');
    if (!existsSync(lock)) continue;
    const rfcs = new Set([...readFileSync(lock, 'utf8').matchAll(/^- rfc: ([0-9]+)$/gm)].map((m) => Number(m[1])));
    for (const rfc of rfcs) out.set(rfc, [...(out.get(rfc) ?? []), name]);
  }
  return out;
}

describe('adversary: the index against its sources, every field', () => {
  it('holds each summary row field for field, with the profiles of the locks', () => {
    const locks = lockedBy();
    expect(rows.length).toBe(summary.length);
    summary.forEach((row, i) => {
      const entry = rows[i]!;
      expect(entry.rfc, `row ${i}`).toBe(row.rfc);
      expect(entry.title, `RFC ${row.rfc}`).toBe(row.title);
      expect(entry.status, `RFC ${row.rfc}`).toBe(row.status);
      expect(entry.stream, `RFC ${row.rfc}`).toBe(row.stream);
      expect(entry.format, `RFC ${row.rfc}`).toBe(row.format ?? 'text');
      expect(entry.area, `RFC ${row.rfc}`).toBe(row.area);
      expect(entry.date, `RFC ${row.rfc}`).toBe(row.date);
      const counts = Object.fromEntries(entry.counts.map((c) => [c.level, c.count]));
      expect(counts, `RFC ${row.rfc}`).toEqual(row.counts);
      expect(entry.profiles, `RFC ${row.rfc}`).toEqual(locks.get(row.rfc) ?? []);
    });
    expect([...locks.keys()].sort((a, b) => a - b)).toEqual([868, 1035, 5905, 6455]);
  });

  it('lists for every status and every level exactly the summary rows a jq filter keeps', () => {
    const statuses = [...new Set(summary.map((r) => r.status))];
    const levels = [...new Set(summary.flatMap((r) => Object.keys(r.counts)))];
    for (const status of statuses) {
      for (const level of [...levels, 'none']) {
        const expected = summary
          .filter((r) => r.status === status)
          .filter((r) =>
            level === 'none' ? Object.values(r.counts).every((n) => n === 0) : (r.counts[level] ?? 0) > 0,
          )
          .map((r) => r.rfc);
        const found = filterRows(rows, { ...EMPTY, status, level }).map((r) => r.rfc);
        expect(found, `${status} / ${level}`).toEqual(expected);
      }
    }
  });
});

describe('adversary: the acceptance URL on the committed index', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    window.history.replaceState(null, '', '/');
  });

  it('prints the jq count above the list and links each row to ?rfc=<n>', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?status=INTERNET%20STANDARD&level=MUST');
    vi.stubGlobal('fetch', async () => new Response(JSON.stringify(generated), { status: 200 }));
    const wrapper = mount(App);
    await flushPromises();
    const expected = summary.filter((r) => r.status === 'INTERNET STANDARD' && (r.counts.MUST ?? 0) > 0);
    expect(wrapper.find('[data-test="count"]').text()).toBe(`${expected.length} of ${summary.length} RFCs`);
    const links = wrapper.findAll('[data-test="results"] tbody tr a.rfc').map((a) => a.attributes('href'));
    expect(links).toEqual(expected.map((r) => `?rfc=${r.rfc}`));
  });
});

describe('adversary: the controls show the filter in effect', () => {
  const index = fixture as ExplorerIndex;
  const cases: [keyof Filter, string][] = [
    ['status', 'internet standard'],
    ['level', 'must'],
    ['format', 'XML'],
    ['stream', 'ietf'],
  ];
  for (const [key, value] of cases) {
    it(`a link with ${key}=${value} lists what the visible controls select`, () => {
      const wrapper = mount(ListView, { props: { index, initial: { ...EMPTY, [key]: value } } });
      const shown = (wrapper.find(`select[name="${key}"]`).element as HTMLSelectElement).value;
      const count = wrapper.find('[data-test="count"]').text();
      const visible = filterRows(index.rfcs, { ...EMPTY, [key]: shown }).length;
      expect(count).toBe(`${visible} of ${index.rfcs.length} RFCs`);
    });
  }
});

describe('adversary: number filter boundaries', () => {
  it('reads every number ?rfc= reads, leading zeros included', () => {
    for (const text of ['6455', '06455', '006455', '0000006455']) {
      const rfc = requestedRfc(`?rfc=${text}`);
      expect(rfc, text).toBe(6455);
      expect(numberRange(text), text).toEqual({ from: rfc, to: rfc });
    }
  });
});

describe('adversary: query-string round trip, a fixed-seed property', () => {
  it('readFilter(writeFilter(f)) == f for random filters', () => {
    let seed = 20261105;
    const next = (): number => {
      seed = (seed * 1103515245 + 12345) % 2147483648;
      return seed;
    };
    const alphabet = ['a', 'Z', ' ', '+', '%', '&', '=', '#', '?', '/', '\n', '\t', 'ü', 'ß', '€', '😀', '%20', '+', "'", '"', '<', '>'];
    const text = (): string =>
      Array.from({ length: next() % 8 }, () => alphabet[next() % alphabet.length]!).join('');
    for (let i = 0; i < 2000; i++) {
      const f: Filter = {
        number: text(),
        title: text(),
        status: text(),
        stream: text(),
        area: text(),
        format: text(),
        level: text(),
      };
      expect(readFilter(writeFilter(f)), JSON.stringify(f)).toEqual(f);
    }
  });
});
