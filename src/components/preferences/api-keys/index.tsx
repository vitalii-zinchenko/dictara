import { invoke } from '@tauri-apps/api/core'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Collapsible, CollapsibleContent } from '../../ui/collapsible'
import { Switch } from '../../ui/switch'
import { AzureOpenAIProvider } from './AzureProvider'
import { OpenAIProvider } from './OpenAiProvider'
import type { AppConfig, Provider } from './types'

export function ApiKeys() {
  const [activeProvider, setActiveProvider] = useState<Provider>(null)
  const [isLoading, setIsLoading] = useState(true)

  // Load app config on mount
  useEffect(() => {
    async function loadConfig() {
      try {
        const config = await invoke<AppConfig>('load_app_config')
        setActiveProvider(config.active_provider)
        console.log('[ApiKeys] Loaded config:', config)
      } catch (e) {
        console.error('[ApiKeys] Failed to load config:', e)
      } finally {
        setIsLoading(false)
      }
    }
    loadConfig()
  }, [])

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
      <Collapsible open={activeProvider === 'open_ai'}>
        <div className="border rounded-lg p-4">
          <div className="flex items-center justify-between w-full mb-4">
            <div className="flex items-center gap-2">
              {activeProvider === 'open_ai' ? (
                <ChevronDown className="h-4 w-4 text-muted-foreground" />
              ) : (
                <ChevronRight className="h-4 w-4 text-muted-foreground" />
              )}
              <h3 className="text-lg font-semibold">OpenAI</h3>
            </div>
            <Switch
              checked={activeProvider === 'open_ai'}
              onCheckedChange={() => handleToggleProvider('open_ai')}
            />
          </div>

          <CollapsibleContent>
            <OpenAIProvider />
          </CollapsibleContent>
        </div>
      </Collapsible>

      {/* Azure OpenAI Provider Section */}
      <Collapsible open={activeProvider === 'azure_open_ai'}>
        <div className="border rounded-lg p-4">
          <div className="flex items-center justify-between w-full mb-4">
            <div className="flex items-center gap-2">
              {activeProvider === 'azure_open_ai' ? (
                <ChevronDown className="h-4 w-4 text-muted-foreground" />
              ) : (
                <ChevronRight className="h-4 w-4 text-muted-foreground" />
              )}
              <h3 className="text-lg font-semibold">Azure OpenAI</h3>
            </div>
            <Switch
              checked={activeProvider === 'azure_open_ai'}
              onCheckedChange={() => handleToggleProvider('azure_open_ai')}
            />
          </div>

          <CollapsibleContent>
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
