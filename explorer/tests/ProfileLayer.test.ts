import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import App from '../src/App.vue';
import ProfileLayer from '../src/components/ProfileLayer.vue';
import RfcView from '../src/components/RfcView.vue';
import type { ExplorerShard } from '../src/generated/types.ts';
import type { RfcspecProfileExplorerProfile as Profile } from '../src/generated-profile/types.ts';
import fixture from './fixtures/profiles/websocket.json';
import h3 from './fixtures/09200.json';
import shard6400 from './fixtures/06400.json';
import index from './fixtures/rfc-index.json';
import websocket from '../../website/static/data/profiles/websocket.json';
import time from '../../website/static/data/profiles/time.json';

const SOURCE = 'https://github.com/beyond10x/rfcspec/blob/main/';

function hrefs(wrapper: { findAll(selector: string): { attributes(name: string): string | undefined }[] }): string[] {
  return wrapper.findAll('a').map((a) => a.attributes('href') ?? '');
}

describe('ProfileLayer on the fixture profile', () => {
  const wrapper = mount(ProfileLayer, { props: { profile: fixture as Profile } });

  it('lists each requirement with its id, level, quote, line span, status and ESS gap', () => {
    const rows = wrapper.findAll('[data-test="requirements"] tbody tr');
    expect(rows).toHaveLength((fixture as Profile).requirements.length);
    const gap = wrapper.find('[data-test="requirements"] tr[data-item="rfc6455/R5.5-1"]');
    expect(gap.text()).toContain('R5.5-1');
    expect(gap.text()).toContain('unsupported');
    expect(gap.text()).toContain('G1');
    expect(gap.text()).toContain('lines');
    const satisfied = wrapper.find('[data-test="requirements"] tr[data-item="rfc6455/R5.2-1"]');
    expect(satisfied.text()).toContain('satisfied');
  });

  it('shows the decision on a decided requirement with its proposal linked and its model id', () => {
    const decided = (fixture as Profile).decisions.find((d) => d.row.kind === 'accept')!;
    const row = wrapper.find(`[data-test="requirements"] tr[data-item="${decided.row.key.item}"]`);
    expect(row.exists(), decided.row.key.item).toBe(true);
    expect(row.text()).toContain('accept');
    expect(row.text()).toContain(decided.model_id!);
    expect(hrefs(row)).toContain(`${SOURCE}${decided.proposal_path!}`);
  });

  it('lists each rejected proposal with its reason, proposal link and model id', () => {
    const rejects = (fixture as Profile).decisions.filter((d) => d.row.kind === 'reject');
    expect(rejects.length).toBeGreaterThan(0);
    const rows = wrapper.findAll('[data-test="rejected"] tbody tr');
    expect(rows).toHaveLength(rejects.length);
    rejects.forEach((reject, i) => {
      expect(rows[i]!.text()).toContain(reject.row.reason!);
      expect(rows[i]!.text()).toContain(reject.model_id!);
      expect(hrefs(rows[i]!)).toContain(`${SOURCE}${reject.proposal_path!}`);
    });
  });

  it('counts the unreviewed requirements by level as the profile page does', () => {
    expect(wrapper.find('[data-test="counts"]').text()).toContain(
      'Unreviewed: 41 requirements of the mechanical catalogue of RFC 6455 have no status, by level 21 MUST, 2 MUST NOT, 4 SHOULD, 13 MAY, 1 OPTIONAL.',
    );
  });

  it('links each machine to its protospec download and its witness on main', () => {
    const rows = wrapper.findAll('[data-test="machines"] tbody tr');
    expect(rows.map((row) => row.findAll('td')[0]!.text())).toEqual(['close', 'ping', 'fragments']);
    for (const [i, name] of ['close', 'ping', 'fragments'].entries()) {
      expect(hrefs(rows[i]!)).toEqual([
        `../downloads/websocket/${name}.protospec.yaml`,
        `${SOURCE}profiles/websocket/witnesses/${name}.inputs.json`,
      ]);
    }
  });

  it('shows the layouts with their fields and bit widths', () => {
    const layout = wrapper.find('[data-test="layout-frame-header"]');
    const first = layout.findAll('tbody tr')[0]!.findAll('td').map((cell) => cell.text());
    expect(first.slice(0, 2)).toEqual(['fin', '1']);
    expect(layout.text()).toContain('payload-len');
  });

  it('links back to the profile page', () => {
    expect(hrefs(wrapper)).toContain('../docs/catalog/websocket');
  });
});

