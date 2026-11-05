import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import App from '../src/App.vue';
import RfcView from '../src/components/RfcView.vue';
import type { ExplorerShard } from '../src/generated/types.ts';
import h3 from './fixtures/09200.json';
import websocket from './fixtures/06400.json';

function hrefs(wrapper: { findAll(selector: string): { attributes(name: string): string | undefined }[] }): string[] {
  return wrapper.findAll('a').map((a) => a.attributes('href') ?? '');
}

describe('RfcView', () => {
  it('shows RFC 9220 with its metadata, its sections and counts and its References linked', () => {
    const wrapper = mount(RfcView, { props: { shard: h3 as ExplorerShard, rfc: 9220 } });
    const text = wrapper.text();
    expect(text).toContain('RFC 9220');
    expect(text).toContain('Bootstrapping WebSockets with HTTP/3');
    expect(text).toContain('PROPOSED STANDARD');
    expect(text).toContain('2022-06');
    expect(text).toContain('XML');
    expect(text).toContain('8a60d0b566bfb252ff18a3dc3df9fe94e90b201b807002a47d45386616b09aae');

    const sections = wrapper.find('[data-test="sections"]');
    expect(sections.text()).toContain('3');
    expect(sections.text()).toContain('WebSockets Upgrade over HTTP/3');
    expect(sections.text()).toContain('1 SHOULD');
    expect(sections.text()).toContain('1 MAY');
    expect(hrefs(sections)).toContain('https://www.rfc-editor.org/rfc/rfc9220.html#section-3');

    const references = wrapper.find('[data-test="references"]');
    const rfc8441 = references.findAll('tr').find((row) => row.find('a[href="?rfc=8441"]').exists());
    expect(rfc8441, references.html()).toBeDefined();
    expect(rfc8441!.text()).toContain('normative');
    expect(rfc8441!.find('a[href="?rfc=8441"]').text()).toBe('RFC 8441');

    const citing = references
      .findAll('tbody tr')
      .map((row) => row.findAll('td')[3]!.text());
    expect(citing).toContain('3 WebSockets Upgrade over HTTP/3');
    expect(citing).not.toContain('websockets-upgrade-over-http3');
    expect(citing).toContain('introduction');
  });

  it('shows RFC 6455 updated by 7936, 8307 and 8441 as links', () => {
    const wrapper = mount(RfcView, { props: { shard: websocket as ExplorerShard, rfc: 6455 } });
    const updated = wrapper.find('[data-test="updated_by"]');
    expect(hrefs(updated)).toEqual(['?rfc=7936', '?rfc=8307', '?rfc=8441']);
    expect(wrapper.find('[data-test="references"]').exists()).toBe(false);
  });

  it('shows a number without a row as plain text', () => {
    const shard = { ...(websocket as ExplorerShard), not_in_index: [8307] };
    const wrapper = mount(RfcView, { props: { shard, rfc: 6455 } });
    const updated = wrapper.find('[data-test="updated_by"]');
    expect(hrefs(updated)).toEqual(['?rfc=7936', '?rfc=8441']);
    expect(updated.text()).toContain('8307');
  });

  it('reads "not in the index" for a number its shard lacks or for no shard', () => {
    expect(mount(RfcView, { props: { shard: h3 as ExplorerShard, rfc: 9221 } }).text()).toContain(
      'RFC 9221 is not in the index',
    );
    expect(mount(RfcView, { props: { shard: null, rfc: 99999 } }).text()).toContain(
      'RFC 99999 is not in the index',
    );
  });
});

describe('App', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    window.history.replaceState(null, '', '/');
  });

  it('fetches the shard of the hundred ?rfc= names and shows the RFC', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=9220');
    const fetched: string[] = [];
    vi.stubGlobal('fetch', async (url: URL | string) => {
      fetched.push(String(url));
      return new Response(JSON.stringify(h3), { status: 200 });
    });
    const wrapper = mount(App);
    await flushPromises();
    expect(fetched).toEqual(['http://localhost:3000/rfcspec/data/rfcs/09200.json']);
    expect(wrapper.text()).toContain('Bootstrapping WebSockets with HTTP/3');
  });

  it('reads "not in the index" for a number longer than any RFC number, fetching nothing', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=9007199254740993');
    const fetched: string[] = [];
    vi.stubGlobal('fetch', async (url: URL | string) => {
      fetched.push(String(url));
      return new Response('', { status: 404 });
    });
    const wrapper = mount(App);
    await flushPromises();
    expect(fetched).toEqual([]);
    expect(wrapper.text()).toContain('RFC 9007199254740993 is not in the index');
  });

  it('reads "not in the index" when the hundred has no shard', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=99999');
    vi.stubGlobal('fetch', async () => new Response('', { status: 404 }));
    const wrapper = mount(App);
    await flushPromises();
    expect(wrapper.text()).toContain('RFC 99999 is not in the index');
  });
});
