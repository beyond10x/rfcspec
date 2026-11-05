import type { Bcp14Level, ExplorerIndexRow, SourceFormat } from '../generated/types.ts';

/**
 * The filters of the front page, each as the text of its control; an empty string filters
 * nothing. `level` is a BCP 14 level, or `none` for the rows without any requirement.
 */
export type Filter = {
  number: string;
  title: string;
  status: string;
  stream: string;
  area: string;
  format: string;
  level: string;
};

/** The filter that keeps every row, its keys in the order the query string writes them. */
export const EMPTY: Filter = {
  number: '',
  title: '',
  status: '',
  stream: '',
  area: '',
  format: '',
  level: '',
};

/** The `level` value that keeps the rows without any requirement. */
export const NO_REQUIREMENTS = 'none';

const KEYS = Object.keys(EMPTY) as (keyof Filter)[];

/** Where the index is served, relative to the explorer page. */
export function indexUrl(page: string): URL {
  return new URL('../data/rfc-index.json', page);
}

/**
 * The numbers the number filter names: one number or an inclusive range `from-to`, null when it
 * is empty and `invalid` when it is neither.
 */
export function numberRange(text: string): { from: number; to: number } | null | 'invalid' {
  const trimmed = text.trim();
  if (trimmed === '') {
    return null;
  }
  const match = /^0*([0-9]{1,5})(?:\s*-\s*0*([0-9]{1,5}))?$/.exec(trimmed);
  if (match === null) {
    return 'invalid';
  }
  const from = Number(match[1]);
  const to = match[2] === undefined ? from : Number(match[2]);
  return from <= to ? { from, to } : 'invalid';
}

function hasLevel(row: ExplorerIndexRow, level: string): boolean {
  if (level === NO_REQUIREMENTS) {
    return row.counts.every((count) => count.count === 0);
  }
  return row.counts.some((count) => count.level === level && count.count > 0);
}

/** A predicate over rows for a filter: every filter set must keep the row. */
export function matcher(filter: Filter): (row: ExplorerIndexRow) => boolean {
  const range = numberRange(filter.number);
  const words = filter.title.toLowerCase().split(/\s+/).filter((word) => word !== '');
  return (row) =>
    range !== 'invalid' &&
    (range === null || (row.rfc >= range.from && row.rfc <= range.to)) &&
    words.every((word) => row.title.toLowerCase().includes(word)) &&
    (filter.status === '' || row.status === filter.status) &&
    (filter.stream === '' || row.stream === filter.stream) &&
    (filter.area === '' || row.area === filter.area) &&
    (filter.format === '' || row.format === filter.format) &&
    (filter.level === '' || hasLevel(row, filter.level));
}

/** The rows a filter keeps, in index order. */
export function filterRows(rows: ExplorerIndexRow[], filter: Filter): ExplorerIndexRow[] {
  return rows.filter(matcher(filter));
}

/** Each value of a field the rows hold, once, sorted. */
export function choices(rows: ExplorerIndexRow[], field: 'status' | 'stream' | 'area'): string[] {
  const values = new Set<string>();
  for (const row of rows) {
    const value = row[field];
    if (value !== undefined) {
      values.add(value);
    }
  }
  return [...values].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
}

/** The BCP 14 levels in the order RFC 2119 lists them. */
export const LEVELS: Bcp14Level[] = [
  'MUST',
  'MUST NOT',
  'REQUIRED',
  'SHALL',
  'SHALL NOT',
  'SHOULD',
  'SHOULD NOT',
  'RECOMMENDED',
  'NOT RECOMMENDED',
  'MAY',
  'OPTIONAL',
];

/** The source formats the index holds. */
export const FORMATS: SourceFormat[] = ['xml', 'text'];

/** The filters a select sets, each from a fixed list of values. */
export type Chosen = 'status' | 'stream' | 'area' | 'format' | 'level';

/** Every filter a select sets, in the order the page shows them. */
export const CHOSEN: Chosen[] = ['status', 'stream', 'area', 'format', 'level'];

/** The values each select offers for an index's rows. */
export function offered(rows: ExplorerIndexRow[]): Record<Chosen, string[]> {
  return {
    status: choices(rows, 'status'),
    stream: choices(rows, 'stream'),
    area: choices(rows, 'area'),
    format: [...FORMATS],
    level: [...LEVELS, NO_REQUIREMENTS],
  };
}

/**
 * A filter with each select-set value replaced by the offered value it equals ignoring case, so
 * the controls show the filter in effect, and the filters whose value no select offers.
 */
export function canonical(
  filter: Filter,
  offers: Record<Chosen, string[]>,
): { filter: Filter; unknown: Chosen[] } {
  const out = { ...filter };
  const unknown: Chosen[] = [];
  for (const key of CHOSEN) {
    if (out[key] === '') {
      continue;
    }
    const lower = out[key].toLowerCase();
    const found = offers[key].find((value) => value.toLowerCase() === lower);
    if (found === undefined) {
      unknown.push(key);
    } else {
      out[key] = found;
    }
  }
  return { filter: out, unknown };
}

/** The filter a query string names; other parameters are ignored. */
export function readFilter(search: string): Filter {
  const params = new URLSearchParams(search);
  const filter = { ...EMPTY };
  for (const key of KEYS) {
    filter[key] = params.get(key) ?? '';
  }
  return filter;
}

/** The query string of a filter: the filters set, in a fixed order, `''` when none is. */
export function writeFilter(filter: Filter): string {
  const parts = KEYS.filter((key) => filter[key] !== '').map(
    (key) => `${key}=${encodeURIComponent(filter[key])}`,
  );
  return parts.length === 0 ? '' : `?${parts.join('&')}`;
}
