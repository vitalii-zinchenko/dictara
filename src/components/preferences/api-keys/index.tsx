import { useAppConfig } from '@/hooks/useAppConfig'
import { invoke } from '@tauri-apps/api/core'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { Collapsible, CollapsibleContent } from '../../ui/collapsible'
import { Switch } from '../../ui/switch'
import { AzureOpenAIProvider } from './AzureProvider'
import { OpenAIProvider } from './OpenAiProvider'
import type { Provider } from './types'

export function ApiKeys() {
  // Load app config using TanStack Query (type-safe via tauri-specta)
  const { data: config, isLoading, error } = useAppConfig()
  const [activeProvider, setActiveProvider] = useState<Provider>(null)
  const [expandedSection, setExpandedSection] = useState<Provider>(null)
  const isInitialized = useRef(false)

  // Sync activeProvider state when config first loads
  useEffect(() => {
    if (config && !isInitialized.current) {
      setActiveProvider(config.active_provider)
      // Auto-expand the active provider's section on initial load
      setExpandedSection(config.active_provider)
      isInitialized.current = true
      console.log('[ApiKeys] Loaded config:', config)
    }
  }, [config])

  if (error) {
    console.error('[ApiKeys] Failed to load config:', error)
  }

  // Toggle section expand/collapse (visual only)
  const handleToggleExpand = (provider: Provider) => {
    setExpandedSection(expandedSection === provider ? null : provider)
  }

  // Toggle provider activation (functional)
  const handleToggleProvider = async (provider: Provider) => {
    console.log('[ApiKeys] Toggling provider:', provider)

    // If clicking the already-active provider, disable it
    const newProvider = activeProvider === provider ? null : provider

    try {
      await invoke('save_app_config', { activeProvider: newProvider })
      setActiveProvider(newProvider)
      console.log('[ApiKeys] Active provider updated:', newProvider)
    } catch (e) {
      console.error('[ApiKeys] Failed to update active provider:', e)
    }
  }

  if (isLoading) {
    return <div className="text-muted-foreground">Loading...</div>
  }

  return (
    <div className="space-y-4">
      <p className="text-sm text-muted-foreground">
        Select a transcription provider. Only one provider can be enabled at a time.
      </p>

      {/* OpenAI Provider Section */}
      <Collapsible open={expandedSection === 'open_ai'}>
        <div className="border rounded-lg p-4">
          <div className="flex items-center justify-between w-full">
            <div className="flex items-center gap-2">
              {expandedSection === 'open_ai' ? (
                <ChevronDown
                  className="h-4 w-4 text-muted-foreground cursor-pointer hover:text-foreground"
                  onClick={() => handleToggleExpand('open_ai')}
                />
              ) : (
                <ChevronRight
                  className="h-4 w-4 text-muted-foreground cursor-pointer hover:text-foreground"
                  onClick={() => handleToggleExpand('open_ai')}
                />
              )}
              <h3 className="text-lg font-semibold">OpenAI</h3>
            </div>
            <Switch
              checked={activeProvider === 'open_ai'}
              onCheckedChange={() => handleToggleProvider('open_ai')}
            />
          </div>

          <CollapsibleContent className="mt-3">
            <OpenAIProvider />
          </CollapsibleContent>
        </div>
      </Collapsible>

      {/* Azure OpenAI Provider Section */}
      <Collapsible open={expandedSection === 'azure_open_ai'}>
        <div className="border rounded-lg p-4">
          <div className="flex items-center justify-between w-full">
            <div className="flex items-center gap-2">
              {expandedSection === 'azure_open_ai' ? (
                <ChevronDown
                  className="h-4 w-4 text-muted-foreground cursor-pointer hover:text-foreground"
                  onClick={() => handleToggleExpand('azure_open_ai')}
                />
              ) : (
                <ChevronRight
                  className="h-4 w-4 text-muted-foreground cursor-pointer hover:text-foreground"
                  onClick={() => handleToggleExpand('azure_open_ai')}
                />
              )}
              <h3 className="text-lg font-semibold">Azure OpenAI</h3>
            </div>
            <Switch
              checked={activeProvider === 'azure_open_ai'}
              onCheckedChange={() => handleToggleProvider('azure_open_ai')}
            />
          </div>

          <CollapsibleContent className="mt-3">
            <AzureOpenAIProvider />
          </CollapsibleContent>
        </div>
      </Collapsible>

      <p className="text-xs text-muted-foreground pt-4 border-t">
        API keys are stored securely in the macOS Keychain. Configuration is saved locally.
      </p>
    </div>
  )
}
