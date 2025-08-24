import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  // By default, Docusaurus generates a sidebar from the docs folder structure
  tutorialSidebar: [

  'intro',
{
  type: 'category',
      label: 'Installation',
    items: ['Installation/Packaging status', 'Installation/Binary', 'Installation/Cargo', 'Installation/Build from source', 'Installation/NetBSD', 'Installation/Arch Linux', 'Installation/NixOS', 'Installation/Docker'],
},
{
  type: 'category',
      label: 'Configuration',
    items: [
  'Configuration/configuration-intro',
  'Configuration/display',
  'Configuration/logging',
  'Configuration/engine',
],
},
{
  type: 'category',
      label: 'Multiplayer',
    items: ['Multiplayer/Local multiplayer', 'Multiplayer/Online multiplayer'],
},
{
  type: 'category',
      label: 'Code Architecture',
    items: ['Code Architecture/Intro', 'Code Architecture/Pieces', 'Code Architecture/Game'],
},
],



};

export default sidebars;
