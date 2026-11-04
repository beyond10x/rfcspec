import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import {withProductSite} from '@beyond10x/docs-system/product-site';

const config: Config = {
  title: 'RFCspec',
  tagline: 'RFCs as executable specifications.',

  future: {
    v4: true,
  },

  url: 'https://beyond10x.github.io',
  baseUrl: '/rfcspec/',

  organizationName: 'beyond10x',
  projectName: 'rfcspec',
  deploymentBranch: 'gh-pages',
  trailingSlash: false,

  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',

  markdown: {
    format: 'detect',
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
    mermaid: true,
  },

  themes: ['@docusaurus/theme-mermaid'],

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: 'docs',
          editUrl: 'https://github.com/beyond10x/rfcspec/tree/main/website/',
        },
        blog: false,
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    navbar: {
      title: 'RFCspec',
      items: [
        {to: '/docs/', label: 'Documentation', position: 'left'},
        {to: '/docs/design', label: 'Design', position: 'left'},
        {to: '/docs/examples/websocket', label: 'WebSocket', position: 'left'},
        {to: '/docs/catalog', label: 'Protocol catalogue', position: 'left'},
        {to: '/docs/status', label: 'Status', position: 'left'},
        {href: 'https://github.com/beyond10x/rfcspec', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      links: [
        {
          title: 'Documentation',
          items: [
            {label: 'Overview', to: '/docs/'},
            {label: 'Getting started', to: '/docs/getting-started'},
            {label: 'The catalogue', to: '/docs/concepts/catalogue'},
            {label: 'Communication semantics', to: '/docs/concepts/communication-semantics'},
            {label: 'Lowering to ESS', to: '/docs/concepts/lowering-to-ess'},
            {label: 'WebSocket', to: '/docs/examples/websocket'},
            {label: 'Design', to: '/docs/design'},
            {label: 'Protocol catalogue', to: '/docs/catalog'},
            {label: 'Status', to: '/docs/status'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'Source', href: 'https://github.com/beyond10x/rfcspec'},
            {label: 'Changelog', href: 'https://github.com/beyond10x/rfcspec/blob/main/CHANGELOG.md'},
            {label: 'ESS', href: 'https://beyond10x.github.io/ess/'},
            {label: 'RFC Editor', href: 'https://www.rfc-editor.org/'},
          ],
        },
      ],
      copyright: 'A beyond10x project. RFCspec · Apache-2.0.',
    },
  } satisfies Preset.ThemeConfig,
};

export default withProductSite(config, {landing: './product.json', mark: 'RS'});
