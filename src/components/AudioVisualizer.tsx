import { useEffect, useRef, useState } from "react";
import "./AudioVisualizer.css";

interface AudioVisualizerProps {
  /** Audio level from 0.0 (silent) to 1.0 (loudest) */
  level: number;
  /** Number of bars to display */
  barCount?: number;
  /** Minimum bar height as percentage (0-100) */
  minHeight?: number;
  /** Maximum bar height as percentage (0-100) */
  maxHeight?: number;
  /** Animation speed in milliseconds */
  animationSpeed?: number;
}

/**
 * Custom hook to manage bar volume animations
 * Each bar responds to the audio level with slight variations
 */
function useBarVolumes(level: number, barCount: number, animationSpeed: number) {
  const [barVolumes, setBarVolumes] = useState<number[]>(Array(barCount).fill(0));
  const animationFrameRef = useRef<number | undefined>(undefined);

  useEffect(() => {
    const animate = () => {
      setBarVolumes((prev) => {
        return prev.map((currentVolume, index) => {
          // Center bars respond more to audio
          const centerProximity = 1 - Math.abs(index - barCount / 2) / (barCount / 2);
          const variationFactor = 0.5 + centerProximity * 0.5;

          // Add more randomness for more distinct bar movement
          const randomVariation = 0.85 + Math.random() * 0.3;

          const targetVolume = level * variationFactor * randomVariation;
          const diff = targetVolume - currentVolume;

          // Very fast snap to target for more visible changes
          const speed = diff > 0 ? 0.6 : 0.3;

          return currentVolume + diff * speed;
        });
      });

      animationFrameRef.current = requestAnimationFrame(animate);
    };

    animationFrameRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [level, barCount, animationSpeed]);

  return barVolumes;
}

export function AudioVisualizer({
  level,
  barCount = 5,
}: AudioVisualizerProps) {
  const barVolumes = useBarVolumes(level, barCount, 50);

  return (
    <div className="audio-visualizer" role="img" aria-label="Audio level visualization">
      {barVolumes.map((volume, index) => {
        // Calculate height with larger range for more dramatic movement
        const heightPx = Math.max(4, volume * 70);

        return (
          <span
            key={index}
            className="audio-bar"
            style={{
              height: `${heightPx}px`,
              backgroundColor: volume > 0.4 ? '#6b7280' : '#9ca3af',
            }}
          />
        );
      })}
    </div>
  );
}
