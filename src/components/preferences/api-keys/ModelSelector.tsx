import { error as logError } from '@tauri-apps/plugin-log'
import {
  useDownloadModel,
  useCancelModelDownload,
  useDeleteModel,
  useUnloadModel,
} from '@/hooks/useLocalModels'
import type { ModelInfo } from '@/bindings'
import { Button } from '../../ui/button'
import { Loader2, Download, Trash2, Check, HardDrive, Cpu, X } from 'lucide-react'
import { cn, formatBytes } from '@/lib/utils'
import { ModelDownloadProgress, useDownloadProgress } from './ModelDownloadProgress'

interface ModelSelectorProps {
  models: ModelInfo[]
  selectedModel: string | null
  onSelectModel: (modelName: string) => void
  onDisableProvider: () => void
}

interface ModelCardProps {
  model: ModelInfo
  isSelected: boolean
  onSelect: () => void
  onDownload: () => void
  onCancelDownload: () => void
  onDelete: () => void
  onDisable: () => void
  isDownloadPending: boolean
  isDeletePending: boolean
  isDisablePending: boolean
}

function ModelCard({
  model,
  isSelected,
  onSelect,
  onDownload,
  onCancelDownload,
  onDelete,
  onDisable,
  isDownloadPending,
  isDeletePending,
  isDisablePending,
}: ModelCardProps) {
  const progress = useDownloadProgress(model.name)
  const isDownloading = model.isDownloading || !!progress

  return (
    <div
      className={cn(
        'border rounded-lg p-3 transition-colors',
        model.isDownloaded && 'cursor-pointer hover:bg-accent/50',
        isSelected && 'border-primary bg-accent/30'
      )}
      onClick={model.isDownloaded ? onSelect : undefined}
    >
      <div className="flex items-start justify-between gap-3">
        {/* Model Info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium">{model.displayName}</span>
            {isSelected && <Check className="h-4 w-4 text-primary" />}
            {model.isLoaded && (
              <span className="text-xs bg-green-100 text-green-800 px-1.5 py-0.5 rounded">
                Loaded
              </span>
            )}
            {model.isLoading && (
              <span className="text-xs bg-blue-100 text-blue-800 px-1.5 py-0.5 rounded flex items-center gap-1">
                <Loader2 className="h-3 w-3 animate-spin" />
                Loading...
              </span>
            )}
          </div>
          <p className="text-xs text-muted-foreground mt-0.5">{model.description}</p>
          <div className="flex items-center gap-4 mt-1 text-xs text-muted-foreground">
            <span className="flex items-center gap-1">
              <HardDrive className="h-3 w-3" />
              {formatBytes(model.sizeBytes)}
            </span>
            <span className="flex items-center gap-1">
              <Cpu className="h-3 w-3" />~{model.estimatedRamMb} MB RAM
            </span>
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2 self-end" onClick={(e) => e.stopPropagation()}>
          {isDownloading ? (
            <Button variant="outline" size="sm" onClick={onCancelDownload}>
              <X className="h-4 w-4 mr-1" />
              Cancel
            </Button>
          ) : model.isDownloaded ? (
            <>
              {model.isLoaded ? (
                <Button
                  variant="outline"
                  size="sm"
                  className="text-xs h-7"
                  onClick={onDisable}
                  disabled={isDisablePending}
                >
                  Disable
                </Button>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  className="text-xs h-7"
                  onClick={onSelect}
                  disabled={model.isLoading}
                >
                  Enable
                </Button>
              )}
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7"
                onClick={onDelete}
                disabled={isDeletePending || model.isLoading}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </>
          ) : (
            <Button variant="outline" size="sm" onClick={onDownload} disabled={isDownloadPending}>
              <Download className="h-4 w-4 mr-1" />
              Download
            </Button>
          )}
        </div>
      </div>

      {/* Download Progress */}
      {isDownloading && <ModelDownloadProgress progress={progress} />}
    </div>
  )
}

export function ModelSelector({
  models,
  selectedModel,
  onSelectModel,
  onDisableProvider,
}: ModelSelectorProps) {
  const downloadModel = useDownloadModel()
  const cancelDownload = useCancelModelDownload()
  const deleteModel = useDeleteModel()
  const unloadModel = useUnloadModel()

  const handleDownload = async (modelName: string) => {
    try {
      await downloadModel.mutateAsync(modelName)
    } catch (e) {
      logError(`Failed to download model: ${e}`)
    }
  }

  const handleCancelDownload = async (modelName: string) => {
    try {
      await cancelDownload.mutateAsync(modelName)
    } catch (e) {
      logError(`Failed to cancel download: ${e}`)
    }
  }

  const handleDelete = async (modelName: string, isLoaded: boolean) => {
    try {
      // Unload first if loaded
      if (isLoaded) {
        await unloadModel.mutateAsync()
      }
      await deleteModel.mutateAsync(modelName)
    } catch (e) {
      logError(`Failed to delete model: ${e}`)
    }
  }

  const handleDisable = async () => {
    try {
      await unloadModel.mutateAsync()
      onDisableProvider()
    } catch (e) {
      logError(`Failed to disable model: ${e}`)
    }
  }

  return (
    <div className="space-y-3">
      <h4 className="text-sm font-medium">Available Models</h4>

      <div className="space-y-2">
        {models.map((model) => (
          <ModelCard
            key={model.name}
            model={model}
            isSelected={selectedModel === model.name}
            onSelect={() => onSelectModel(model.name)}
            onDownload={() => handleDownload(model.name)}
            onCancelDownload={() => handleCancelDownload(model.name)}
            onDelete={() => handleDelete(model.name, model.isLoaded)}
            onDisable={handleDisable}
            isDownloadPending={downloadModel.isPending}
            isDeletePending={deleteModel.isPending}
            isDisablePending={unloadModel.isPending}
          />
        ))}
      </div>
    </div>
  )
}
