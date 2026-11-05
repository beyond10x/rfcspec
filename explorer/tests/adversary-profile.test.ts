/// <reference types="node" />
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import App from '../src/App.vue';
import ProfileLayer from '../src/components/ProfileLayer.vue';
import type { ExplorerIndex } from '../src/generated/types.ts';
import type { RfcspecProfileExplorerProfile as Profile } from '../src/generated-profile/types.ts';
import shard6400 from './fixtures/06400.json';
import index from './fixtures/rfc-index.json';
import fixture from './fixtures/profiles/websocket.json';
import dns from '../../website/static/data/profiles/dns.json';
import ntp from '../../website/static/data/profiles/ntp.json';
import time from '../../website/static/data/profiles/time.json';
import websocket from '../../website/static/data/profiles/websocket.json';

/** The first sentence of the page's `Reviewed:` and `Unreviewed:` lines. */
function pageLines(name: string): string[] {
  const page = readFileSync(resolve(__dirname, `../../website/docs/catalog/${name}.md`), 'utf8');
  return page
    .split('\n')
    .filter((line) => line.startsWith('Reviewed: ') || line.startsWith('Unreviewed: '))
    .map((line) => line.replace(/ `rfcspec extract` lists them.*$/, ''));
}

describe('the reviewed layer counts every committed profile as its page does', () => {
  for (const [name, profile] of [
    ['dns', dns],
    ['ntp', ntp],
    ['time', time],
    ['websocket', websocket],
  ] as const) {
    it(name, () => {
      const wrapper = mount(ProfileLayer, { props: { profile: profile as Profile } });
      const counts = wrapper.find('[data-test="counts"]').text();
      const lines = pageLines(name);
      expect(lines).toHaveLength(2);
      for (const line of lines) {
        expect(counts).toContain(line);
      }
    });
  }
});

describe('App with several profile marks', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    window.history.replaceState(null, '', '/');
  });

  function serve(files: Record<string, unknown>) {
    vi.stubGlobal('fetch', async (url: URL | string) => {
      const path = new URL(String(url)).pathname;
      return path in files
        ? new Response(JSON.stringify(files[path]), { status: 200 })
        : new Response('', { status: 404 });
    });
  }

  /** The fixture index with RFC 6455 marked by `marks`. */
  function marked(marks: string[]): ExplorerIndex {
    const copy = structuredClone(index) as ExplorerIndex;
    copy.rfcs.find((row) => row.rfc === 6455)!.profiles = marks;
    return copy;
  }

  it('shows the reviewed layer of a mark whose file loads when an earlier mark has none', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=6455');
    serve({
      '/rfcspec/data/rfcs/06400.json': shard6400,
      '/rfcspec/data/rfc-index.json': marked(['draft', 'websocket']),
      '/rfcspec/data/profiles/websocket.json': fixture,
    });
    const wrapper = mount(App);
    await flushPromises();
    expect(wrapper.findAll('[data-test="reviewed-layer"]')).toHaveLength(1);
  });

  it('reports no failure for a mark of a profile without an overlay, which has no file by design', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=6455');
    serve({
      '/rfcspec/data/rfcs/06400.json': shard6400,
      '/rfcspec/data/rfc-index.json': marked(['draft']),
    });
    const wrapper = mount(App);
    await flushPromises();
    expect(wrapper.text()).toContain('The WebSocket Protocol');
    expect(wrapper.text()).not.toContain('could not be loaded');
  });

  it('still shows the reviewed layer when the RFC has no shard', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=6455');
    serve({
      '/rfcspec/data/rfc-index.json': index,
      '/rfcspec/data/profiles/websocket.json': fixture,
    });
    const wrapper = mount(App);
    await flushPromises();
    expect(wrapper.findAll('[data-test="reviewed-layer"]')).toHaveLength(1);
  });

  it('renders a quote holding markup as text', () => {
    const profile = structuredClone(fixture) as Profile;
    profile.requirements[0]!.quote = '<img src=x onerror=alert(1)>';
    const wrapper = mount(ProfileLayer, { props: { profile } });
    expect(wrapper.find('img').exists()).toBe(false);
    expect(wrapper.text()).toContain('<img src=x onerror=alert(1)>');
  });
});
