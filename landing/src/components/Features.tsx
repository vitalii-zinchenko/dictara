import { Zap, Globe, Shield, Cpu, Keyboard, Copy } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";

const features = [
  {
    icon: Zap,
    title: "Lightning Fast",
    description:
      "Get your transcriptions in seconds. No waiting, no loading screens—just instant results.",
    gradient: "from-warm-golden to-warm-orange",
  },
  {
    icon: Globe,
    title: "90+ Languages",
    description:
      "Speak in your native language. Whisper understands and transcribes over 90 languages accurately.",
    gradient: "from-cool-cyan to-cool-blue",
  },
  {
    icon: Shield,
    title: "Privacy First",
    description:
      "Your audio is recorded locally first. API keys stored securely in your system keychain.",
    gradient: "from-cool-purple to-cool-blue",
  },
  {
    icon: Cpu,
    title: "Powered by Whisper",
    description:
      "Leveraging OpenAI's state-of-the-art Whisper model for industry-leading accuracy.",
    gradient: "from-warm-coral to-warm-orange",
  },
  {
    icon: Keyboard,
    title: "Simple Workflow",
    description:
      "Just press the FN key to start recording, release to transcribe. No complicated menus.",
    gradient: "from-cool-blue to-cool-cyan",
  },
  {
    icon: Copy,
    title: "Auto Copy",
    description:
      "Transcribed text is automatically copied to your clipboard. Ready to paste anywhere.",
    gradient: "from-warm-orange to-warm-golden",
  },
];

export function Features() {
  return (
    <section id="features" className="relative py-24 sm:py-32">
      {/* Background accents */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-0 left-1/4 w-96 h-96 bg-radial-purple-soft blur-3xl" />
        <div className="absolute bottom-0 right-1/4 w-96 h-96 bg-radial-coral-soft blur-3xl" />
      </div>

      <div className="relative z-10 max-w-6xl mx-auto px-6">
        {/* Section header */}
        <div className="text-center mb-16">
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold mb-4">
            Everything you need to{" "}
            <span className="text-gradient-warm">dictate</span>
          </h2>
          <p className="text-lg text-white/60 max-w-2xl mx-auto">
            A minimal, focused tool that does one thing exceptionally well—turns
            your voice into text.
          </p>
        </div>

        {/* Features grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, index) => (
            <Card
              key={index}
              className="group hover:border-white/20 transition-all duration-300 hover:-translate-y-1"
            >
              <CardContent className="p-6">
                {/* Icon */}
                <div
                  className={`inline-flex items-center justify-center w-12 h-12 rounded-xl bg-linear-to-br ${feature.gradient} mb-4 group-hover:scale-110 transition-transform duration-300`}
                >
                  <feature.icon className="w-6 h-6 text-white" />
                </div>

                {/* Title */}
                <h3 className="text-lg font-semibold mb-2">{feature.title}</h3>

                {/* Description */}
                <p className="text-white/60 text-sm leading-relaxed">
                  {feature.description}
                </p>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </section>
  );
}
