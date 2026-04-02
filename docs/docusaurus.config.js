// @ts-check
import { themes as prismThemes } from 'prism-react-renderer';

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Capybara-Lang',
  tagline: '🐾 한국어 친화적 실험적 프로그래밍 언어',
  favicon: 'img/favicon.ico',

  url: 'https://DevNergis.github.io',
  baseUrl: '/Capybara-Lang/',

  organizationName: 'DevNergis',
  projectName: 'Capybara-Lang',
  deploymentBranch: 'gh-pages',
  trailingSlash: false,

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'ko',
    locales: ['ko'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: './sidebars.js',
          editUrl:
            'https://github.com/DevNergis/Capybara-Lang/tree/main/docs/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      navbar: {
        title: 'Capybara-Lang',
        logo: {
          alt: 'Capybara-Lang Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'docsSidebar',
            position: 'left',
            label: '문서',
          },
          {
            href: 'https://github.com/DevNergis/Capybara-Lang',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: '문서',
            items: [
              {
                label: '소개',
                to: '/docs/intro',
              },
              {
                label: '문법',
                to: '/docs/grammar',
              },
            ],
          },
          {
            title: '더 보기',
            items: [
              {
                label: 'GitHub',
                href: 'https://github.com/DevNergis/Capybara-Lang',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} DevNergis. Built with Docusaurus.`,
      },
      prism: {
        theme: prismThemes.github,
        darkTheme: prismThemes.dracula,
      },
      colorMode: {
        defaultMode: 'light',
        disableSwitch: false,
        respectPrefersColorScheme: true,
      },
    }),
};

export default config;
