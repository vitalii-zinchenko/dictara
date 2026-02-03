/**
 * Post-build script to generate static HTML pages for SEO.
 *
 * GitHub Pages serves directory/index.html for /directory/ paths natively,
 * so we generate sub-route HTML files with route-specific meta tags.
 * Also copies index.html as 404.html for SPA fallback on unknown routes.
 */

import { readFileSync, writeFileSync, mkdirSync, copyFileSync } from "fs";
import { join } from "path";

const distDir = join(import.meta.dirname, "..", "dist");
const indexHtml = readFileSync(join(distDir, "index.html"), "utf-8");

// Route-specific meta tag overrides
const routes = [
  {
    path: "privacy",
    title: "Privacy Policy - Dictara",
    description:
      "Dictara is committed to protecting your privacy. Learn how we handle your data — audio recordings are temporary, API keys stay in your system keychain, and we collect zero analytics.",
    canonical: "https://dictara.app/privacy",
    staticContent: `
      <header><nav><a href="/">Dictara</a></nav></header>
      <main>
        <h1>Privacy Policy</h1>
        <p>Dictara is committed to protecting your privacy. This policy explains how we handle your data.</p>
        <h2>Information We Collect</h2>
        <p>Audio recordings are temporarily stored on your device only during transcription. API keys are stored securely in your system's keychain. Settings are stored locally on your device.</p>
        <h2>What We Don't Collect</h2>
        <p>Dictara does not collect analytics or telemetry data, usage statistics, personal information, your transcribed text, or any data beyond what's needed for transcription.</p>
      </main>
      <footer><a href="/">Back to Dictara</a> · <a href="/terms">Terms of Service</a></footer>`,
  },
  {
    path: "terms",
    title: "Terms of Service - Dictara",
    description:
      "Terms of service for Dictara, a free open-source desktop application that transcribes speech to text using AI. Bring your own API key, pay providers directly.",
    canonical: "https://dictara.app/terms",
    staticContent: `
      <header><nav><a href="/">Dictara</a></nav></header>
      <main>
        <h1>Terms of Service</h1>
        <p>By using Dictara, you agree to these terms.</p>
        <h2>What Dictara Is</h2>
        <p>Dictara is a free, open-source desktop application that transcribes your speech to text using AI services (OpenAI or Azure OpenAI). You bring your own API keys and pay those providers directly for usage.</p>
        <h2>Open Source</h2>
        <p>Dictara is open source under the MIT License. You can view, modify, and distribute the code.</p>
      </main>
      <footer><a href="/">Back to Dictara</a> · <a href="/privacy">Privacy Policy</a></footer>`,
  },
];

/**
 * Replace meta tags in the HTML template for a specific route.
 */
function generatePageHtml(route) {
  let html = indexHtml;

  // Replace title
  html = html.replace(
    /<title>.*?<\/title>/,
    `<title>${route.title}</title>`
  );
  html = html.replace(
    /<meta name="title" content=".*?" \/>/,
    `<meta name="title" content="${route.title}" />`
  );

  // Replace description
  html = html.replace(
    /<meta name="description" content=".*?" \/>/,
    `<meta name="description" content="${route.description}" />`
  );

  // Replace canonical
  html = html.replace(
    /<link rel="canonical" href=".*?" \/>/,
    `<link rel="canonical" href="${route.canonical}" />`
  );

  // Replace OG tags
  html = html.replace(
    /<meta property="og:url" content=".*?" \/>/,
    `<meta property="og:url" content="${route.canonical}" />`
  );
  html = html.replace(
    /<meta property="og:title" content=".*?" \/>/,
    `<meta property="og:title" content="${route.title}" />`
  );
  html = html.replace(
    /<meta property="og:description" content=".*?" \/>/,
    `<meta property="og:description" content="${route.description}" />`
  );

  // Replace Twitter tags
  html = html.replace(
    /<meta property="twitter:url" content=".*?" \/>/,
    `<meta property="twitter:url" content="${route.canonical}" />`
  );
  html = html.replace(
    /<meta property="twitter:title" content=".*?" \/>/,
    `<meta property="twitter:title" content="${route.title}" />`
  );
  html = html.replace(
    /<meta property="twitter:description" content=".*?" \/>/,
    `<meta property="twitter:description" content="${route.description}" />`
  );

  // Replace static content in #root
  html = html.replace(
    /(<div id="root">)([\s\S]*?)(<\/div>\s*<script)/,
    `$1${route.staticContent}\n    $3`
  );

  return html;
}

// Generate sub-route pages
for (const route of routes) {
  const routeDir = join(distDir, route.path);
  mkdirSync(routeDir, { recursive: true });
  const html = generatePageHtml(route);
  writeFileSync(join(routeDir, "index.html"), html);
  console.log(`Generated: ${route.path}/index.html`);
}

// Copy index.html as 404.html (SPA fallback for unknown routes)
copyFileSync(join(distDir, "index.html"), join(distDir, "404.html"));
console.log("Generated: 404.html");

console.log("SEO pages generated successfully.");
