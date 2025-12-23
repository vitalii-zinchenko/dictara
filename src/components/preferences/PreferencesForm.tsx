import { useForm } from '@tanstack/react-form'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Button } from '../ui/button'
import { Collapsible, CollapsibleContent } from '../ui/collapsible'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { Switch } from '../ui/switch'

type Provider = 'openai' | 'azure' | null

interface ProviderConfig {
  enabled_provider: Provider
  azure_endpoint: string | null
}

interface ProviderState {
  existingKey: string | null
  testResult: 'success' | 'error' | null
  saveSuccess: boolean
  errorMessage: string | null
}

function maskApiKey(key: string): string {
  if (key.length <= 12) return key
  return `${key.slice(0, 8)}...${key.slice(-4)}`
}

export default function PreferencesForm() {
  const [config, setConfig] = useState<ProviderConfig>({
    enabled_provider: null,
    azure_endpoint: null,
  })
  const [openaiState, setOpenaiState] = useState<ProviderState>({
    existingKey: null,
    testResult: null,
    saveSuccess: false,
    errorMessage: null,
  })
  const [azureState, setAzureState] = useState<ProviderState>({
    existingKey: null,
    testResult: null,
    saveSuccess: false,
    errorMessage: null,
  })
  const [isLoading, setIsLoading] = useState(true)
  const [isTesting, setIsTesting] = useState<Provider>(null)
  const [appVersion, setAppVersion] = useState<string | null>(null)

  // Load configuration and keys on mount
  useEffect(() => {
    async function loadData() {
      try {
        // Load provider config
        const loadedConfig = await invoke<ProviderConfig>('load_provider_config')
        setConfig(loadedConfig)

        // Load OpenAI key
        const openaiKey = await invoke<string | null>('load_openai_key')
        setOpenaiState((prev) => ({ ...prev, existingKey: openaiKey }))

        // Load Azure key
        const azureKey = await invoke<string | null>('load_azure_key')
        setAzureState((prev) => ({ ...prev, existingKey: azureKey }))

        console.log('[PreferencesForm] Loaded config:', loadedConfig)
      } catch (e) {
        console.error('[PreferencesForm] Failed to load data:', e)
      } finally {
        setIsLoading(false)
      }
    }
    loadData()
  }, [])

  // Load app version on mount
  useEffect(() => {
    getVersion()
      .then((v) => {
        console.log('[PreferencesForm] App version loaded:', v)
        setAppVersion(v)
      })
      .catch((e: unknown) => {
        console.error('[PreferencesForm] Failed to load app version:', e)
      })
  }, [])

  // OpenAI form
  const openaiForm = useForm({
    defaultValues: {
      apiKey: '',
    },
    onSubmit: async ({ value }) => {
      console.log('[PreferencesForm] Saving OpenAI API key...')
      setOpenaiState((prev) => ({ ...prev, errorMessage: null, saveSuccess: false }))

      try {
        await invoke('save_openai_key', { key: value.apiKey })
        console.log('[PreferencesForm] OpenAI API key saved successfully')
        setOpenaiState((prev) => ({
          ...prev,
          existingKey: value.apiKey,
          saveSuccess: true,
        }))
        openaiForm.reset()
        setOpenaiState((prev) => ({ ...prev, testResult: null }))
      } catch (e) {
        console.error('[PreferencesForm] Failed to save OpenAI API key:', e)
        setOpenaiState((prev) => ({ ...prev, errorMessage: `Failed to save: ${e}` }))
      }
    },
  })

  // Azure form
  const azureForm = useForm({
    defaultValues: {
      apiKey: '',
      endpoint: config.azure_endpoint || '',
    },
    onSubmit: async ({ value }) => {
      console.log('[PreferencesForm] Saving Azure configuration...')
      setAzureState((prev) => ({ ...prev, errorMessage: null, saveSuccess: false }))

      try {
        await invoke('save_azure_key', { key: value.apiKey })
        await invoke('save_provider_config', {
          enabledProvider: 'azure',
          azureEndpoint: value.endpoint,
        })
        console.log('[PreferencesForm] Azure configuration saved successfully')
        setAzureState((prev) => ({
          ...prev,
          existingKey: value.apiKey,
          saveSuccess: true,
        }))
        setConfig((prev) => ({ ...prev, azure_endpoint: value.endpoint }))
        azureForm.reset()
        setAzureState((prev) => ({ ...prev, testResult: null }))
      } catch (e) {
        console.error('[PreferencesForm] Failed to save Azure configuration:', e)
        setAzureState((prev) => ({ ...prev, errorMessage: `Failed to save: ${e}` }))
      }
    },
  })

  const handleTestOpenAI = async () => {
    const apiKey = openaiForm.getFieldValue('apiKey')
    if (!apiKey) return

    console.log('[PreferencesForm] Testing OpenAI API key...')
    setIsTesting('openai')
    setOpenaiState((prev) => ({ ...prev, testResult: null, errorMessage: null }))

    try {
      const isValid = await invoke<boolean>('test_openai_key', { key: apiKey })
      console.log('[PreferencesForm] OpenAI API key test result:', isValid)
      setOpenaiState((prev) => ({
        ...prev,
        testResult: isValid ? 'success' : 'error',
        errorMessage: isValid ? null : 'Invalid API key',
      }))
    } catch (e) {
      console.error('[PreferencesForm] Failed to test OpenAI API key:', e)
      setOpenaiState((prev) => ({
        ...prev,
        testResult: 'error',
        errorMessage: `Test failed: ${e}`,
      }))
    } finally {
      setIsTesting(null)
    }
  }

  const handleTestAzure = async () => {
    const apiKey = azureForm.getFieldValue('apiKey')
    const endpoint = azureForm.getFieldValue('endpoint')
    if (!apiKey || !endpoint) return

    console.log('[PreferencesForm] Testing Azure API key...')
    setIsTesting('azure')
    setAzureState((prev) => ({ ...prev, testResult: null, errorMessage: null }))

    try {
      const isValid = await invoke<boolean>('test_azure_key', {
        key: apiKey,
        endpoint,
      })
      console.log('[PreferencesForm] Azure API key test result:', isValid)
      setAzureState((prev) => ({
        ...prev,
        testResult: isValid ? 'success' : 'error',
        errorMessage: isValid ? null : 'Invalid API key or endpoint',
      }))
    } catch (e) {
      console.error('[PreferencesForm] Failed to test Azure API key:', e)
      setAzureState((prev) => ({
        ...prev,
        testResult: 'error',
        errorMessage: `Test failed: ${e}`,
      }))
    } finally {
      setIsTesting(null)
    }
  }

  const handleDeleteOpenAI = async () => {
    console.log('[PreferencesForm] Deleting OpenAI API key...')
    try {
      await invoke('delete_openai_key')
      console.log('[PreferencesForm] OpenAI API key deleted successfully')
      setOpenaiState({
        existingKey: null,
        testResult: null,
        saveSuccess: false,
        errorMessage: null,
      })
    } catch (e) {
      console.error('[PreferencesForm] Failed to delete OpenAI API key:', e)
      setOpenaiState((prev) => ({ ...prev, errorMessage: `Failed to delete: ${e}` }))
    }
  }

  const handleDeleteAzure = async () => {
    console.log('[PreferencesForm] Deleting Azure API key...')
    try {
      await invoke('delete_azure_key')
      console.log('[PreferencesForm] Azure API key deleted successfully')
      setAzureState({
        existingKey: null,
        testResult: null,
        saveSuccess: false,
        errorMessage: null,
      })
    } catch (e) {
      console.error('[PreferencesForm] Failed to delete Azure API key:', e)
      setAzureState((prev) => ({ ...prev, errorMessage: `Failed to delete: ${e}` }))
    }
  }

  const handleToggleProvider = async (provider: Provider) => {
    console.log('[PreferencesForm] Toggling provider:', provider)

    // If clicking the already-enabled provider, disable it
    const newProvider = config.enabled_provider === provider ? null : provider

    try {
      await invoke('save_provider_config', {
        enabledProvider: newProvider,
        azureEndpoint: config.azure_endpoint,
      })
      setConfig((prev) => ({ ...prev, enabled_provider: newProvider }))
      console.log('[PreferencesForm] Provider config updated:', newProvider)
    } catch (e) {
      console.error('[PreferencesForm] Failed to update provider config:', e)
    }
  }

  if (isLoading) {
    return <div className="text-muted-foreground">Loading...</div>
  }

  return (
    <>
      <div>
        <h1 className="text-2xl font-bold">Preferences</h1>
        <p className="text-muted-foreground mt-1">Configure your OpenAI API key</p>
      </div>

      <div className="space-y-4">
        <p className="text-sm text-muted-foreground mb-4">
          Select a transcription provider. Only one provider can be enabled at a time.
        </p>

        {/* OpenAI Provider Section */}
        <Collapsible open={config.enabled_provider === 'openai'}>
          <div className="border rounded-lg p-4">
            <div className="flex items-center justify-between w-full mb-4">
              <div className="flex items-center gap-2">
                {config.enabled_provider === 'openai' ? (
                  <ChevronDown className="h-4 w-4 text-muted-foreground" />
                ) : (
                  <ChevronRight className="h-4 w-4 text-muted-foreground" />
                )}
                <h3 className="text-lg font-semibold">OpenAI</h3>
              </div>
              <Switch
                checked={config.enabled_provider === 'openai'}
                onCheckedChange={() => handleToggleProvider('openai')}
              />
            </div>

            <CollapsibleContent>
              {/* Existing key display */}
              {openaiState.existingKey && (
                <div className="p-3 bg-muted rounded-lg mb-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-sm text-muted-foreground">Current API Key</p>
                      <p className="font-mono text-sm">{maskApiKey(openaiState.existingKey)}</p>
                    </div>
                    <Button variant="destructive" size="sm" onClick={handleDeleteOpenAI}>
                      Delete
                    </Button>
                  </div>
                </div>
              )}

              {/* OpenAI Form */}
              <form
                onSubmit={(e) => {
                  e.preventDefault()
                  e.stopPropagation()
                  openaiForm.handleSubmit()
                }}
                className="space-y-4"
              >
                <div className="space-y-2">
                  <Label htmlFor="openai-api-key">
                    {openaiState.existingKey ? 'Update API Key' : 'OpenAI API Key'}
                  </Label>
                  <openaiForm.Field
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
                            setOpenaiState((prev) => ({
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
                  </openaiForm.Field>
                </div>

                {/* Feedback messages */}
                {openaiState.errorMessage && (
                  <p className="text-sm text-destructive">{openaiState.errorMessage}</p>
                )}
                {openaiState.testResult === 'success' && (
                  <p className="text-sm text-green-600">API key is valid!</p>
                )}
                {openaiState.saveSuccess && (
                  <p className="text-sm text-green-600">API key saved successfully!</p>
                )}

                {/* Action buttons */}
                <div className="flex gap-2">
                  <openaiForm.Subscribe
                    selector={(state) => ({
                      canSubmit: state.canSubmit,
                      isSubmitting: state.isSubmitting,
                      apiKey: state.values.apiKey,
                    })}
                  >
                    {({ canSubmit, isSubmitting, apiKey }) => (
                      <>
                        <Button
                          type="button"
                          variant="outline"
                          onClick={handleTestOpenAI}
                          disabled={!apiKey || isTesting === 'openai' || !canSubmit}
                        >
                          {isTesting === 'openai' ? 'Testing...' : 'Test Key'}
                        </Button>
                        <Button
                          type="submit"
                          disabled={
                            !canSubmit || isSubmitting || openaiState.testResult !== 'success'
                          }
                        >
                          {isSubmitting ? 'Saving...' : 'Save'}
                        </Button>
                      </>
                    )}
                  </openaiForm.Subscribe>
                </div>
              </form>
            </CollapsibleContent>
          </div>
        </Collapsible>

        {/* Azure OpenAI Provider Section */}
        <Collapsible open={config.enabled_provider === 'azure'}>
          <div className="border rounded-lg p-4">
            <div className="flex items-center justify-between w-full mb-4">
              <div className="flex items-center gap-2">
                {config.enabled_provider === 'azure' ? (
                  <ChevronDown className="h-4 w-4 text-muted-foreground" />
                ) : (
                  <ChevronRight className="h-4 w-4 text-muted-foreground" />
                )}
                <h3 className="text-lg font-semibold">Azure OpenAI</h3>
              </div>
              <Switch
                checked={config.enabled_provider === 'azure'}
                onCheckedChange={() => handleToggleProvider('azure')}
              />
            </div>

            <CollapsibleContent>
              {/* Existing key display */}
              {azureState.existingKey && (
                <div className="p-3 bg-muted rounded-lg mb-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-sm text-muted-foreground">Current API Key</p>
                      <p className="font-mono text-sm">{maskApiKey(azureState.existingKey)}</p>
                    </div>
                    <Button variant="destructive" size="sm" onClick={handleDeleteAzure}>
                      Delete
                    </Button>
                  </div>
                </div>
              )}

              {config.azure_endpoint && (
                <div className="p-3 bg-muted rounded-lg mb-4">
                  <p className="text-sm text-muted-foreground">Current Endpoint</p>
                  <p className="font-mono text-sm break-all">{config.azure_endpoint}</p>
                </div>
              )}

              {/* Azure Form */}
              <form
                onSubmit={(e) => {
                  e.preventDefault()
                  e.stopPropagation()
                  azureForm.handleSubmit()
                }}
                className="space-y-4"
              >
                <div className="space-y-2">
                  <Label htmlFor="azure-endpoint">Azure Endpoint</Label>
                  <azureForm.Field
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
                            setAzureState((prev) => ({
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
                  </azureForm.Field>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="azure-api-key">
                    {azureState.existingKey ? 'Update API Key' : 'Azure API Key'}
                  </Label>
                  <azureForm.Field
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
                            setAzureState((prev) => ({
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
                  </azureForm.Field>
                </div>

                {/* Feedback messages */}
                {azureState.errorMessage && (
                  <p className="text-sm text-destructive">{azureState.errorMessage}</p>
                )}
                {azureState.testResult === 'success' && (
                  <p className="text-sm text-green-600">Configuration is valid!</p>
                )}
                {azureState.saveSuccess && (
                  <p className="text-sm text-green-600">Configuration saved successfully!</p>
                )}

                {/* Action buttons */}
                <div className="flex gap-2">
                  <azureForm.Subscribe
                    selector={(state) => ({
                      canSubmit: state.canSubmit,
                      isSubmitting: state.isSubmitting,
                      apiKey: state.values.apiKey,
                      endpoint: state.values.endpoint,
                    })}
                  >
                    {({ canSubmit, isSubmitting, apiKey, endpoint }) => (
                      <>
                        <Button
                          type="button"
                          variant="outline"
                          onClick={handleTestAzure}
                          disabled={!apiKey || !endpoint || isTesting === 'azure' || !canSubmit}
                        >
                          {isTesting === 'azure' ? 'Testing...' : 'Test Configuration'}
                        </Button>
                        <Button
                          type="submit"
                          disabled={
                            !canSubmit || isSubmitting || azureState.testResult !== 'success'
                          }
                        >
                          {isSubmitting ? 'Saving...' : 'Save'}
                        </Button>
                      </>
                    )}
                  </azureForm.Subscribe>
                </div>
              </form>
            </CollapsibleContent>
          </div>
        </Collapsible>

        <div className="space-y-2 pt-4">
          <p className="text-xs text-muted-foreground">
            API keys are stored securely in the macOS Keychain. Configuration is saved locally.
          </p>
        </div>
      </div>
    </>
  )
}
