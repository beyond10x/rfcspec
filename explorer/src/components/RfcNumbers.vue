<script setup lang="ts">
import type { ExplorerShard } from '../generated/types.ts';
import { isIndexed } from '../lib/shard.ts';

defineProps<{ shard: ExplorerShard; numbers: number[] }>();
</script>

<template>
  <span v-if="numbers.length === 0">-</span>
  <template v-for="(n, i) in numbers" :key="n">
    <span v-if="i > 0">, </span>
    <a v-if="isIndexed(shard, n)" :href="`?rfc=${n}`">{{ n }}</a>
    <span v-else>{{ n }}</span>
  </template>
</template>
