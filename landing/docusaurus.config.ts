import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/// Where the site is served. The editor is a separate app under /app of the
/// same host, so links to it are written out in full: docusaurus checks
/// internal links against its own pages, and /app is not one of them.
const url = "https://iblang.on.accelley.com";

const config: Config = {
    title: "IB Pseudocode Web IDE",
    tagline:
        "Your All-in-One IDE for Running, Debugging, and Perfecting IB Pseudocode!",
    favicon: "img/favicon.svg",

    // Set the production url of your site here
    url: url,
    // Set the /<baseUrl>/ pathname under which your site is served
    // For GitHub pages deployment, it is often '/<projectName>/'
    baseUrl: "/",

    // GitHub pages deployment config.
    // If you aren't using GitHub pages, you don't need these.
    organizationName: "FilipToth",
    projectName: "ib-lang",

    onBrokenLinks: "throw",
    onBrokenMarkdownLinks: "warn",

    // Even if you don't use internationalization, you can use this field to set
    // useful metadata like html lang. For example, if your site is Chinese, you
    // may want to replace "en" with "zh-Hans".
    i18n: {
        defaultLocale: "en",
        locales: ["en"],
    },

    presets: [
        [
            "classic",
            {
                docs: {
                    sidebarPath: "./sidebars.ts",
                    // Please change this to your repo.
                    // Remove this to remove the "edit this page" links.
                    editUrl:
                        "https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/",
                },
                blog: {
                    showReadingTime: true,
                    feedOptions: {
                        type: ["rss", "atom"],
                        xslt: true,
                    },
                    // Please change this to your repo.
                    // Remove this to remove the "edit this page" links.
                    editUrl:
                        "https://github.com/facebook/docusaurus/tree/main/packages/create-docusaurus/templates/shared/",
                    // Useful options to enforce blogging best practices
                    onInlineTags: "warn",
                    onInlineAuthors: "warn",
                    onUntruncatedBlogPosts: "warn",
                },
                theme: {
                    customCss: "./src/css/custom.css",
                },
            } satisfies Preset.Options,
        ],
    ],

    themeConfig: {
        // Replace with your project's social card
        image: "img/docusaurus-social-card.jpg",
        navbar: {
            title: "IB Pseudocode Web IDE",
            items: [
                {
                    href: `${url}/app`,
                    label: "App",
                    position: "left",
                },
                {
                    type: "docSidebar",
                    sidebarId: "tutorialSidebar",
                    position: "left",
                    label: "Docs",
                },
                {
                    href: "https://github.com/FilipToth/ib-lang/",
                    label: "GitHub",
                    position: "right",
                },
            ],
        },
        footer: {
            style: "dark",
            // a flat list, so the links sit in one centred row rather
            // than under titles of their own
            links: [
                {
                    label: "Docs",
                    to: "/docs/intro",
                },
                {
                    label: "GitHub",
                    href: "https://github.com/FilipToth/ib-lang",
                },
            ],
            copyright: `Copyright © ${new Date().getFullYear()} Filip Toth. Built with Docusaurus.`,
        },
        prism: {
            theme: prismThemes.github,
            darkTheme: prismThemes.dracula,
        },
    } satisfies Preset.ThemeConfig,
};

export default config;
