<script setup lang="ts">
import { onMounted, ref } from 'vue';
import type { ExplorerIndex, ExplorerShard } from './generated/types.ts';
import { requestedRfc, shardUrl } from './lib/shard.ts';
import { type Filter, indexUrl, readFilter, writeFilter } from './lib/search.ts';
import ListView from './components/ListView.vue';
import RfcView from './components/RfcView.vue';

type State =
  | { kind: 'listing' }
  | { kind: 'listed'; index: ExplorerIndex; filter: Filter }
  | { kind: 'unlisted'; reason: string }
  | { kind: 'unknown'; asked: string }
  | { kind: 'loading'; rfc: number }
  | { kind: 'loaded'; rfc: number; shard: ExplorerShard | null }
  | { kind: 'failed'; rfc: number; reason: string };

const state = ref<State>({ kind: 'listing' });
const typed = ref('');

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

/** The RFC view of the number ?rfc= names. */
async function view(rfc: number): Promise<void> {
  state.value = { kind: 'loading', rfc };
  try {
    const response = await fetch(shardUrl(rfc, window.location.href));
    if (response.status === 404) {
      state.value = { kind: 'loaded', rfc, shard: null };
    } else if (!response.ok) {
      state.value = { kind: 'failed', rfc, reason: `HTTP ${response.status}` };
    } else {
      state.value = { kind: 'loaded', rfc, shard: (await response.json()) as ExplorerShard };
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
  <main>
    <nav>
      <span><a href="../">RFCspec</a> · <a href="./">RFC explorer</a> · <a href="../docs/catalog/rfcs">RFC index</a></span>
      <form @submit.prevent="open">
        <label>RFC <input v-model="typed" inputmode="numeric" size="6" /></label>
        <button type="submit">Open</button>
      </form>
    </nav>
    <p v-if="state.kind === 'listing'">Loading the RFC index…</p>
    <p v-else-if="state.kind === 'unlisted'" class="missing">The RFC index could not be loaded: {{ state.reason }}</p>
    <ListView v-else-if="state.kind === 'listed'" :index="state.index" :initial="state.filter" @change="filtered" />
    <p v-else-if="state.kind === 'unknown'" class="missing">RFC {{ state.asked }} is not in the index.</p>
    <p v-else-if="state.kind === 'loading'">Loading RFC {{ state.rfc }}…</p>
    <p v-else-if="state.kind === 'failed'" class="missing">RFC {{ state.rfc }} could not be loaded: {{ state.reason }}</p>
    <RfcView v-else :shard="state.shard" :rfc="state.rfc" />
  </main>
</template>
