<script setup lang="ts">
import { onMounted, ref } from 'vue';
import type { ExplorerIndex, ExplorerShard } from './generated/types.ts';
import type { RfcspecProfileExplorerProfile as Profile } from './generated-profile/types.ts';
import { requestedRfc, shardUrl } from './lib/shard.ts';
import { type ProfileNote, profileUrl } from './lib/profile.ts';
import { type Filter, indexUrl, readFilter, writeFilter } from './lib/search.ts';
import ListView from './components/ListView.vue';
import RfcView from './components/RfcView.vue';
import SiteFooter from './components/SiteFooter.vue';
import SiteHeader from './components/SiteHeader.vue';

type State =
  | { kind: 'listing' }
  | { kind: 'listed'; index: ExplorerIndex; filter: Filter }
  | { kind: 'unlisted'; reason: string }
  | { kind: 'unknown'; asked: string }
  | { kind: 'loading'; rfc: number }
  | { kind: 'loaded'; rfc: number; shard: ExplorerShard | null; layer: Layer }
  | { kind: 'failed'; rfc: number; reason: string };

/**
 * The profile files loaded for an RFC; a note for each mark whose reviewed layer is not shown, and
 * why; an error when the index naming the marks could not be read.
 */
type Layer = { profiles: Profile[]; notes: ProfileNote[]; error: string | null };

const state = ref<State>({ kind: 'listing' });
const typed = ref('');

/**
 * The colour mode of the documentation site: the choice its colour-mode switch stored for this
 * site, else the system preference, as the site's own head script reads them.
 */
function colourMode(): 'light' | 'dark' {
  try {
    const stored = window.localStorage.getItem('theme-cdd');
    if (stored === 'light' || stored === 'dark') {
      return stored;
    }
  } catch {
    // Storage may be unavailable; the system preference decides.
  }
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

document.documentElement.setAttribute('data-theme', colourMode());

/** The front page: the index, filtered by the query string. */
async function list(): Promise<void> {
  try {
    const response = await fetch(indexUrl(window.location.href));
    if (!response.ok) {
      state.value = { kind: 'unlisted', reason: `HTTP ${response.status}` };
    } else {
      const index = (await response.json()) as ExplorerIndex;
      state.value = { kind: 'listed', index, filter: readFilter(window.location.search) };
    }
  } catch (error) {
    state.value = { kind: 'unlisted', reason: String(error) };
  }
}

/**
 * The reviewed layer of an RFC: the file of each profile the index marks it with, the one source
 * of which profiles lock it. None for an RFC without a mark.
 */
async function layer(rfc: number): Promise<Layer> {
  let marks: string[];
  try {
    const response = await fetch(indexUrl(window.location.href));
    if (!response.ok) {
      return { profiles: [], notes: [], error: `the RFC index: HTTP ${response.status}` };
    }
    const index = (await response.json()) as ExplorerIndex;
    marks = index.rfcs.find((row) => row.rfc === rfc)?.profiles ?? [];
  } catch (error) {
    return { profiles: [], notes: [], error: String(error) };
  }
  const profiles: Profile[] = [];
  const notes: ProfileNote[] = [];
  for (const name of marks) {
    const note = (text: string) => notes.push({ profile: name, text: `Profile ${name}: ${text}` });
    try {
      const file = await fetch(profileUrl(name, window.location.href));
      if (file.status === 404) {
        note('No reviewed overlay yet.');
      } else if (!file.ok) {
        note(`its reviewed layer could not be loaded: HTTP ${file.status}`);
      } else {
        const profile = (await file.json()) as Profile;
        if (profile.rfc === rfc) {
          profiles.push(profile);
        } else {
          note(`its file is of RFC ${profile.rfc}, not RFC ${rfc}, so its reviewed layer is not shown.`);
        }
      }
    } catch (error) {
      note(`its reviewed layer could not be loaded: ${String(error)}`);
    }
  }
  return { profiles, notes, error: null };
}

/** The RFC view of the number ?rfc= names, its shard and its reviewed layer read together. */
async function view(rfc: number): Promise<void> {
  state.value = { kind: 'loading', rfc };
  const reviewed = layer(rfc);
  try {
    const response = await fetch(shardUrl(rfc, window.location.href));
    if (response.status === 404) {
      state.value = { kind: 'loaded', rfc, shard: null, layer: await reviewed };
    } else if (!response.ok) {
      state.value = { kind: 'failed', rfc, reason: `HTTP ${response.status}` };
    } else {
      const shard = (await response.json()) as ExplorerShard;
      state.value = { kind: 'loaded', rfc, shard, layer: await reviewed };
    }
  } catch (error) {
    state.value = { kind: 'failed', rfc, reason: String(error) };
  }
}

onMounted(async () => {
  const asked = new URLSearchParams(window.location.search).get('rfc');
  if (asked === null || asked === '') {
    await list();
    return;
  }
  const rfc = requestedRfc(window.location.search);
  if (rfc === null) {
    state.value = { kind: 'unknown', asked };
    return;
  }
  await view(rfc);
});

/** Keep the filter in the query string, so the filtered list is a link. */
function filtered(filter: Filter): void {
  window.history.replaceState(null, '', `${window.location.pathname}${writeFilter(filter)}`);
}

function open(): void {
  const rfc = requestedRfc(`?rfc=${typed.value.trim()}`);
  if (rfc !== null) {
    window.location.search = `?rfc=${rfc}`;
  }
}
</script>

<template>
  <SiteHeader />
  <main>
    <div class="toolbar">
      <a href="../docs/catalog/rfcs">RFC index</a>
      <form @submit.prevent="open">
        <label>RFC <input v-model="typed" inputmode="numeric" size="6" /></label>
        <button type="submit">Open</button>
      </form>
    </div>
    <p v-if="state.kind === 'listing'">Loading the RFC index…</p>
    <p v-else-if="state.kind === 'unlisted'" class="missing">The RFC index could not be loaded: {{ state.reason }}</p>
    <ListView v-else-if="state.kind === 'listed'" :index="state.index" :initial="state.filter" @change="filtered" />
    <p v-else-if="state.kind === 'unknown'" class="missing">RFC {{ state.asked }} is not in the index.</p>
    <p v-else-if="state.kind === 'loading'">Loading RFC {{ state.rfc }}…</p>
    <p v-else-if="state.kind === 'failed'" class="missing">RFC {{ state.rfc }} could not be loaded: {{ state.reason }}</p>
    <RfcView
      v-else
      :shard="state.shard"
      :rfc="state.rfc"
      :profiles="state.layer.profiles"
      :notes="state.layer.notes"
      :layer-error="state.layer.error"
    />
  </main>
  <SiteFooter />
</template>
