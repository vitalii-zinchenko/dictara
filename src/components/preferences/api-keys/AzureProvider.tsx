import { waitForPaint } from '@/utils/waitForPaint'
import { error as logError } from '@tauri-apps/plugin-log'
import { useForm } from '@tanstack/react-form'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'
import { Button } from '../../ui/button'
import { Input } from '../../ui/input'
import { Label } from '../../ui/label'
import { ProviderSection } from './ProviderSection'
import type { Provider } from './types'
import { MASKED_API_KEY_PLACEHOLDER } from './utils'
import {
  useAzureOpenAIConfig,
  useSaveAzureOpenAIConfig,
  useTestAzureOpenAIConfig,
  useDeleteAzureOpenAIConfig,
} from '@/hooks/useAzureOpenAIConfig'

interface AzureOpenAIProviderProps {
  isActive: boolean
  isExpanded: boolean
  onToggleActive: (provider: Provider) => void
  onToggleExpand: (provider: Provider) => void
}

export function AzureOpenAIProvider({
  isActive,
  isExpanded,
  onToggleActive,
  onToggleExpand,
}: AzureOpenAIProviderProps) {
  const [saveSuccess, setSaveSuccess] = useState(false)

  // TanStack Query hooks
  const { data: existingConfig, isLoading } = useAzureOpenAIConfig()
  const saveConfig = useSaveAzureOpenAIConfig()
  const testConfig = useTestAzureOpenAIConfig()
  const deleteConfig = useDeleteAzureOpenAIConfig()

  const form = useForm({
    defaultValues: {
      apiKey: '',
      endpoint: '',
    },
    validators: {
      onSubmitAsync: async ({ value }) => {
        // Workaround for TanStack Form not yielding to browser paint cycle
        // See: https://github.com/TanStack/form/issues/1967
        await waitForPaint()

        try {
          const isValid = await testConfig.mutateAsync({
            apiKey: value.apiKey,
            endpoint: value.endpoint,
          })

          if (!isValid) {
            return {
              form: 'Invalid Azure OpenAI configuration. Please check your endpoint and API key.',
              fields: {},
            }
          }

          return undefined
        } catch {
          return {
            form: 'Failed to validate configuration. Please try again.',
            fields: {},
          }
        }
      },
    },
    onSubmit: async ({ value }) => {
      setSaveSuccess(false)

      try {
        await saveConfig.mutateAsync({
          apiKey: value.apiKey,
          endpoint: value.endpoint,
        })
        setSaveSuccess(true)
        form.reset()
        // Auto-enable the provider after successful save
        if (!isActive) {
          onToggleActive('azure_open_ai')
        }
      } catch (e) {
        logError(`[AzureOpenAIProvider] Failed to save config: ${e}`)
      }
    },
  })

  const handleDelete = async () => {
    try {
      await deleteConfig.mutateAsync()
      setSaveSuccess(false)
      form.reset()
      // Deactivate the provider if it was active
      if (isActive) {
        onToggleActive('azure_open_ai')
      }
    } catch (e) {
      logError(`[AzureOpenAIProvider] Failed to delete config: ${e}`)
    }
  }

  // Derive error message from mutations
  const errorMessage = saveConfig.error?.message || deleteConfig.error?.message

  if (isLoading) {
    return (
      <ProviderSection
        provider="azure_open_ai"
        title="Azure OpenAI"
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
      provider="azure_open_ai"
      title="Azure OpenAI"
      isExpanded={isExpanded}
      isActive={isActive}
      canEnable={!!existingConfig}
      onToggleExpand={onToggleExpand}
      onToggleActive={onToggleActive}
    >
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
          <Label htmlFor="azure-endpoint">
            {existingConfig ? 'Update Azure Endpoint' : 'Azure Endpoint'}
          </Label>
          <form.Field
            name="endpoint"
            validators={{
              onChange: ({ value }) => {
                if (!value) return 'Endpoint is required'
                if (!value.startsWith('https://')) return 'Endpoint must start with https://'
                return undefined
              },
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Input
                  id="azure-endpoint"
                  type="url"
                  placeholder={existingConfig?.endpoint || 'https://your-resource.openai.azure.com'}
                  value={field.state.value}
                  onChange={(e) => {
                    field.handleChange(e.target.value)
                    setSaveSuccess(false)
                  }}
                  onBlur={field.handleBlur}
                />
                {field.state.meta.isTouched && field.state.meta.errors.length > 0 && (
                  <p className="text-sm text-destructive">{field.state.meta.errors.join(', ')}</p>
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
                <div className="flex gap-2">
                  <Input
                    id="azure-api-key"
                    type="password"
                    placeholder={existingConfig ? MASKED_API_KEY_PLACEHOLDER : 'Your Azure API key'}
                    value={field.state.value}
                    onChange={(e) => {
                      field.handleChange(e.target.value)
                      setSaveSuccess(false)
                    }}
                    onBlur={field.handleBlur}
                    className="flex-1"
                  />
                  {existingConfig && (
                    <Button
                      type="button"
                      variant="destructive"
                      onClick={handleDelete}
                      disabled={deleteConfig.isPending}
                    >
                      {deleteConfig.isPending ? 'Deleting...' : 'Delete'}
                    </Button>
                  )}
                </div>
                {field.state.meta.isTouched && field.state.meta.errors.length > 0 && (
                  <p className="text-sm text-destructive">{field.state.meta.errors.join(', ')}</p>
                )}
              </div>
            )}
          </form.Field>
        </div>

        {/* Feedback messages */}
        <form.Subscribe selector={(state) => state.errorMap}>
          {(errorMap) => (
            <>
              {errorMap.onSubmit && <p className="text-sm text-destructive">{errorMap.onSubmit}</p>}
            </>
          )}
        </form.Subscribe>
        {errorMessage && <p className="text-sm text-destructive">{errorMessage}</p>}
        {saveSuccess && <p className="text-sm text-green-600">Configuration saved successfully!</p>}

        {/* Action buttons */}
        <div className="flex gap-2">
          <form.Subscribe
            selector={(state) => [state.canSubmit, state.isSubmitting]}
            children={([canSubmit, isSubmitting]) => (
              <Button type="submit" disabled={!canSubmit}>
                {isSubmitting ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Saving...
                  </>
                ) : (
                  'Save'
                )}
              </Button>
            )}
          />
        </div>
      </form>
    </ProviderSection>
  )
}
