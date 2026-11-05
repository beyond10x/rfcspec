<script setup lang="ts">
import { computed } from 'vue';
import type { ExplorerShard } from '../generated/types.ts';
import { counted, findRfc, isIndexed, sectionHref } from '../lib/shard.ts';
import RfcNumbers from './RfcNumbers.vue';

const props = defineProps<{ shard: ExplorerShard | null; rfc: number }>();

const entry = computed(() => (props.shard === null ? null : findRfc(props.shard, props.rfc)));

/** A citing section by its anchor: its number and title where the RFC lists it, else the anchor. */
const citing = computed(() => {
  const named = new Map<string, string>();
  for (const section of entry.value?.sections ?? []) {
    if (section.anchor !== undefined && section.number !== undefined) {
      named.set(section.anchor, [section.number, section.title].filter((part) => part !== undefined).join(' '));
    }
  }
  return (anchor: string | undefined): string => (anchor === undefined ? '' : (named.get(anchor) ?? anchor));
});

const graph = computed(() => {
  const found = entry.value;
  if (found === null) {
    return [];
  }
  return [
    { key: 'obsoletes', label: 'Obsoletes', numbers: found.obsoletes },
    { key: 'obsoleted_by', label: 'Obsoleted by', numbers: found.obsoleted_by },
    { key: 'updates', label: 'Updates', numbers: found.updates },
    { key: 'updated_by', label: 'Updated by', numbers: found.updated_by },
  ];
});
</script>

<template>
  <p v-if="entry === null || shard === null" class="missing">RFC {{ rfc }} is not in the index.</p>
  <article v-else>
    <h1>RFC {{ entry.rfc }}: {{ entry.title }}</h1>
    <p>
      <a :href="sectionHref(entry.rfc, undefined)">Read RFC {{ entry.rfc }} at the RFC Editor</a>
    </p>
    <table class="meta">
      <tbody>
        <tr><th>Status</th><td>{{ entry.status }}</td></tr>
        <tr><th>Stream</th><td>{{ entry.stream }}</td></tr>
        <tr><th>Area</th><td>{{ entry.area ?? '-' }}</td></tr>
        <tr><th>Date</th><td>{{ entry.date }}</td></tr>
        <tr><th>Source read</th><td>{{ entry.format === 'xml' ? 'XML' : 'text' }}</td></tr>
        <tr><th>SHA-256</th><td><code>{{ entry.sha256 }}</code></td></tr>
        <tr><th>Requirements</th><td>{{ counted(entry.counts) }}</td></tr>
        <tr v-for="edge in graph" :key="edge.key" :data-test="edge.key">
          <th>{{ edge.label }}</th>
          <td><RfcNumbers :shard="shard" :numbers="edge.numbers" /></td>
        </tr>
      </tbody>
    </table>

    <h2>Requirements by section</h2>
    <p class="note">
      BCP 14 requirements found by the mechanical extraction, unreviewed; only sections that hold one
      are listed.
    </p>
    <p v-if="entry.sections.length === 0">No BCP 14 requirement was found.</p>
    <table v-else data-test="sections">
      <thead>
        <tr><th>Section</th><th>Title</th><th>Requirements</th></tr>
      </thead>
      <tbody>
        <tr v-for="(section, i) in entry.sections" :key="section.number ?? `none-${i}`">
          <td>
            <a v-if="section.number !== undefined" :href="sectionHref(entry.rfc, section.number)">{{ section.number }}</a>
            <span v-else>outside every numbered section</span>
          </td>
          <td>{{ section.title ?? '' }}</td>
          <td>{{ counted(section.counts) }}</td>
        </tr>
      </tbody>
    </table>

    <template v-if="entry.references.length > 0">
      <h2>References</h2>
      <p class="note">Each citation of another RFC in the RFCXML source, in source order.</p>
      <table data-test="references">
        <thead>
          <tr><th>Cited RFC</th><th>Kind</th><th>Cited section</th><th>Citing section</th></tr>
        </thead>
        <tbody>
          <tr v-for="(reference, i) in entry.references" :key="i">
            <td>
              <a v-if="isIndexed(shard, reference.target_rfc)" :href="`?rfc=${reference.target_rfc}`">RFC {{ reference.target_rfc }}</a>
              <span v-else>RFC {{ reference.target_rfc }}</span>
            </td>
            <td>{{ reference.kind }}</td>
            <td>
              <a v-if="reference.target_section !== undefined" :href="sectionHref(reference.target_rfc, reference.target_section)">{{ reference.target_section }}</a>
            </td>
            <td>{{ citing(reference.section) }}</td>
          </tr>
        </tbody>
      </table>
    </template>
  </article>
</template>
