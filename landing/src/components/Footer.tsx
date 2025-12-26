import { Github, Heart } from "lucide-react";

export function Footer() {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="relative py-12 border-t border-white/5">
      <div className="max-w-6xl mx-auto px-6">
        <div className="flex flex-col sm:flex-row items-center justify-between gap-6">
          {/* Logo and copyright */}
          <div className="flex flex-col items-center sm:items-start gap-2">
            <span className="text-xl font-bold text-gradient-cool">
              Dictara
            </span>
            <p className="text-sm text-white/40">
              &copy; {currentYear} Dictara. Open source under MIT.
            </p>
          </div>

          {/* Links */}
          <div className="flex items-center gap-6">
            <a
              href="https://github.com/vitalii-zinchenko/dictara"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 text-white/60 hover:text-white transition-colors"
            >
              <Github className="w-5 h-5" />
              <span className="text-sm">GitHub</span>
            </a>
            <a
              href="https://github.com/vitalii-zinchenko/dictara/releases"
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm text-white/60 hover:text-white transition-colors"
            >
              Releases
            </a>
          </div>
        </div>

        {/* Made with love */}
        <div className="mt-8 pt-6 border-t border-white/5 text-center">
          <p className="text-sm text-white/40 flex items-center justify-center gap-1">
            Made with <Heart className="w-4 h-4 text-warm-coral fill-warm-coral" /> for productivity
          </p>
        </div>
      </div>
    </footer>
  );
}
