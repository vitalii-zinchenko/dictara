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

// Extract Vite-generated asset tags from the built index.html
const cssLinks = indexHtml.match(/<link rel="stylesheet"[^>]+>/g) ?? [];
const scriptTags = indexHtml.match(/<script type="module"[^>]*>[\s\S]*?<\/script>/g) ?? [];

// Route-specific page definitions
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
 * Build a complete HTML page from a clean template.
 */
function buildPage({ title, description, canonical, staticContent }) {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/png" href="/favicon.png" />
    <link rel="apple-touch-icon" href="/icon.png" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>${title}</title>
    <meta name="title" content="${title}" />
    <meta name="description" content="${description}" />
    <link rel="canonical" href="${canonical}" />
    <meta property="og:type" content="website" />
    <meta property="og:url" content="${canonical}" />
    <meta property="og:title" content="${title}" />
    <meta property="og:description" content="${description}" />
    <meta property="og:image" content="https://dictara.app/og-image.png" />
    <meta property="og:site_name" content="Dictara" />
    <meta property="twitter:card" content="summary_large_image" />
    <meta property="twitter:url" content="${canonical}" />
    <meta property="twitter:title" content="${title}" />
    <meta property="twitter:description" content="${description}" />
    <meta property="twitter:image" content="https://dictara.app/og-image.png" />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap" rel="stylesheet" />
    <meta name="theme-color" content="#0F0A1A" />
    ${cssLinks.join("\n    ")}
  </head>
  <body>
    <div id="root">${staticContent}
    </div>
    ${scriptTags.join("\n    ")}
  </body>
</html>`;
}

// Generate sub-route pages
for (const route of routes) {
  const routeDir = join(distDir, route.path);
  mkdirSync(routeDir, { recursive: true });
  writeFileSync(join(routeDir, "index.html"), buildPage(route));
  console.log(`Generated: ${route.path}/index.html`);
}

// Copy index.html as 404.html (SPA fallback for unknown routes)
copyFileSync(join(distDir, "index.html"), join(distDir, "404.html"));
console.log("Generated: 404.html");

console.log("SEO pages generated successfully.");
