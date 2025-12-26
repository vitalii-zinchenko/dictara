import { Github, Download } from "lucide-react";
import { Button } from "@/components/ui/button";

export function Header() {
  return (
    <header className="fixed top-0 left-0 right-0 z-50">
      <div className="mx-auto max-w-6xl px-6 py-4">
        <nav className="flex items-center justify-between rounded-2xl border border-white/10 bg-background/80 backdrop-blur-xl px-6 py-3">
          {/* Logo */}
          <a href="/" className="flex items-center gap-2 group">
            <span className="text-xl font-bold text-gradient-cool">Dictara</span>
          </a>

          {/* Right side actions */}
          <div className="flex items-center gap-4">
            <a
              href="https://github.com/vitalii-zinchenko/dictara"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 text-white/60 hover:text-white transition-colors"
            >
              <Github className="w-5 h-5" />
              <span className="hidden sm:inline text-sm">GitHub</span>
            </a>
            <Button
              variant="warm"
              size="sm"
              onClick={() =>
                window.open(
                  "https://github.com/vitalii-zinchenko/dictara/releases",
                  "_blank"
                )
              }
            >
              <Download className="w-4 h-4" />
              <span className="hidden sm:inline">Download</span>
            </Button>
          </div>
        </nav>
      </div>
    </header>
  );
}
