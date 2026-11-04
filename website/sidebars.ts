import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    'index',
    'getting-started',
    {
      type: 'category',
      label: 'Concepts',
      collapsed: false,
      items: [
        'concepts/rfc-kinds',
        'concepts/catalogue',
        'concepts/communication-semantics',
        'concepts/lowering-to-ess',
        'concepts/ai-proposals',
      ],
    },
    {
      type: 'category',
      label: 'Examples',
      collapsed: false,
      items: ['examples/websocket'],
    },
    'design',
    'status',
  ],
};

export default sidebars;
