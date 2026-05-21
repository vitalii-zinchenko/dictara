import {
  useDeleteCustomEndpointConfig,
  useCustomEndpointConfig,
  useSaveCustomEndpointConfig,
  useTestCustomEndpointConfig,
} from '@/hooks/useCustomEndpointConfig'
import { waitForPaint } from '@/utils/waitForPaint'
import { useForm } from '@tanstack/react-form'
import { error as logError } from '@tauri-apps/plugin-log'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'
import { Button } from '../../ui/button'
import { Input } from '../../ui/input'
import { Label } from '../../ui/label'
import { ProviderSection } from './ProviderSection'
import type { Provider } from './types'
import { MASKED_API_KEY_PLACEHOLDER } from './utils'

interface CustomEndpointProviderProps {
  isActive: boolean
  isExpanded: boolean
  onToggleActive: (provider: Provider) => void
  onToggleExpand: (provider: Provider) => void
}

export function CustomEndpointProvider({
  isActive,
  isExpanded,
  onToggleActive,
  onToggleExpand,
}: CustomEndpointProviderProps) {
  const [saveSuccess, setSaveSuccess] = useState(false)

  // TanStack Query hooks
  const { data: existingConfig, isLoading } = useCustomEndpointConfig()
  const saveConfig = useSaveCustomEndpointConfig()
  const testConfig = useTestCustomEndpointConfig()
  const deleteConfig = useDeleteCustomEndpointConfig()

  const form = useForm({
    defaultValues: {
      baseUrl: existingConfig?.baseUrl || '',
      apiKey: existingConfig?.hasApiKey ? MASKED_API_KEY_PLACEHOLDER : '',
      model: existingConfig?.model || '',
    },
    validators: {
      onSubmitAsync: async ({ value }) => {
        await waitForPaint()

        try {
          const isValid = await testConfig.mutateAsync({
            apiKey: value.apiKey ? value.apiKey : null,
            baseUrl: value.baseUrl,
            model: value.model,
          })

          if (!isValid) {
            return {
              form: 'Invalid Custom Endpoint configuration. Please check endpoint status or credentials.',
              fields: {},
            }
          }

          return undefined
        } catch (e) {
          logError(`[CustomEndpointProvider] onSubmitAsync caught error: ${e}`)
          return {
            form: 'Failed to validate Custom Endpoint. Make sure the server is running and accessible.',
            fields: {},
          }
        }
      },
    },
    onSubmit: async ({ value }) => {
      setSaveSuccess(false)

      try {
        await saveConfig.mutateAsync({
          apiKey: value.apiKey ? value.apiKey : null,
          baseUrl: value.baseUrl,
          model: value.model,
        })
        setSaveSuccess(true)
        form.reset()
        // Auto-enable the provider after successful save
        if (!isActive) {
          onToggleActive('custom_endpoint')
        }
      } catch (e) {
        logError(`[CustomEndpointProvider] Failed to save config: ${e}`)
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
        onToggleActive('custom_endpoint')
      }
    } catch (e) {
      logError(`[CustomEndpointProvider] Failed to delete config: ${e}`)
    }
  }

  // Derive error message from mutations
  const errorMessage = saveConfig.error?.message || deleteConfig.error?.message

  if (isLoading) {
    return (
      <ProviderSection
        provider="custom_endpoint"
        title="Custom Endpoint"
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
      provider="custom_endpoint"
      title="Custom Endpoint"
      isExpanded={isExpanded}
      isActive={isActive}
      canEnable={!!existingConfig}
      onToggleExpand={onToggleExpand}
      onToggleActive={onToggleActive}
    >
      <form
        onSubmit={(e) => {
          e.preventDefault()
          e.stopPropagation()
          form.handleSubmit()
        }}
        className="space-y-4"
      >
        <p className="text-sm text-muted-foreground">
          Integrate with any OpenAI-compatible Speech-to-Text endpoint (e.g. LM Studio, Groq, local
          server).
        </p>

        <div className="space-y-2">
          <Label htmlFor="custom-base-url">Base URL</Label>
          <form.Field
            name="baseUrl"
            validators={{
              onChange: ({ value }) => {
                if (!value) return 'Base URL is required'
                if (!value.startsWith('http://') && !value.startsWith('https://')) {
                  return 'URL must start with http:// or https://'
                }
                return undefined
              },
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Input
                  id="custom-base-url"
                  type="url"
                  placeholder="e.g. http://localhost:1234/v1"
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
          <Label htmlFor="custom-api-key">
            {existingConfig?.hasApiKey ? 'Update API Key (Optional)' : 'API Key (Optional)'}
          </Label>
          <form.Field name="apiKey">
            {(field) => (
              <div className="space-y-1">
                <div className="flex gap-2">
                  <Input
                    id="custom-api-key"
                    type="password"
                    placeholder={
                      existingConfig?.hasApiKey
                        ? MASKED_API_KEY_PLACEHOLDER
                        : 'Leave blank if not required'
                    }
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
              </div>
            )}
          </form.Field>
        </div>

        <div className="space-y-2">
          <Label htmlFor="custom-model">Model</Label>
          <form.Field
            name="model"
            validators={{
              onChange: ({ value }) => {
                if (!value) return 'Model is required'
                return undefined
              },
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Input
                  id="custom-model"
                  type="text"
                  placeholder="e.g. whisper-1"
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

        {/* Feedback messages */}
        <form.Subscribe selector={(state) => state.errorMap}>
          {(errorMap) => (
            <>
              {errorMap.onSubmit && <p className="text-sm text-destructive">{errorMap.onSubmit}</p>}
            </>
          )}
        </form.Subscribe>
        {errorMessage && <p className="text-sm text-destructive">{errorMessage}</p>}
        {saveSuccess && (
          <p className="text-sm text-green-600 font-medium">Configuration saved successfully!</p>
        )}

        {/* Action buttons */}
        <div className="flex gap-2">
          <form.Subscribe
            selector={(state) => [state.canSubmit, state.isSubmitting]}
            children={([canSubmit, isSubmitting]) => (
              <Button type="submit" disabled={!canSubmit}>
                {isSubmitting ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin mr-2" />
                    Saving & Testing...
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
