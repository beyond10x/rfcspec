import { describe, expect, it } from 'vitest';
import { findRfc, isIndexed, requestedRfc, sectionHref, shardFile } from '../src/lib/shard.ts';
import type { ExplorerShard } from '../src/generated/types.ts';
import h3 from './fixtures/09200.json';

const shard = h3 as ExplorerShard;

describe('shardFile', () => {
  it('names the hundred an RFC number lies in, zero-padded to five digits', () => {
    expect(shardFile(9220)).toBe('09200.json');
    expect(shardFile(1)).toBe('00000.json');
    expect(shardFile(99)).toBe('00000.json');
    expect(shardFile(100)).toBe('00100.json');
    expect(shardFile(10001)).toBe('10000.json');
  });
});

describe('requestedRfc', () => {
  it('reads a positive whole number from ?rfc=', () => {
    expect(requestedRfc('?rfc=9220')).toBe(9220);
    expect(requestedRfc('')).toBeNull();
    expect(requestedRfc('?rfc=')).toBeNull();
    expect(requestedRfc('?rfc=abc')).toBeNull();
    expect(requestedRfc('?rfc=0')).toBeNull();
    expect(requestedRfc('?rfc=12.5')).toBeNull();
  });
});

describe('a shard', () => {
  it('finds its RFCs and links every number not listed as absent from the index', () => {
    expect(findRfc(shard, 9220)?.title).toBe('Bootstrapping WebSockets with HTTP/3');
    expect(findRfc(shard, 9221)).toBeNull();
    expect(isIndexed({ ...shard, not_in_index: [8441] }, 8441)).toBe(false);
    expect(isIndexed(shard, 8441)).toBe(true);
  });
});

describe('sectionHref', () => {
  it('links a numbered section, an appendix and an RFC without a section number', () => {
    expect(sectionHref(9220, '3')).toBe('https://www.rfc-editor.org/rfc/rfc9220.html#section-3');
    expect(sectionHref(9114, 'A.3')).toBe('https://www.rfc-editor.org/rfc/rfc9114.html#appendix-A.3');
    expect(sectionHref(9220, undefined)).toBe('https://www.rfc-editor.org/rfc/rfc9220');
  });
});