describe('ProfileLayer on the committed profile files', () => {
  it('shows every RFC 6455 requirement of the overlay and 85 accepted or edited and 29 rejected decisions', () => {
    const profile = websocket as Profile;
    const wrapper = mount(ProfileLayer, { props: { profile } });
    expect(wrapper.findAll('[data-test="requirements"] tbody tr')).toHaveLength(155);
    expect(wrapper.find('[data-test="decisions"]').text()).toContain('85 accepted or edited');
    expect(wrapper.find('[data-test="decisions"]').text()).toContain('29 rejected');
    const rejected = wrapper.findAll('[data-test="rejected"] tbody tr');
    expect(rejected).toHaveLength(29);
    for (const row of rejected) {
      expect(row.findAll('td')[1]!.text()).not.toBe('');
    }
    const decided = wrapper
      .findAll('[data-test="requirements"] tbody tr')
      .filter((row) => row.find('[data-test="decision"]').exists());
    expect(decided).toHaveLength(85);
    for (const row of decided) {
      expect(hrefs(row).some((href) => href.startsWith(`${SOURCE}profiles/websocket/proposals/`))).toBe(true);
    }
  });

  it('shows the RFC 868 tcp and udp machines with their witnesses and no decisions', () => {
    const wrapper = mount(ProfileLayer, { props: { profile: time as Profile } });
    const rows = wrapper.findAll('[data-test="machines"] tbody tr');
    expect(rows.map((row) => row.findAll('td')[0]!.text())).toEqual(['tcp', 'udp']);
    expect(hrefs(wrapper.find('[data-test="machines"]'))).toContain(
      `${SOURCE}profiles/time/witnesses/udp.inputs.json`,
    );
    expect(wrapper.find('[data-test="rejected"]').exists()).toBe(false);
    expect(wrapper.find('[data-test="decision"]').exists()).toBe(false);
    expect(wrapper.find('[data-test="decisions"]').text()).toContain('No review decision');
  });
});

describe('RfcView', () => {
  it('shows the reviewed layer of each profile it is given below the bulk data', () => {
    const wrapper = mount(RfcView, {
      props: { shard: shard6400 as ExplorerShard, rfc: 6455, profiles: [fixture as Profile] },
    });
    expect(wrapper.findAll('[data-test="reviewed-layer"]')).toHaveLength(1);
    const html = wrapper.html();
    expect(html.indexOf('data-test="sections"')).toBeLessThan(html.indexOf('data-test="reviewed-layer"'));
  });

  it('shows no reviewed layer for an RFC no profile locks', () => {
    const wrapper = mount(RfcView, { props: { shard: h3 as ExplorerShard, rfc: 9220, profiles: [] } });
    expect(wrapper.find('[data-test="reviewed-layer"]').exists()).toBe(false);
  });
});

