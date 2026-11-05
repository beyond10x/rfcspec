import { describe, expect, it } from 'vitest';
import type { ExplorerShard } from '../src/generated/types.ts';
import h3 from './fixtures/09200.json';
import websocket from './fixtures/06400.json';
import generated09200 from '../../website/static/data/rfcs/09200.json';
import generated06400 from '../../website/static/data/rfcs/06400.json';

/** The generated shard narrowed to one RFC, as a fixture holds it. */
function narrowed(shard: ExplorerShard, rfc: number): ExplorerShard {
  return { ...shard, rfcs: shard.rfcs.filter((entry) => entry.rfc === rfc) };
}

describe('the fixtures', () => {
  it('are the generated shards narrowed to RFC 9220 and RFC 6455, so a shape change fails here', () => {
    expect(h3).toEqual(narrowed(generated09200 as ExplorerShard, 9220));
    expect(websocket).toEqual(narrowed(generated06400 as ExplorerShard, 6455));
    expect((h3 as ExplorerShard).rfcs).toHaveLength(1);
    expect((websocket as ExplorerShard).rfcs).toHaveLength(1);
  });
});
