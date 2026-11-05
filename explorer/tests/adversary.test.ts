import { describe, expect, it } from 'vitest';
import { requestedRfc, shardFile } from '../src/lib/shard.ts';

describe('requestedRfc, adversarial', () => {
  it('reads leading zeros as the number they name', () => {
    expect(requestedRfc('?rfc=09220')).toBe(9220);
    expect(shardFile(requestedRfc('?rfc=0099')!)).toBe('00000.json');
  });

  it('refuses negatives, signs, spaces and exponents', () => {
    expect(requestedRfc('?rfc=-1')).toBeNull();
    expect(requestedRfc('?rfc=%2B9220')).toBeNull();
    expect(requestedRfc('?rfc=1e3')).toBeNull();
    expect(requestedRfc('?rfc=0x10')).toBeNull();
  });

  it('never answers with another number than the one ?rfc= names', () => {
    const asked = '9007199254740993';
    const rfc = requestedRfc(`?rfc=${asked}`);
    expect(rfc === null || String(rfc) === asked, `asked ${asked}, got ${rfc}`).toBe(true);
    const huge = '9'.repeat(400);
    const big = requestedRfc(`?rfc=${huge}`);
    expect(big === null || String(big) === huge, `asked 400 nines, got ${big}`).toBe(true);
  });
});
