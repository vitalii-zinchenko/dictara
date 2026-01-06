import { useState } from 'react'
import { AlertTriangle } from 'lucide-react'
import {
  useAvailableModels,
  useLocalModelConfig,
  useSaveLocalModelConfig,
  useLoadModel,
  useUnloadModel,
} from '@/hooks/useLocalModels'
import { ProviderSection } from './ProviderSection'
import type { Provider } from './types'
import { ModelSelector } from './ModelSelector'

interface LocalProviderProps {
  isActive: boolean
  isExpanded: boolean
  onToggleActive: (provider: Provider) => void
  onToggleExpand: (provider: Provider) => void
}

export function LocalProvider({
  isActive,
  isExpanded,
  onToggleActive,
  onToggleExpand,
}: LocalProviderProps) {
  // Error state for displaying load failures
  const [loadError, setLoadError] = useState<string | null>(null)

  // Query hooks
  const { data: models, isLoading: modelsLoading } = useAvailableModels()
  const { data: config, isLoading: configLoading } = useLocalModelConfig()
  const saveConfig = useSaveLocalModelConfig()
  const loadModel = useLoadModel()
  const unloadModel = useUnloadModel()

  const isLoading = modelsLoading || configLoading

  // Check if any model is downloaded and selected (ready to use)
  const selectedModel = config?.selectedModel
  const selectedModelInfo = models?.find((m) => m.name === selectedModel)
  const canEnable = !!selectedModel && !!selectedModelInfo?.isDownloaded

  // Handle toggling the provider - unload model when disabling
  const handleToggleActive = (provider: Provider) => {
    if (isActive && provider === 'local') {
      // Disabling local provider - unload the model to free memory
      unloadModel.mutate()
    }
    onToggleActive(provider)
  }

  // Handle disabling from the model card's Disable button
  const handleDisableProvider = () => {
    if (isActive) {
      onToggleActive('local')
    }
  }

  const handleSelectModel = async (modelName: string) => {
    // Clear any previous error
    setLoadError(null)

    // Save the selection
    await saveConfig.mutateAsync(modelName)

    // Load the model into memory if it's downloaded
    const modelInfo = models?.find((m) => m.name === modelName)
    if (modelInfo?.isDownloaded) {
      try {
        await loadModel.mutateAsync(modelName)
        // Auto-enable the provider only after successful model load
        if (!isActive) {
          onToggleActive('local')
        }
      } catch (e) {
        // Model failed to load - display the error message
        const errorMessage = e instanceof Error ? e.message : String(e)
        setLoadError(errorMessage)
        console.error('Failed to load model:', e)
      }
    }
  }

  if (isLoading) {
    return (
      <ProviderSection
        provider="local"
        title="Local (Offline)"
        isExpanded={isExpanded}
        isActive={isActive}
        canEnable={false}
        onToggleExpand={onToggleExpand}
        onToggleActive={handleToggleActive}
      >
        <div className="text-muted-foreground text-sm">Loading...</div>
      </ProviderSection>
    )
  }

  return (
    <ProviderSection
      provider="local"
      title="Local (Offline)"
      isExpanded={isExpanded}
      isActive={isActive}
      canEnable={canEnable}
      onToggleExpand={onToggleExpand}
      onToggleActive={handleToggleActive}
    >
      <div className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Transcribe locally using Whisper models. No API key required, works completely offline.
        </p>

        {/* Error message display */}
        {loadError && (
          <div className="flex items-start gap-2 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-700">
            <AlertTriangle className="h-4 w-4 flex-shrink-0 mt-0.5" />
            <div>
              <p className="font-medium">Failed to load model</p>
              <p className="text-red-600 mt-0.5">{loadError}</p>
            </div>
          </div>
        )}

        <ModelSelector
          models={models ?? []}
          selectedModel={selectedModel ?? null}
          onSelectModel={handleSelectModel}
          onDisableProvider={handleDisableProvider}
        />
      </div>
    </ProviderSection>
  )
}
