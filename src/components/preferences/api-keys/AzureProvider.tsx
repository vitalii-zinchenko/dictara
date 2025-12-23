import { useForm } from '@tanstack/react-form'
import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'
import { Button } from '../../ui/button'
import { Input } from '../../ui/input'
import { Label } from '../../ui/label'
import type { AzureOpenAIConfig, ProviderFormState } from './types'
import { maskApiKey } from './utils'

export function AzureOpenAIProvider() {
  const [existingConfig, setExistingConfig] = useState<AzureOpenAIConfig | null>(null)
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
        const config = await invoke<AzureOpenAIConfig | null>('load_azure_openai_config')
        setExistingConfig(config)
        console.log('[AzureOpenAIProvider] Loaded config:', config ? 'exists' : 'none')
      } catch (e) {
        console.error('[AzureOpenAIProvider] Failed to load config:', e)
      } finally {
        setState((prev) => ({ ...prev, isLoading: false }))
      }
    }
    loadConfig()
  }, [])

  const form = useForm({
    defaultValues: {
      apiKey: '',
      endpoint: '',
    },
    onSubmit: async ({ value }) => {
      console.log('[AzureOpenAIProvider] Saving config...')
      setState((prev) => ({ ...prev, errorMessage: null, saveSuccess: false }))

      try {
        await invoke('save_azure_openai_config', {
          apiKey: value.apiKey,
          endpoint: value.endpoint,
        })
        console.log('[AzureOpenAIProvider] Config saved successfully')
        setExistingConfig({ api_key: value.apiKey, endpoint: value.endpoint })
        setState((prev) => ({ ...prev, saveSuccess: true, testResult: null }))
        form.reset()
      } catch (e) {
        console.error('[AzureOpenAIProvider] Failed to save config:', e)
        setState((prev) => ({ ...prev, errorMessage: `Failed to save: ${e}` }))
      }
    },
  })

  const handleTest = async () => {
    const apiKey = form.getFieldValue('apiKey')
    const endpoint = form.getFieldValue('endpoint')
    if (!apiKey || !endpoint) return

    console.log('[AzureOpenAIProvider] Testing config...')
    setState((prev) => ({ ...prev, isTesting: true, testResult: null, errorMessage: null }))

    try {
      const isValid = await invoke<boolean>('test_azure_openai_config', {
        apiKey,
        endpoint,
      })
      console.log('[AzureOpenAIProvider] Test result:', isValid)
      setState((prev) => ({
        ...prev,
        testResult: isValid ? 'success' : 'error',
        errorMessage: isValid ? null : 'Invalid API key or endpoint',
        isTesting: false,
      }))
    } catch (e) {
      console.error('[AzureOpenAIProvider] Failed to test config:', e)
      setState((prev) => ({
        ...prev,
        testResult: 'error',
        errorMessage: `Test failed: ${e}`,
        isTesting: false,
      }))
    }
  }

  const handleDelete = async () => {
    console.log('[AzureOpenAIProvider] Deleting config...')
    try {
      await invoke('delete_azure_openai_config')
      console.log('[AzureOpenAIProvider] Config deleted successfully')
      setExistingConfig(null)
      setState({
        testResult: null,
        saveSuccess: false,
        errorMessage: null,
        isTesting: false,
        isLoading: false,
      })
    } catch (e) {
      console.error('[AzureOpenAIProvider] Failed to delete config:', e)
      setState((prev) => ({ ...prev, errorMessage: `Failed to delete: ${e}` }))
    }
  }

  if (state.isLoading) {
    return <div className="text-muted-foreground text-sm">Loading...</div>
  }

  return (
    <>
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

      {existingConfig?.endpoint && (
        <div className="p-3 bg-muted rounded-lg mb-4">
          <p className="text-sm text-muted-foreground">Current Endpoint</p>
          <p className="font-mono text-sm break-all">{existingConfig.endpoint}</p>
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
          <Label htmlFor="azure-endpoint">Azure Endpoint</Label>
          <form.Field
            name="endpoint"
            validators={{
              onChange: ({ value }) => {
                if (!value) return 'Endpoint is required'
                if (!value.startsWith('https://'))
                  return 'Endpoint must start with https://'
                return undefined
              },
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Input
                  id="azure-endpoint"
                  type="url"
                  placeholder="https://your-resource.openai.azure.com"
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

        <div className="space-y-2">
          <Label htmlFor="azure-api-key">
            {existingConfig ? 'Update API Key' : 'Azure API Key'}
          </Label>
          <form.Field
            name="apiKey"
            validators={{
              onChange: ({ value }) => {
                if (!value) return 'API key is required'
                if (value.length < 20) return 'API key is too short'
                return undefined
              },
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Input
                  id="azure-api-key"
                  type="password"
                  placeholder="Your Azure API key"
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
          <p className="text-sm text-green-600">Configuration is valid!</p>
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
              endpoint: formState.values.endpoint,
            })}
          >
            {({ canSubmit, isSubmitting, apiKey, endpoint }) => (
              <>
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleTest}
                  disabled={!apiKey || !endpoint || state.isTesting || !canSubmit}
                >
                  {state.isTesting ? 'Testing...' : 'Test Configuration'}
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
    </>
  )
}
