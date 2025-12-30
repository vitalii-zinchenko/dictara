import { error as logError } from '@tauri-apps/plugin-log'
import { commands } from '@/bindings'
import { Channel } from '@tauri-apps/api/core'
import { useEffect, useRef, useState } from 'react'

interface UseAudioLevelResult {
  audioLevel: number
  smoothedLevel: number
}

export function useAudioLevel(): UseAudioLevelResult {
  const [audioLevel, setAudioLevel] = useState(0)
  const [smoothedLevel, setSmoothedLevel] = useState(0)
  const animationFrameRef = useRef<number | undefined>(undefined)

  // Smooth the audio level using requestAnimationFrame
  useEffect(() => {
    const animate = () => {
      setSmoothedLevel((current) => {
        const diff = audioLevel - current
        // Fast response to increases, slower decay
        const speed = diff > 0 ? 0.3 : 0.15
        return current + diff * speed
      })
      animationFrameRef.current = requestAnimationFrame(animate)
    }

    animationFrameRef.current = requestAnimationFrame(animate)

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current)
      }
    }
  }, [audioLevel])

  // Set up audio level channel
  useEffect(() => {
    const setupAudioLevelChannel = async () => {
      const audioLevelChannel = new Channel<number>()

      audioLevelChannel.onmessage = (level: number) => {
        setAudioLevel(level)
      }

      try {
        const result = await commands.registerAudioLevelChannel(audioLevelChannel)
        if (result.status === 'error') {
          throw new Error(result.error)
        }
      } catch (err) {
        logError(`[Popup] Failed to register audio level channel: ${err}`)
      }
    }

    setupAudioLevelChannel()
  }, [])

  return { audioLevel, smoothedLevel }
}
