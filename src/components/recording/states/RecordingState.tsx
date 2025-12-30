import { Square, X } from 'lucide-react'

interface RecordingStateProps {
  elapsedMs: number
  smoothedLevel: number
  onCancel: () => void
  onStop: () => void
  isCancelPending: boolean
  isStopPending: boolean
}

// Calculate inset shadow based on audio level
// Creates a white glow from the edges inward when speaking
function getInsetShadow(level: number): string {
  const spreadSize = Math.round(level * 60) // 0-60px spread
  const opacity = level * 0.3 // 0-0.3 opacity for subtle effect
  return `inset 0 0 ${spreadSize}px rgba(255, 255, 255, ${opacity})`
}

// Format milliseconds to MM:SS
function formatTime(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000)
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
}

export function RecordingState({
  elapsedMs,
  smoothedLevel,
  onCancel,
  onStop,
  isCancelPending,
  isStopPending,
}: RecordingStateProps) {
  return (
    <div
      className="flex flex-col items-center justify-center w-full h-full bg-gray-800"
      style={{ boxShadow: getInsetShadow(smoothedLevel) }}
    >
      {/* Timer Display */}
      <div className="text-gray-300 font-mono text-xs mb-2">{formatTime(elapsedMs)}</div>

      {/* Button Row */}
      <div className="flex gap-2">
        {/* Cancel Button */}
        <button
          onClick={onCancel}
          disabled={isCancelPending}
          className="w-6 h-6 aspect-square rounded-lg shrink-0 bg-gray-700 hover:bg-gray-600 flex items-center justify-center transition-colors cursor-pointer disabled:opacity-50"
        >
          <X className="w-4 h-4 text-white" strokeWidth={2.5} />
        </button>

        {/* Stop Recording Button */}
        <button
          onClick={onStop}
          disabled={isStopPending}
          className="w-6 h-6 aspect-square rounded-lg shrink-0 bg-red-500 hover:bg-red-600 flex items-center justify-center transition-colors cursor-pointer disabled:opacity-50"
        >
          <Square className="w-3 h-3 text-white" fill="white" strokeWidth={0} />
        </button>
      </div>
    </div>
  )
}
