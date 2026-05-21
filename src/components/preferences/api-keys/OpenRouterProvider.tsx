import {
  useDeleteOpenRouterConfig,
  useOpenRouterConfig,
  useSaveOpenRouterConfig,
  useTestOpenRouterConfig,
} from '@/hooks/useOpenRouterConfig'
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

interface OpenRouterProviderProps {
  isActive: boolean
  isExpanded: boolean
  onToggleActive: (provider: Provider) => void
  onToggleExpand: (provider: Provider) => void
}

export function OpenRouterProvider({
  isActive,
  isExpanded,
  onToggleActive,
  onToggleExpand,
}: OpenRouterProviderProps) {
  const [saveSuccess, setSaveSuccess] = useState(false)

  // TanStack Query hooks
  const { data: existingConfig, isLoading } = useOpenRouterConfig()
  const saveConfig = useSaveOpenRouterConfig()
  const testConfig = useTestOpenRouterConfig()
  const deleteConfig = useDeleteOpenRouterConfig()

  const form = useForm({
    defaultValues: {
      apiKey: existingConfig?.configured ? MASKED_API_KEY_PLACEHOLDER : '',
      model: existingConfig?.model || 'openai/whisper-large-v3-turbo',
    },
    validators: {
      onSubmitAsync: async ({ value }) => {
        await waitForPaint()

        try {
          const isValid = await testConfig.mutateAsync({
            apiKey: value.apiKey,
            model: value.model,
          })

          if (!isValid) {
            return {
              form: 'Invalid OpenRouter API key or model. Please check your inputs.',
              fields: {},
            }
          }

          return undefined
        } catch (e) {
          logError(`[OpenRouterProvider] onSubmitAsync caught error: ${e}`)
          return {
            form: 'Failed to validate OpenRouter config. Please try again.',
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
          model: value.model,
        })
        setSaveSuccess(true)
        form.reset()
        // Auto-enable the provider after successful save
        if (!isActive) {
          onToggleActive('open_router')
        }
      } catch (e) {
        logError(`[OpenRouterProvider] Failed to save config: ${e}`)
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
        onToggleActive('open_router')
      }
    } catch (e) {
      logError(`[OpenRouterProvider] Failed to delete config: ${e}`)
    }
  }

  // Derive error message from mutations
  const errorMessage = saveConfig.error?.message || deleteConfig.error?.message

  if (isLoading) {
    return (
      <ProviderSection
        provider="open_router"
        title="OpenRouter"
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
      provider="open_router"
      title="OpenRouter"
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
        <div className="space-y-2">
          <Label htmlFor="openrouter-api-key">
            {existingConfig ? 'Update API Key' : 'OpenRouter API Key'}
          </Label>
          <form.Field
            name="apiKey"
            validators={{
              onChange: ({ value }) => {
                if (!value) return 'API key is required'
                return undefined
              },
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <div className="flex gap-2">
                  <Input
                    id="openrouter-api-key"
                    type="password"
                    placeholder={existingConfig ? MASKED_API_KEY_PLACEHOLDER : 'sk-or-...'}
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

        <div className="space-y-2">
          <Label htmlFor="openrouter-model">Model</Label>
          <form.Field
            name="model"
            validators={{
              onChange: ({ value }) => {
                if (!value) return 'Model name is required'
                return undefined
              },
            }}
          >
            {(field) => (
              <div className="space-y-1">
                <Input
                  id="openrouter-model"
                  type="text"
                  placeholder="openai/whisper-large-v3-turbo"
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
