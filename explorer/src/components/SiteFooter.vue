<script setup lang="ts">
/** The documentation site's footer (`website/docusaurus.config.ts`), its links relative to the explorer. */
const COLUMNS = [
  {
    title: 'Documentation',
    items: [
      { label: 'Overview', href: '../docs' },
      { label: 'Getting started', href: '../docs/getting-started' },
      { label: 'The catalogue', href: '../docs/concepts/catalogue' },
      { label: 'Communication semantics', href: '../docs/concepts/communication-semantics' },
      { label: 'Lowering to ESS', href: '../docs/concepts/lowering-to-ess' },
      { label: 'WebSocket', href: '../docs/examples/websocket' },
      { label: 'Design', href: '../docs/design' },
      { label: 'Protocol catalogue', href: '../docs/catalog' },
      { label: 'RFC explorer', href: './' },
      { label: 'Status', href: '../docs/status' },
    ],
  },
  {
    title: 'Project',
    items: [
      { label: 'Source', href: 'https://github.com/beyond10x/rfcspec' },
      { label: 'Changelog', href: 'https://github.com/beyond10x/rfcspec/blob/main/CHANGELOG.md' },
      { label: 'ESS', href: 'https://beyond10x.github.io/ess/' },
      { label: 'RFC Editor', href: 'https://www.rfc-editor.org/' },
    ],
  },
];

function external(href: string): boolean {
  return href.startsWith('https:');
}
</script>

<template>
  <footer class="footer">
    <div class="container">
      <div class="footer__links">
        <div v-for="column in COLUMNS" :key="column.title" class="footer__col">
          <div class="footer__title">{{ column.title }}</div>
          <ul class="footer__items">
            <li v-for="item in column.items" :key="item.label" class="footer__item">
              <a
                class="footer__link-item"
                :href="item.href"
                :target="external(item.href) ? '_blank' : undefined"
                :rel="external(item.href) ? 'noopener noreferrer' : undefined"
                >{{ item.label }}</a
              >
            </li>
          </ul>
        </div>
      </div>
      <div class="footer__bottom">
        <div class="footer__copyright">A beyond10x project. RFCspec · Apache-2.0.</div>
      </div>
    </div>
  </footer>
</template>
