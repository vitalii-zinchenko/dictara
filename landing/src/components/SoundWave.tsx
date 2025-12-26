import { cn } from "@/lib/utils";

interface SoundWaveProps {
  className?: string;
  barCount?: number;
}

// Deterministic pseudo-random based on index
function seededRandom(seed: number): number {
  const x = Math.sin(seed * 12.9898 + 78.233) * 43758.5453;
  return x - Math.floor(x);
}

export function SoundWave({ className, barCount = 40 }: SoundWaveProps) {
  const bars = Array.from({ length: barCount }, (_, i) => i);

  return (
    <div
      className={cn(
        "absolute inset-0 z-0 flex items-center justify-center gap-0.5 opacity-20 pointer-events-none overflow-hidden",
        className
      )}
      aria-hidden="true"
    >
      {bars.map((i) => {
        const height =
          20 +
          Math.sin((i / barCount) * Math.PI * 2) * 60 +
          seededRandom(i) * 30;
        const delay = (i * 0.05) % 1.5;
        const duration = 1 + seededRandom(i + 100) * 0.5;

        return (
          <div
            key={i}
            className="w-1 rounded-full bg-gradient-to-t from-cool-purple via-cool-blue to-cool-cyan"
            style={{
              height: `${height}px`,
              animation: `wave ${duration}s ease-in-out ${delay}s infinite`,
              opacity: 0.3 + Math.sin((i / barCount) * Math.PI) * 0.7,
            }}
          />
        );
      })}
    </div>
  );
}

export function SoundWaveSmall({ className }: { className?: string }) {
  return (
    <div className={cn("flex items-center justify-center gap-1", className)}>
      {[1, 2, 3, 4, 5].map((i) => (
        <div
          key={i}
          className={cn(
            "w-1 rounded-full bg-gradient-to-t from-warm-coral to-warm-golden",
            i === 1 && "h-3 animate-wave-1",
            i === 2 && "h-5 animate-wave-2",
            i === 3 && "h-7 animate-wave-3",
            i === 4 && "h-5 animate-wave-4",
            i === 5 && "h-3 animate-wave-5"
          )}
        />
      ))}
    </div>
  );
}
