<script setup lang="ts">
import { onMounted, ref } from 'vue';
import type { ExplorerShard } from './generated/types.ts';
import { requestedRfc, shardUrl } from './lib/shard.ts';
import RfcView from './components/RfcView.vue';

type State =
  | { kind: 'ask' }
  | { kind: 'unknown'; asked: string }
  | { kind: 'loading'; rfc: number }
  | { kind: 'loaded'; rfc: number; shard: ExplorerShard | null }
  | { kind: 'failed'; rfc: number; reason: string };

const state = ref<State>({ kind: 'ask' });
const typed = ref('');

onMounted(async () => {
  const rfc = requestedRfc(window.location.search);
  if (rfc === null) {
    const asked = new URLSearchParams(window.location.search).get('rfc');
    if (asked !== null && asked !== '') {
      state.value = { kind: 'unknown', asked };
    }
    return;
  }
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
});

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
      <a href="../">RFCspec</a> · <a href="../docs/catalog/rfcs">RFC index</a>
      <form @submit.prevent="open">
        <label>RFC <input v-model="typed" inputmode="numeric" size="6" /></label>
        <button type="submit">Open</button>
      </form>
    </nav>
    <p v-if="state.kind === 'ask'">Enter an RFC number to see its requirements by section and its References.</p>
    <p v-else-if="state.kind === 'unknown'" class="missing">RFC {{ state.asked }} is not in the index.</p>
    <p v-else-if="state.kind === 'loading'">Loading RFC {{ state.rfc }}…</p>
    <p v-else-if="state.kind === 'failed'" class="missing">RFC {{ state.rfc }} could not be loaded: {{ state.reason }}</p>
    <RfcView v-else :shard="state.shard" :rfc="state.rfc" />
  </main>
</template>
