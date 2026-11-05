<script setup lang="ts">
import { computed, onMounted, reactive, watch } from 'vue';
import type { ExplorerIndex } from '../generated/types.ts';
import { counted } from '../lib/shard.ts';
import {
  type Chosen,
  type Filter,
  NO_REQUIREMENTS,
  canonical,
  filterRows,
  numberRange,
  offered,
} from '../lib/search.ts';

const props = defineProps<{ index: ExplorerIndex; initial: Filter }>();
const emit = defineEmits<{ change: [filter: Filter] }>();

const offers = offered(props.index.rfcs);
const read = canonical(props.initial, offers);
const filter = reactive<Filter>({ ...read.filter });
watch(filter, () => emit('change', { ...filter }), { deep: true });
onMounted(() => {
  if (JSON.stringify(read.filter) !== JSON.stringify(props.initial)) {
    emit('change', { ...filter });
  }
});

/** A select's label and the text of each value it offers. */
const SELECTS: { key: Chosen; label: string; text: (value: string) => string }[] = [
  { key: 'status', label: 'Status', text: (value) => value },
  { key: 'stream', label: 'Stream', text: (value) => value },
  { key: 'area', label: 'Area', text: (value) => value },
  { key: 'format', label: 'Source', text: (value) => (value === 'xml' ? 'XML' : value) },
  {
    key: 'level',
    label: 'Requirements',
    text: (value) => (value === NO_REQUIREMENTS ? 'none' : `at least one ${value}`),
  },
];

/** The filters whose value no select offers; each is kept, shown as its own option and named. */
const unknown = computed(() =>
  SELECTS.filter(({ key }) => filter[key] !== '' && !offers[key].includes(filter[key])),
);
const rows = computed(() => filterRows(props.index.rfcs, filter));
const unreadable = computed(() => numberRange(filter.number) === 'invalid');
</script>

<template>
  <section>
    <h1>RFC explorer</h1>
    <p class="note">
      Every RFC the RFC Editor publishes as plain text or RFCXML, with its BCP 14 requirement counts
      found by the mechanical extraction, unreviewed. Filter the list, then open an RFC to see its
      requirements by section and its References. A filtered list is a link.
    </p>
    <form class="filters" @submit.prevent>
      <label>Number or range <input v-model="filter.number" name="number" inputmode="numeric" size="11" placeholder="6000-6999" /></label>
      <label>Title words <input v-model="filter.title" name="title" size="24" /></label>
      <label v-for="select in SELECTS" :key="select.key">
        {{ select.label }}
        <select v-model="filter[select.key]" :name="select.key">
          <option value="">any</option>
          <option v-for="value in offers[select.key]" :key="value" :value="value">{{ select.text(value) }}</option>
          <option v-if="filter[select.key] !== '' && !offers[select.key].includes(filter[select.key])" :value="filter[select.key]">
            {{ filter[select.key] }} (not a known value)
          </option>
        </select>
      </label>
    </form>
    <p v-if="unreadable" class="missing">Enter a number such as 6455 or a range such as 6000-6999.</p>
    <p v-for="select in unknown" :key="select.key" class="missing" :data-test="`unknown-${select.key}`">
      {{ select.label }} "{{ filter[select.key] }}" is not a known value; no RFC matches it.
    </p>
    <p data-test="count">{{ rows.length }} of {{ index.rfcs.length }} RFCs</p>
    <table data-test="results">
      <thead>
        <tr>
          <th>RFC</th><th>Title</th><th>Status</th><th>Stream</th><th>Area</th><th>Date</th>
          <th>Requirements</th><th>Source</th><th>Profile</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in rows" :key="row.rfc">
          <td><a class="rfc" :href="`?rfc=${row.rfc}`">{{ row.rfc }}</a></td>
          <td>{{ row.title }}</td>
          <td>{{ row.status }}</td>
          <td>{{ row.stream }}</td>
          <td>{{ row.area ?? '-' }}</td>
          <td>{{ row.date }}</td>
          <td>{{ counted(row.counts) }}</td>
          <td>{{ row.format === 'xml' ? 'XML' : 'text' }}</td>
          <td>{{ row.profiles.join(', ') }}</td>
        </tr>
      </tbody>
    </table>
  </section>
</template>
