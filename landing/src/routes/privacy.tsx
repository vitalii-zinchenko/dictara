import { createFileRoute, Link } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";

export const Route = createFileRoute("/privacy")({
  component: PrivacyPage,
});

function PrivacyPage() {
  return (
    <div className="min-h-screen">
      {/* Header */}
      <header className="relative py-6 border-b border-white/5">
        <div className="max-w-4xl mx-auto px-6">
          <Link
            to="/"
            className="inline-flex items-center gap-2 text-white/60 hover:text-white transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            <span className="text-xl font-bold text-gradient-cool">
              Dictara
            </span>
          </Link>
        </div>
      </header>

      {/* Content */}
      <main className="max-w-4xl mx-auto px-6 py-16">
        <h1 className="text-4xl font-bold text-white mb-2">Privacy Policy</h1>
        <p className="text-white/40 mb-12">Last updated: December 2025</p>

        <div className="space-y-10 text-white/70 leading-relaxed">
          <div className="p-6 rounded-xl border border-cool-cyan/20 bg-cool-cyan/5">
            <p className="text-white/90 font-medium">
              Dictara's developers never receive your audio, transcriptions, or
              API keys. All data either stays on your device or flows directly
              between the app and the transcription provider you choose — we are
              never in the middle.
            </p>
          </div>

          <p>
            Dictara is committed to protecting your privacy. This policy
            explains how we handle your data.
          </p>

          <section>
            <h2 className="text-xl font-semibold text-white mb-4">
              1. Information We Collect
            </h2>
            <p className="mb-4">
              Dictara is designed with privacy in mind. Here's what we handle:
            </p>
            <ul className="list-disc list-inside space-y-2 ml-4">
              <li>
                <strong className="text-white/90">Audio Recordings:</strong>{" "}
                Temporarily stored on your device only during transcription.
                Audio is sent directly from the app to your chosen provider
                (OpenAI or Azure) for processing — it never passes through our
                servers. After transcription, audio is deleted from your device.
              </li>
              <li>
                <strong className="text-white/90">API Keys:</strong> Stored
                securely in your system's keychain (macOS Keychain). We never
                have access to your API keys.
              </li>
              <li>
                <strong className="text-white/90">Settings:</strong> Your
                preferences are stored locally on your device.
              </li>
            </ul>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-4">
              2. What We Don't Collect
            </h2>
            <p className="mb-4">
              Dictara does <strong className="text-white/90">not</strong>{" "}
              collect:
            </p>
            <ul className="list-disc list-inside space-y-2 ml-4">
              <li>Analytics or telemetry data</li>
              <li>Usage statistics or behavioral data</li>
              <li>Personal information or account data</li>
              <li>Your transcribed text or audio recordings</li>
              <li>Any data at all — Dictara has no servers to collect data</li>
            </ul>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-4">
              3. Third-Party Services
            </h2>
            <p className="mb-4">
              When you use cloud transcription, audio is sent directly from your
              device to the provider you choose. Dictara does not operate any
              servers — there is no intermediary between you and the provider:
            </p>
            <ul className="list-disc list-inside space-y-2 ml-4">
              <li>
                <strong className="text-white/90">OpenAI:</strong> Subject to{" "}
                <a
                  href="https://openai.com/policies/privacy-policy"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-cool-cyan hover:underline"
                >
                  OpenAI's Privacy Policy
                </a>
              </li>
              <li>
                <strong className="text-white/90">Azure OpenAI:</strong> Subject
                to{" "}
                <a
                  href="https://privacy.microsoft.com/en-us/privacystatement"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-cool-cyan hover:underline"
                >
                  Microsoft's Privacy Statement
                </a>
              </li>
            </ul>
            <p className="mt-4">
              You provide your own API keys and are responsible for your usage
              of these services.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-4">
              4. Data Security
            </h2>
            <p>
              Your API keys are stored using your operating system's secure
              keychain. Audio files are temporary and automatically deleted
              after successful transcription. All communication with
              transcription providers uses HTTPS encryption.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-4">
              5. Your Control
            </h2>
            <p>You have full control over your data:</p>
            <ul className="list-disc list-inside space-y-2 ml-4 mt-4">
              <li>Remove your API keys at any time through the app settings</li>
              <li>
                Uninstall the app to remove all local data and cached files
              </li>
              <li>
                The app is open source — you can audit exactly what it does
              </li>
            </ul>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-4">
              6. Children's Privacy
            </h2>
            <p>
              Dictara is not intended for children under 13. We do not knowingly
              collect any information from children.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-4">
              7. Changes to This Policy
            </h2>
            <p>
              We may update this privacy policy from time to time. Any changes
              will be reflected on this page with an updated date.
            </p>
          </section>

          <section>
            <h2 className="text-xl font-semibold text-white mb-4">
              8. Contact
            </h2>
            <p>
              If you have questions about this policy, please open an issue on
              our{" "}
              <a
                href="https://github.com/vitalii-zinchenko/dictara/issues"
                target="_blank"
                rel="noopener noreferrer"
                className="text-cool-cyan hover:underline"
              >
                GitHub repository
              </a>
              .
            </p>
          </section>
        </div>
      </main>

      {/* Footer */}
      <footer className="py-8 border-t border-white/5">
        <div className="max-w-4xl mx-auto px-6 text-center text-white/40 text-sm">
          © {new Date().getFullYear()} Dictara. Open source under MIT.
        </div>
      </footer>
    </div>
  );
}