describe('App', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    window.history.replaceState(null, '', '/');
  });

  function serve(files: Record<string, unknown>, fetched: string[]) {
    vi.stubGlobal('fetch', async (url: URL | string) => {
      const path = new URL(String(url)).pathname;
      fetched.push(path);
      return path in files
        ? new Response(JSON.stringify(files[path]), { status: 200 })
        : new Response('', { status: 404 });
    });
  }

  it('loads the profile file of each mark the index gives RFC 6455 and shows its reviewed layer', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=6455');
    const fetched: string[] = [];
    serve(
      {
        '/rfcspec/data/rfcs/06400.json': shard6400,
        '/rfcspec/data/rfc-index.json': index,
        '/rfcspec/data/profiles/websocket.json': fixture,
      },
      fetched,
    );
    const wrapper = mount(App);
    await flushPromises();
    expect(fetched.sort()).toEqual([
      '/rfcspec/data/profiles/websocket.json',
      '/rfcspec/data/rfc-index.json',
      '/rfcspec/data/rfcs/06400.json',
    ]);
    expect(wrapper.findAll('[data-test="reviewed-layer"]')).toHaveLength(1);
  });

  it('loads no profile file for RFC 9220, which no profile locks', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=9220');
    const fetched: string[] = [];
    serve({ '/rfcspec/data/rfcs/09200.json': h3, '/rfcspec/data/rfc-index.json': index }, fetched);
    const wrapper = mount(App);
    await flushPromises();
    expect(fetched.filter((path) => path.includes('/profiles/'))).toEqual([]);
    expect(wrapper.text()).toContain('Bootstrapping WebSockets with HTTP/3');
    expect(wrapper.find('[data-test="reviewed-layer"]').exists()).toBe(false);
  });

  it('says "No reviewed overlay yet" for a mark without a profile file and goes on to the next mark', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=6455');
    const marked = structuredClone(index);
    marked.rfcs.find((row) => row.rfc === 6455)!.profiles = ['draft', 'websocket'];
    serve(
      {
        '/rfcspec/data/rfcs/06400.json': shard6400,
        '/rfcspec/data/rfc-index.json': marked,
        '/rfcspec/data/profiles/websocket.json': fixture,
      },
      [],
    );
    const wrapper = mount(App);
    await flushPromises();
    const note = wrapper.find('[data-test="profile-note"][data-profile="draft"]');
    expect(note.text()).toContain('Profile draft: No reviewed overlay yet.');
    expect(wrapper.findAll('[data-test="reviewed-layer"]')).toHaveLength(1);
  });

  it('says so when a profile file is of another RFC than the mark, instead of dropping it', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=6455');
    serve(
      {
        '/rfcspec/data/rfcs/06400.json': shard6400,
        '/rfcspec/data/rfc-index.json': index,
        '/rfcspec/data/profiles/websocket.json': { ...fixture, rfc: 6456 },
      },
      [],
    );
    const wrapper = mount(App);
    await flushPromises();
    expect(wrapper.find('[data-test="reviewed-layer"]').exists()).toBe(false);
    expect(wrapper.find('[data-test="profile-note"][data-profile="websocket"]').text()).toContain(
      'Profile websocket: its file is of RFC 6456, not RFC 6455, so its reviewed layer is not shown.',
    );
  });

  it('says a profile file that fails to load other than by 404 could not be loaded, and goes on', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=6455');
    const marked = structuredClone(index);
    marked.rfcs.find((row) => row.rfc === 6455)!.profiles = ['broken', 'websocket'];
    vi.stubGlobal('fetch', async (url: URL | string) => {
      const path = new URL(String(url)).pathname;
      const files: Record<string, unknown> = {
        '/rfcspec/data/rfcs/06400.json': shard6400,
        '/rfcspec/data/rfc-index.json': marked,
        '/rfcspec/data/profiles/websocket.json': fixture,
      };
      if (path === '/rfcspec/data/profiles/broken.json') {
        return new Response('', { status: 500 });
      }
      return new Response(JSON.stringify(files[path]), { status: 200 });
    });
    const wrapper = mount(App);
    await flushPromises();
    expect(wrapper.find('[data-test="profile-note"][data-profile="broken"]').text()).toContain(
      'Profile broken: its reviewed layer could not be loaded: HTTP 500',
    );
    expect(wrapper.findAll('[data-test="reviewed-layer"]')).toHaveLength(1);
  });

  it('still shows the bulk data when the index cannot be loaded, saying the reviewed layer could not be', async () => {
    window.history.replaceState(null, '', '/rfcspec/explorer/?rfc=6455');
    serve({ '/rfcspec/data/rfcs/06400.json': shard6400 }, []);
    const wrapper = mount(App);
    await flushPromises();
    expect(wrapper.text()).toContain('The WebSocket Protocol');
    expect(wrapper.text()).toContain('The reviewed layer could not be loaded');
  });
});
