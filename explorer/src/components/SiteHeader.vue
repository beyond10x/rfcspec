<script setup lang="ts">
/** The documentation site's navbar (`website/docusaurus.config.ts`), its links relative to the explorer. */
const LEFT = [
  { label: 'Documentation', href: '../docs' },
  { label: 'Design', href: '../docs/design' },
  { label: 'WebSocket', href: '../docs/examples/websocket' },
  { label: 'Protocol catalogue', href: '../docs/catalog' },
  { label: 'RFC explorer', href: './', current: true },
  { label: 'Status', href: '../docs/status' },
];
</script>

<template>
  <nav aria-label="Main" class="navbar">
    <div class="navbar__inner">
      <div class="navbar__items">
        <a class="navbar__brand" href="../"><b class="navbar__title">RFCspec</b></a>
        <a
          v-for="link in LEFT"
          :key="link.label"
          :class="['navbar__item', 'navbar__link', { 'navbar__link--active': link.current }]"
          :aria-current="link.current ? 'page' : undefined"
          :href="link.href"
          >{{ link.label }}</a
        >
      </div>
      <div class="navbar__items navbar__items--right">
        <a class="navbar__item navbar__link" href="https://github.com/beyond10x/rfcspec" target="_blank" rel="noopener noreferrer"
          >GitHub</a
        >
      </div>
    </div>
  </nav>
</template>
