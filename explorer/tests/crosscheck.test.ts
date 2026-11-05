/// <reference types="node" />
import { describe, expect, it } from 'vitest';
import type { ExplorerIndex } from '../src/generated/types.ts';
import { EMPTY, filterRows, readFilter } from '../src/lib/search.ts';
import generated from '../../website/static/data/rfc-index.json';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

/** A bulk summary row as `bulk/rfc-summary.jsonl` holds it: counts as a map by level. */
type SummaryRow = { rfc: number; status: string; counts: Record<string, number> };

const summaryText = readFileSync(resolve(__dirname, '../../bulk/rfc-summary.jsonl'), 'utf8');

const summary: SummaryRow[] = summaryText
  .split('\n')
  .slice(1)
  .filter((line) => line !== '')
  .map((line) => JSON.parse(line) as SummaryRow);

const rows = (generated as ExplorerIndex).rfcs;

describe('the committed index against the bulk summary', () => {
  it('lists for ?status=INTERNET%20STANDARD&level=MUST exactly the summary rows with that status and a MUST count above zero', () => {
    const expected = summary
      .filter((row) => row.status === 'INTERNET STANDARD' && (row.counts.MUST ?? 0) > 0)
      .map((row) => row.rfc);
    const found = filterRows(rows, readFilter('?status=INTERNET%20STANDARD&level=MUST')).map((row) => row.rfc);
    expect(expected.length).toBeGreaterThan(0);
    expect(found).toEqual(expected);
  });

  it('lists for level=none exactly the summary rows without a requirement, and every row unfiltered', () => {
    const expected = summary
      .filter((row) => Object.values(row.counts).every((n) => n === 0))
      .map((row) => row.rfc);
    expect(filterRows(rows, readFilter('?level=none')).map((row) => row.rfc)).toEqual(expected);
    expect(filterRows(rows, EMPTY).map((row) => row.rfc)).toEqual(summary.map((row) => row.rfc));
  });
});
