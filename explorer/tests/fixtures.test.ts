import { describe, expect, it } from 'vitest';
import type { ExplorerIndex, ExplorerShard } from '../src/generated/types.ts';
import h3 from './fixtures/09200.json';
import websocket from './fixtures/06400.json';
import generated09200 from '../../website/static/data/rfcs/09200.json';
import generated06400 from '../../website/static/data/rfcs/06400.json';
import index from './fixtures/rfc-index.json';
import generatedIndex from '../../website/static/data/rfc-index.json';
import type { RfcspecProfileExplorerProfile as Profile } from '../src/generated-profile/types.ts';
import websocketProfile from './fixtures/profiles/websocket.json';
import generatedWebsocketProfile from '../../website/static/data/profiles/websocket.json';

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

  it('hold the generated index narrowed to twelve RFCs, so a shape change fails here', () => {
    const fixture = index as ExplorerIndex;
    const numbers = fixture.rfcs.map((row) => row.rfc);
    expect(numbers).toEqual([1, 868, 1035, 2119, 5905, 6455, 7540, 8441, 8763, 9220, 9915, 9962]);
    const generated = generatedIndex as ExplorerIndex;
    expect(fixture).toEqual({ ...generated, rfcs: generated.rfcs.filter((row) => numbers.includes(row.rfc)) });
  });

  it('hold the generated websocket profile narrowed to three requirements and two decisions, so a shape change fails here', () => {
    const fixture = websocketProfile as Profile;
    const requirements = ['rfc6455/R5.1-1', 'rfc6455/R5.2-1', 'rfc6455/R5.5-1'];
    const decisions = ['rfc6455/R5.1-1', 'rfc6455/R5.1-4'];
    const generated = generatedWebsocketProfile as Profile;
    expect(fixture).toEqual({
      ...generated,
      requirements: generated.requirements.filter((r) => requirements.includes(r.status.requirement_id)),
      decisions: generated.decisions.filter((d) => decisions.includes(d.row.key.item)),
    });
    expect(fixture.requirements).toHaveLength(3);
    expect(fixture.decisions.map((d) => d.row.kind)).toEqual(['accept', 'reject']);
  });
});
