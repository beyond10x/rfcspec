import { describe, expect, it } from 'vitest';
import type { ExplorerIndex } from '../src/generated/types.ts';
import {
  CHOSEN,
  EMPTY,
  type Filter,
  canonical,
  choices,
  filterRows,
  indexUrl,
  numberRange,
  offered,
  readFilter,
  writeFilter,
} from '../src/lib/search.ts';
import fixture from './fixtures/rfc-index.json';

const rows = (fixture as ExplorerIndex).rfcs;

function numbers(filter: Partial<Filter>): number[] {
  return filterRows(rows, { ...EMPTY, ...filter }).map((row) => row.rfc);
}

describe('the number filter', () => {
  it('reads one number or an inclusive range, and refuses anything else', () => {
    expect(numberRange('')).toBeNull();
    expect(numberRange(' 6455 ')).toEqual({ from: 6455, to: 6455 });
    expect(numberRange('5000-7999')).toEqual({ from: 5000, to: 7999 });
    expect(numberRange('5000 - 7999')).toEqual({ from: 5000, to: 7999 });
    expect(numberRange('7999-5000')).toBe('invalid');
    expect(numberRange('abc')).toBe('invalid');
    expect(numberRange('1e3')).toBe('invalid');
    expect(numberRange('-5')).toBe('invalid');
    expect(numberRange('06455')).toEqual({ from: 6455, to: 6455 });
    expect(numberRange('0000006455')).toEqual({ from: 6455, to: 6455 });
    expect(numberRange('06000-0006999')).toEqual({ from: 6000, to: 6999 });
    expect(numberRange('123456')).toBe('invalid');
    expect(numberRange('0123456')).toBe('invalid');
  });

  it('keeps the number asked for, the rows of a range, and none for an unreadable number', () => {
    expect(numbers({ number: '6455' })).toEqual([6455]);
    expect(numbers({ number: '6456' })).toEqual([]);
    expect(numbers({ number: '5000-7999' })).toEqual([5905, 6455, 7540]);
    expect(numbers({ number: '868-1035' })).toEqual([868, 1035]);
    expect(numbers({ number: 'x' })).toEqual([]);
    expect(numbers({})).toHaveLength(rows.length);
  });
});

describe('the title filter', () => {
  it('keeps the rows whose title holds every word, in any case and any order', () => {
    expect(numbers({ title: 'websocket' })).toEqual([6455, 8441, 9220]);
    expect(numbers({ title: 'HTTP/3 bootstrapping' })).toEqual([9220]);
    expect(numbers({ title: '  WEBSOCKETS   http/2 ' })).toEqual([8441]);
    expect(numbers({ title: 'websocket gopher' })).toEqual([]);
  });
});

describe('the status, stream, area and format filters', () => {
  it('keep the rows equal to the value chosen', () => {
    expect(numbers({ status: 'INTERNET STANDARD' })).toEqual([868, 1035, 9915]);
    expect(numbers({ status: 'internet standard' })).toEqual([]);
    expect(numbers({ stream: 'IRTF' })).toEqual([8763]);
    expect(numbers({ stream: 'Legacy' })).toEqual([1, 868, 1035]);
    expect(numbers({ area: 'wit' })).toEqual([7540, 8441, 9220]);
    expect(numbers({ format: 'xml' })).toEqual([8763, 9220, 9915, 9962]);
    expect(numbers({ format: 'text' })).toHaveLength(8);
  });

  it('offer each value the index holds, once, sorted', () => {
    expect(choices(rows, 'stream')).toEqual(['IETF', 'INDEPENDENT', 'IRTF', 'Legacy']);
    expect(choices(rows, 'area')).toEqual(['art', 'int', 'wit']);
    expect(choices(rows, 'status')).toEqual([
      'BEST CURRENT PRACTICE',
      'INFORMATIONAL',
      'INTERNET STANDARD',
      'PROPOSED STANDARD',
      'UNKNOWN',
    ]);
  });
});

