import { error as logError } from '@tauri-apps/plugin-log'
import { useAppConfig } from '@/hooks/useAppConfig'
import { useSaveAppConfig } from '@/hooks/useSaveAppConfig'
import { useEffect, useRef, useState } from 'react'
import { AzureOpenAIProvider } from './AzureProvider'
import { OpenAIProvider } from './OpenAiProvider'
import type { Provider } from './types'

export function ApiKeys() {
  // Load app config using TanStack Query (type-safe via tauri-specta)
  const { data: config, isLoading } = useAppConfig()
  const saveConfig = useSaveAppConfig()

  // Local state - initialized from config on first load
  const [activeProvider, setActiveProvider] = useState<Provider>(null)
  const [expandedSection, setExpandedSection] = useState<Provider>(null)
  const isInitialized = useRef(false)

  // Sync state from config on initial load only
  useEffect(() => {
    if (config && !isInitialized.current) {
      setActiveProvider(config.activeProvider)
      setExpandedSection(config.activeProvider)
      isInitialized.current = true
    }
  }, [config])

  // Toggle section expand/collapse (visual only)
  const handleToggleExpand = (provider: Provider) => {
    setExpandedSection(expandedSection === provider ? null : provider)
  }

  // Toggle provider activation (functional)
  const handleToggleProvider = (provider: Provider) => {
    // If clicking the already-active provider, disable it
    const newProvider = activeProvider === provider ? null : provider
    const previousProvider = activeProvider

    // Update local state immediately (optimistic update)
    setActiveProvider(newProvider)

    saveConfig.mutate(
      { activeProvider: newProvider },
      {
        onError: (e) => {
          // Revert on error
          setActiveProvider(previousProvider)
          logError(`[ApiKeys] Failed to update active provider: ${e}`)
        },
      }
    )
  }

  if (isLoading) {
    return <div className="text-muted-foreground">Loading...</div>
  }

  return (
    <div className="space-y-4">
      <p className="text-sm text-muted-foreground">
        Select a transcription provider. Only one provider can be enabled at a time.
      </p>

      <OpenAIProvider
        isExpanded={expandedSection === 'open_ai'}
        isActive={activeProvider === 'open_ai'}
        onToggleExpand={handleToggleExpand}
        onToggleActive={handleToggleProvider}
      />

      <AzureOpenAIProvider
        isExpanded={expandedSection === 'azure_open_ai'}
        isActive={activeProvider === 'azure_open_ai'}
        onToggleExpand={handleToggleExpand}
        onToggleActive={handleToggleProvider}
      />

      <p className="text-xs text-muted-foreground pt-4 border-t">
        API keys are stored securely in the macOS Keychain. Configuration is saved locally.
      </p>
    </div>
  )
}
