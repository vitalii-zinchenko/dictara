import { useForm } from '@tanstack/react-form'
import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'
import { Button } from '../../ui/button'
import { Input } from '../../ui/input'
import { Label } from '../../ui/label'
import { ProviderSection } from './ProviderSection'
import type { OpenAIConfig, Provider, ProviderFormState } from './types'
import { maskApiKey } from './utils'

interface OpenAIProviderProps {
  isActive: boolean
  isExpanded: boolean
  onToggleActive: (provider: Provider) => void
  onToggleExpand: (provider: Provider) => void
}

export function OpenAIProvider({
  isActive,
  isExpanded,
  onToggleActive,
  onToggleExpand,
}: OpenAIProviderProps) {
  const [existingConfig, setExistingConfig] = useState<OpenAIConfig | null>(null)
  const [state, setState] = useState<ProviderFormState>({
    testResult: null,
    saveSuccess: false,
    errorMessage: null,
    isTesting: false,
    isLoading: true,
  })

  // Load config on mount
  useEffect(() => {
    async function loadConfig() {
      try {
        const config = await invoke<OpenAIConfig | null>('load_openai_config')
        setExistingConfig(config)
        console.log('[OpenAIProvider] Loaded config:', config ? 'exists' : 'none')
      } catch (e) {
        console.error('[OpenAIProvider] Failed to load config:', e)
      } finally {
        setState((prev) => ({ ...prev, isLoading: false }))
      }
    }
    loadConfig()
  }, [])

  const form = useForm({
    defaultValues: {
      apiKey: '',
    },
    onSubmit: async ({ value }) => {
      console.log('[OpenAIProvider] Saving config...')
      setState((prev) => ({ ...prev, errorMessage: null, saveSuccess: false }))

      try {
        await invoke('save_openai_config', { apiKey: value.apiKey })
        console.log('[OpenAIProvider] Config saved successfully')
        setExistingConfig({ api_key: value.apiKey })
        setState((prev) => ({ ...prev, saveSuccess: true, testResult: null }))
        form.reset()
      } catch (e) {
        console.error('[OpenAIProvider] Failed to save config:', e)
        setState((prev) => ({ ...prev, errorMessage: `Failed to save: ${e}` }))
      }
    },
  })

  const handleTest = async () => {
    const apiKey = form.getFieldValue('apiKey')
    if (!apiKey) return

    console.log('[OpenAIProvider] Testing config...')
    setState((prev) => ({ ...prev, isTesting: true, testResult: null, errorMessage: null }))

    try {
      const isValid = await invoke<boolean>('test_openai_config', { apiKey })
      console.log('[OpenAIProvider] Test result:', isValid)
      setState((prev) => ({
        ...prev,
        testResult: isValid ? 'success' : 'error',
        errorMessage: isValid ? null : 'Invalid API key',
        isTesting: false,
      }))
    } catch (e) {
      console.error('[OpenAIProvider] Failed to test config:', e)
      setState((prev) => ({
        ...prev,
        testResult: 'error',
        errorMessage: `Test failed: ${e}`,
        isTesting: false,
      }))
    }
  }

  const handleDelete = async () => {
    console.log('[OpenAIProvider] Deleting config...')
    try {
      await invoke('delete_openai_config')
      console.log('[OpenAIProvider] Config deleted successfully')
      setExistingConfig(null)
      setState({
        testResult: null,
        saveSuccess: false,
        errorMessage: null,
        isTesting: false,
        isLoading: false,
      })
    } catch (e) {
      console.error('[OpenAIProvider] Failed to delete config:', e)
      setState((prev) => ({ ...prev, errorMessage: `Failed to delete: ${e}` }))
    }
  }

  if (state.isLoading) {
    return (
      <ProviderSection
        provider="open_ai"
        title="OpenAI"
        isExpanded={isExpanded}
        isActive={isActive}
        canEnable={false}
        onToggleExpand={onToggleExpand}
        onToggleActive={onToggleActive}
      >
        <div className="text-muted-foreground text-sm">Loading...</div>
      </ProviderSection>
    )
  }

  return (
    <ProviderSection
      provider="open_ai"
      title="OpenAI"
      isExpanded={isExpanded}
      isActive={isActive}
      canEnable={!!existingConfig}
      onToggleExpand={onToggleExpand}
      onToggleActive={onToggleActive}
    >
      {/* Existing config display */}
      {existingConfig && (
        <div className="p-3 bg-muted rounded-lg mb-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">Current API Key</p>
              <p className="font-mono text-sm">{maskApiKey(existingConfig.api_key)}</p>
            </div>
            <Button variant="destructive" size="sm" onClick={handleDelete}>
              Delete
            </Button>
          </div>
        </div>
      )}

      {/* Form */}
      <form
        onSubmit={(e) => {
          e.preventDefault()
          e.stopPropagation()
          form.handleSubmit()
        }}
        className="space-y-4"
      >
        <div className="space-y-2">
          <Label htmlFor="openai-api-key">
            {existingConfig ? 'Update API Key' : 'OpenAI API Key'}
          </Label>
          <form.Field
            name="apiKey"
            validators={{
              onChange: ({ value }) => {
                if (!value) return 'API key is required'
                if (value.length < 20) return 'API key is too short'
                if (!value.startsWith('sk-')) return 'API key should start with sk-'
                return undefined
              },
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Input
                  id="openai-api-key"
                  type="password"
                  placeholder="sk-..."
                  value={field.state.value}
                  onChange={(e) => {
                    field.handleChange(e.target.value)
                    setState((prev) => ({
                      ...prev,
                      testResult: null,
                      saveSuccess: false,
                    }))
                  }}
                  onBlur={field.handleBlur}
                />
                {field.state.meta.isTouched && field.state.meta.errors.length > 0 && (
                  <p className="text-sm text-destructive">
                    {field.state.meta.errors.join(', ')}
                  </p>
                )}
              </div>
            )}
          </form.Field>
        </div>

        {/* Feedback messages */}
        {state.errorMessage && (
          <p className="text-sm text-destructive">{state.errorMessage}</p>
        )}
        {state.testResult === 'success' && (
          <p className="text-sm text-green-600">API key is valid!</p>
        )}
        {state.saveSuccess && (
          <p className="text-sm text-green-600">Configuration saved successfully!</p>
        )}

        {/* Action buttons */}
        <div className="flex gap-2">
          <form.Subscribe
            selector={(formState) => ({
              canSubmit: formState.canSubmit,
              isSubmitting: formState.isSubmitting,
              apiKey: formState.values.apiKey,
            })}
          >
            {({ canSubmit, isSubmitting, apiKey }) => (
              <>
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleTest}
                  disabled={!apiKey || state.isTesting || !canSubmit}
                >
                  {state.isTesting ? 'Testing...' : 'Test Key'}
                </Button>
                <Button
                  type="submit"
                  disabled={!canSubmit || isSubmitting || state.testResult !== 'success'}
                >
                  {isSubmitting ? 'Saving...' : 'Save'}
                </Button>
              </>
            )}
          </form.Subscribe>
        </div>
      </form>
    </ProviderSection>
  )
}