describe('the requirement level filter', () => {
  it('keeps the rows with at least one requirement of the level chosen', () => {
    expect(numbers({ level: 'MUST' })).toEqual([2119, 5905, 6455, 7540, 8441, 9915, 9962]);
    expect(numbers({ level: 'OPTIONAL' })).toEqual([6455]);
    expect(numbers({ level: 'RECOMMENDED' })).toEqual([9915]);
    expect(numbers({ level: 'SHALL' })).toEqual([]);
  });

  it('keeps the rows without any requirement for "none"', () => {
    expect(numbers({ level: 'none' })).toEqual([1, 868, 1035, 8763]);
  });

  it('treats a level counted zero as absent', () => {
    const zero = [{ ...rows[0]!, counts: [{ level: 'MUST' as const, count: 0 }] }];
    expect(filterRows(zero, { ...EMPTY, level: 'MUST' })).toEqual([]);
    expect(filterRows(zero, { ...EMPTY, level: 'none' })).toEqual(zero);
  });
});

describe('canonical', () => {
  const offers = offered(rows);

  it('replaces every select-set value by the offered value it equals ignoring case', () => {
    const mangled: Filter = {
      ...EMPTY,
      status: 'internet STANDARD',
      stream: 'ietf',
      area: 'WIT',
      format: 'XML',
      level: 'must not',
    };
    expect(canonical(mangled, offers)).toEqual({
      filter: { ...EMPTY, status: 'INTERNET STANDARD', stream: 'IETF', area: 'wit', format: 'xml', level: 'MUST NOT' },
      unknown: [],
    });
    expect(canonical({ ...EMPTY, level: 'NONE' }, offers).filter.level).toBe('none');
  });

  it('names, for every select, a value it does not offer, and keeps it', () => {
    for (const key of CHOSEN) {
      const read = canonical({ ...EMPTY, [key]: 'gopher' }, offers);
      expect(read.unknown, key).toEqual([key]);
      expect(read.filter[key], key).toBe('gopher');
    }
    expect(canonical({ ...EMPTY, title: 'gopher', number: '1' }, offers).unknown).toEqual([]);
  });
});

describe('combined filters', () => {
  it('keep only the rows every filter keeps', () => {
    expect(numbers({ status: 'INTERNET STANDARD', level: 'MUST' })).toEqual([9915]);
    expect(numbers({ status: 'INTERNET STANDARD', level: 'none' })).toEqual([868, 1035]);
    expect(numbers({ area: 'wit', format: 'xml', title: 'http' })).toEqual([9220]);
    expect(numbers({ number: '6000-9999', stream: 'IETF', level: 'MAY', area: 'wit' })).toEqual([
      7540, 8441, 9220,
    ]);
    expect(numbers({ stream: 'IRTF', level: 'MUST' })).toEqual([]);
  });
});

describe('the query string', () => {
  it('reads every filter and ignores other parameters', () => {
    expect(readFilter('?status=INTERNET%20STANDARD&level=MUST&rfc=1&x=y')).toEqual({
      ...EMPTY,
      status: 'INTERNET STANDARD',
      level: 'MUST',
    });
    expect(readFilter('?title=web+socket&number=6000-6999')).toEqual({
      ...EMPTY,
      title: 'web socket',
      number: '6000-6999',
    });
    expect(readFilter('')).toEqual(EMPTY);
  });

  it('writes only the filters set, as a link the acceptance URL reads', () => {
    expect(writeFilter(EMPTY)).toBe('');
    expect(writeFilter({ ...EMPTY, status: 'INTERNET STANDARD', level: 'MUST' })).toBe(
      '?status=INTERNET%20STANDARD&level=MUST',
    );
  });

  it('round-trips every filter, special characters included', () => {
    const full: Filter = {
      number: '6000-6999',
      title: 'HTTP/2 & "web" #1 ?x=y',
      status: 'PROPOSED STANDARD',
      stream: 'IETF',
      area: 'wit',
      format: 'xml',
      level: 'MUST NOT',
    };
    expect(readFilter(writeFilter(full))).toEqual(full);
    for (const key of Object.keys(full) as (keyof Filter)[]) {
      const one = { ...EMPTY, [key]: full[key] };
      expect(readFilter(writeFilter(one)), key).toEqual(one);
    }
    const link = '?status=INTERNET%20STANDARD&level=MUST';
    expect(writeFilter(readFilter(link))).toBe(link);
  });
});

describe('indexUrl', () => {
  it('names the index beside the shards, relative to the explorer page', () => {
    expect(String(indexUrl('https://beyond10x.github.io/rfcspec/explorer/?level=MUST'))).toBe(
      'https://beyond10x.github.io/rfcspec/data/rfc-index.json',
    );
  });
});
