import { useEffect, useState } from 'react'
import { error as logError } from '@tauri-apps/plugin-log'
import { events } from '@/bindings'
import { formatBytes } from '@/lib/utils'
import { Progress } from '../../ui/progress'

/** Download progress state */
interface DownloadProgressData {
  type: 'progress'
  modelName: string
  downloadedBytes: number
  totalBytes: number
  percentage: number
}

/** Verifying checksum state */
interface VerifyingData {
  type: 'verifying'
  modelName: string
}

/** Combined download state */
export type DownloadState = DownloadProgressData | VerifyingData | null

interface ModelDownloadProgressProps {
  /** Progress state passed from parent to avoid duplicate event listeners */
  progress: DownloadState
}

/**
 * Hook to track download progress for a specific model.
 * Listens to consolidated download state change events.
 */
export function useDownloadProgress(modelName: string): DownloadState {
  const [state, setState] = useState<DownloadState>(null)

  useEffect(() => {
    const unlisten = events.modelDownloadStateChanged.listen((event) => {
      if (event.payload.modelName !== modelName) return

      switch (event.payload.state) {
        case 'progress':
          setState({
            type: 'progress',
            modelName: event.payload.modelName,
            downloadedBytes: event.payload.downloadedBytes,
            totalBytes: event.payload.totalBytes,
            percentage: event.payload.percentage,
          })
          break
        case 'verifying':
          setState({
            type: 'verifying',
            modelName: event.payload.modelName,
          })
          break
        case 'complete':
          setState(null)
          break
        case 'error':
          setState(null)
          logError(`Model download error: ${event.payload.error}`)
          break
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [modelName])

  return state
}

/**
 * Component to display download progress for a model.
 * Shows a progress bar during download, then a verifying message.
 * Progress state should be passed from parent to avoid duplicate event listeners.
 */
export function ModelDownloadProgress({ progress }: ModelDownloadProgressProps) {
  if (!progress) {
    return null
  }

  if (progress.type === 'verifying') {
    return (
      <div className="mt-3 space-y-1">
        <Progress value={100} className="h-2" />
        <div className="flex justify-between text-xs text-muted-foreground">
          <span>Verifying checksum...</span>
          <span>This may take a minute</span>
        </div>
      </div>
    )
  }

  return (
    <div className="mt-3 space-y-1">
      <Progress value={progress.percentage} className="h-2" />
      <div className="flex justify-between text-xs text-muted-foreground">
        <span>
          {formatBytes(progress.downloadedBytes)} / {formatBytes(progress.totalBytes)}
        </span>
        <span>{progress.percentage.toFixed(1)}%</span>
      </div>
    </div>
  )
}
