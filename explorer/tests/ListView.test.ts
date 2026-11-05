import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import App from '../src/App.vue';
import ListView from '../src/components/ListView.vue';
import type { ExplorerIndex } from '../src/generated/types.ts';
import { EMPTY } from '../src/lib/search.ts';
import fixture from './fixtures/rfc-index.json';

const index = fixture as ExplorerIndex;

function listed(wrapper: ReturnType<typeof mount>): string[] {
  return wrapper.findAll('[data-test="results"] tbody tr a.rfc').map((a) => a.attributes('href') ?? '');
}

describe('ListView', () => {
  it('lists every row without a filter, each linked to its RFC view, with the count above', () => {
    const wrapper = mount(ListView, { props: { index, initial: EMPTY } });
    expect(wrapper.find('[data-test="count"]').text()).toBe(`${index.rfcs.length} of ${index.rfcs.length} RFCs`);
    expect(listed(wrapper)).toEqual(index.rfcs.map((row) => `?rfc=${row.rfc}`));
    const websocket = wrapper.findAll('[data-test="results"] tbody tr').find((tr) => tr.text().includes('The WebSocket Protocol'));
    expect(websocket!.text()).toContain('websocket');
    expect(websocket!.text()).toContain('108 MUST');
  });

  it('applies the filter it starts with and reports each change as a filter', async () => {
    const wrapper = mount(ListView, {
      props: { index, initial: { ...EMPTY, status: 'INTERNET STANDARD', level: 'MUST' } },
    });
    expect(wrapper.find('[data-test="count"]').text()).toBe(`1 of ${index.rfcs.length} RFCs`);
    expect(listed(wrapper)).toEqual(['?rfc=9915']);

    await wrapper.find('select[name="level"]').setValue('none');
    expect(listed(wrapper)).toEqual(['?rfc=868', '?rfc=1035']);
    await wrapper.find('input[name="title"]').setValue('TIME');
    expect(listed(wrapper)).toEqual(['?rfc=868']);
    const changes = wrapper.emitted('change') as [typeof EMPTY][];
    expect(changes.at(-1)![0]).toEqual({ ...EMPTY, status: 'INTERNET STANDARD', level: 'none', title: 'TIME' });
  });

  it('offers the statuses, streams and areas of the index, and says when a number is unreadable', async () => {
    const wrapper = mount(ListView, { props: { index, initial: EMPTY } });
    const options = (name: string): string[] =>
      wrapper.findAll(`select[name="${name}"] option`).map((o) => o.attributes('value') ?? '');
    expect(options('stream')).toEqual(['', 'IETF', 'INDEPENDENT', 'IRTF', 'Legacy']);
    expect(options('area')).toEqual(['', 'art', 'int', 'wit']);
    expect(options('format')).toEqual(['', 'xml', 'text']);
    expect(options('level')).toContain('none');
    expect(options('level')).toContain('MUST NOT');
    await wrapper.find('input[name="number"]').setValue('12x');
    expect(wrapper.find('[data-test="count"]').text()).toBe(`0 of ${index.rfcs.length} RFCs`);
    expect(wrapper.text()).toContain('a number such as 6455 or a range such as 6000-6999');
  });
});

describe('ListView with a query value in another case or unknown', () => {
  const selects = ['status', 'stream', 'area', 'format', 'level'] as const;

  it('shows the canonical value in each select, filters by it and reports it once', () => {
    const wrapper = mount(ListView, {
      props: { index, initial: { ...EMPTY, status: 'internet standard', level: 'must' } },
    });
    const shown = (name: string): string => (wrapper.find(`select[name="${name}"]`).element as HTMLSelectElement).value;
    expect(shown('status')).toBe('INTERNET STANDARD');
    expect(shown('level')).toBe('MUST');
    expect(listed(wrapper)).toEqual(['?rfc=9915']);
    expect(wrapper.emitted('change')![0]![0]).toEqual({ ...EMPTY, status: 'INTERNET STANDARD', level: 'MUST' });
  });

  it('shows a value no select offers as its own option with a notice, for every select', () => {
    for (const key of selects) {
      const wrapper = mount(ListView, { props: { index, initial: { ...EMPTY, [key]: 'gopher' } } });
      expect((wrapper.find(`select[name="${key}"]`).element as HTMLSelectElement).value, key).toBe('gopher');
      expect(wrapper.find(`[data-test="unknown-${key}"]`).text(), key).toContain('"gopher" is not a known value');
      expect(wrapper.find('[data-test="count"]').text(), key).toBe(`0 of ${index.rfcs.length} RFCs`);
      expect(wrapper.emitted('change'), key).toBeUndefined();
    }
  });
});

describe('App front page', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    window.history.replaceState(null, '', '/');
  });

  it('fetches the index and lists the rows the query string filters to', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?status=INTERNET%20STANDARD&level=MUST');
    const fetched: string[] = [];
    vi.stubGlobal('fetch', async (url: URL | string) => {
      fetched.push(String(url));
      return new Response(JSON.stringify(index), { status: 200 });
    });
    const wrapper = mount(App);
    await flushPromises();
    expect(fetched).toEqual(['http://localhost:3000/rfcspec/data/rfc-index.json']);
    expect(wrapper.find('[data-test="count"]').text()).toBe(`1 of ${index.rfcs.length} RFCs`);
    expect(listed(wrapper)).toEqual(['?rfc=9915']);
  });

  it('writes a changed filter into the query string, so the list is a link', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/');
    vi.stubGlobal('fetch', async () => new Response(JSON.stringify(index), { status: 200 }));
    const wrapper = mount(App);
    await flushPromises();
    await wrapper.find('select[name="status"]').setValue('INTERNET STANDARD');
    await wrapper.find('select[name="level"]').setValue('MUST');
    expect(window.location.pathname).toBe('/rfcspec/explorer/');
    expect(window.location.search).toBe('?status=INTERNET%20STANDARD&level=MUST');
    await wrapper.find('select[name="status"]').setValue('');
    await wrapper.find('select[name="level"]').setValue('');
    expect(window.location.search).toBe('');
  });

  it('rewrites a query value in another case to the canonical one', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?status=internet%20standard&level=must');
    vi.stubGlobal('fetch', async () => new Response(JSON.stringify(index), { status: 200 }));
    const wrapper = mount(App);
    await flushPromises();
    expect(listed(wrapper)).toEqual(['?rfc=9915']);
    expect(window.location.search).toBe('?status=INTERNET%20STANDARD&level=MUST');
  });

  it('says so when the index cannot be loaded', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/');
    vi.stubGlobal('fetch', async () => new Response('', { status: 500 }));
    const wrapper = mount(App);
    await flushPromises();
    expect(wrapper.text()).toContain('The RFC index could not be loaded: HTTP 500');
  });

  it('still opens the RFC view for ?rfc=', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=99999');
    const fetched: string[] = [];
    vi.stubGlobal('fetch', async (url: URL | string) => {
      fetched.push(String(url));
      return new Response('', { status: 404 });
    });
    const wrapper = mount(App);
    await flushPromises();
    expect(fetched).toEqual(['http://localhost:3000/rfcspec/data/rfcs/99900.json']);
    expect(wrapper.text()).toContain('RFC 99999 is not in the index');
  });
});
