import { Download, Apple, Loader2, Monitor } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useDownload } from "@/hooks/useDownload";

export function Hero() {
  const { download, isLoading, isSupported, platform, platformLabel } = useDownload();

  const getPlatformIcon = () => {
    if (platform === "mac-arm" || platform === "mac-intel") {
      return <Apple className="w-4 h-4" />;
    }
    return <Monitor className="w-4 h-4" />;
  };

  const getPlatformMessage = () => {
    if (isLoading) return "Detecting platform...";
    if (isSupported) return `Available for ${platformLabel}`;
    if (platform === "mac-intel") return "Intel Mac support coming soon";
    if (platform === "windows") return "Windows support coming soon";
    if (platform === "linux") return "Linux support coming soon";
    return "Currently available for macOS";
  };

  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden">
      {/* Background gradient orbs */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute -top-1/2 -left-1/2 w-full h-full bg-radial-purple blur-3xl" />
        <div className="absolute -bottom-1/2 -right-1/2 w-full h-full bg-radial-cyan blur-3xl" />
        <div className="absolute top-1/4 right-1/4 w-96 h-96 bg-radial-coral blur-3xl animate-pulse-slow" />
      </div>

      {/* Content */}
      <div className="relative z-10 max-w-4xl mx-auto px-6 text-center">
        {/* Big - Main headline */}
        <h1 className="text-2xl sm:text-4xl md:text-5xl font-bold text-white mb-6 whitespace-nowrap">
          Typing is slow. <span className="text-gradient-warm">Speaking isn't.</span>
        </h1>

        {/* Middle - Tagline badges */}
        <p className="text-lg sm:text-xl md:text-2xl text-white/70 font-medium mb-4 max-w-2xl mx-auto">
          Free · Bring Your Own Key · <span className="text-gradient-cool">Speech-to-Text</span>
        </p>

        {/* Small - Description */}
        <p className="text-base sm:text-lg text-white/60 mb-10 max-w-xl mx-auto leading-relaxed">
          Turn your spoken words into text — in any app, any language.
        </p>

        {/* CTA Buttons */}
        <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
          <Button
            variant="warm"
            size="xl"
            className="w-full sm:w-auto"
            onClick={download}
            disabled={isLoading}
          >
            {isLoading ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : (
              <Download className="w-5 h-5" />
            )}
            {isLoading ? "Loading..." : isSupported ? "Download" : "View Releases"}
          </Button>
        </div>

        {/* Platform indicator */}
        <div className="mt-6 flex items-center justify-center gap-2 text-white/40 text-sm">
          {getPlatformIcon()}
          <span>{getPlatformMessage()}</span>
        </div>
      </div>

      {/* Scroll indicator */}
      <button
        onClick={() => {
          document.getElementById("features")?.scrollIntoView({ behavior: "smooth" });
        }}
        className="absolute bottom-8 left-1/2 -translate-x-1/2 flex flex-col items-center gap-2 text-white/40 hover:text-white/60 transition-colors cursor-pointer"
      >
        <span className="text-xs uppercase tracking-widest">Scroll</span>
        <svg
          width="20"
          height="20"
          viewBox="0 0 20 20"
          fill="none"
          className="opacity-60"
        >
          <path
            d="M10 4v12M5 11l5 5 5-5"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </button>
    </section>
  );
}
