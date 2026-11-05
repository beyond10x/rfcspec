import type { ExplorerRfc, ExplorerShard, LevelCount } from '../generated/types.ts';

const RFC_EDITOR = 'https://www.rfc-editor.org/rfc/';

/** The shard file holding an RFC: its hundred, zero-padded to five digits. */
export function shardFile(rfc: number): string {
  return `${String(Math.floor(rfc / 100) * 100).padStart(5, '0')}.json`;
}

/** Where the shard of an RFC is served, relative to the explorer page. */
export function shardUrl(rfc: number, page: string): URL {
  return new URL(`../data/rfcs/${shardFile(rfc)}`, page);
}

/** The most digits an RFC number has; a longer one is in no shard. */
const MAX_DIGITS = 5;

/**
 * The RFC number `?rfc=` names, or null when it names none: only decimal digits, leading zeros
 * allowed, at most five digits after them, so the number read is always the one asked for.
 */
export function requestedRfc(search: string): number | null {
  const value = new URLSearchParams(search).get('rfc');
  if (value === null || !/^[0-9]+$/.test(value)) {
    return null;
  }
  const digits = value.replace(/^0+/, '');
  if (digits.length === 0 || digits.length > MAX_DIGITS) {
    return null;
  }
  return Number(digits);
}

export function findRfc(shard: ExplorerShard, rfc: number): ExplorerRfc | null {
  return shard.rfcs.find((entry) => entry.rfc === rfc) ?? null;
}

/** Whether a number the shard names has a row in the summary, and so its own view. */
export function isIndexed(shard: ExplorerShard, rfc: number): boolean {
  return !shard.not_in_index.includes(rfc);
}

/** The RFC Editor rendering of an RFC, at a section where the number names one. */
export function sectionHref(rfc: number, section: string | undefined): string {
  if (section === undefined) {
    return `${RFC_EDITOR}rfc${rfc}`;
  }
  const kind = /^[A-Z]/.test(section) ? 'appendix' : 'section';
  return `${RFC_EDITOR}rfc${rfc}.html#${kind}-${section}`;
}

/** Counts by level as a page reads them: `3 MUST, 1 MAY`, or `none`. */
export function counted(counts: LevelCount[]): string {
  return counts.length === 0 ? 'none' : counts.map((c) => `${c.count} ${c.level}`).join(', ');
}
