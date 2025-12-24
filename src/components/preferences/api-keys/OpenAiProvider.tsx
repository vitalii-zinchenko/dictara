import { useForm } from '@tanstack/react-form'
import { useState } from 'react'
import { Button } from '../../ui/button'
import { Input } from '../../ui/input'
import { Label } from '../../ui/label'
import { ProviderSection } from './ProviderSection'
import type { Provider } from './types'
import { MASKED_API_KEY_PLACEHOLDER } from './utils'
import {
  useOpenAIConfig,
  useSaveOpenAIConfig,
  useTestOpenAIConfig,
  useDeleteOpenAIConfig,
} from '@/hooks/useOpenAIConfig'

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
  const [saveSuccess, setSaveSuccess] = useState(false)

  // TanStack Query hooks
  const { data: existingConfig, isLoading } = useOpenAIConfig()
  const saveConfig = useSaveOpenAIConfig()
  const testConfig = useTestOpenAIConfig()
  const deleteConfig = useDeleteOpenAIConfig()

  const form = useForm({
    defaultValues: {
      apiKey: '',
    },
    validators: {
      onSubmitAsync: async ({ value }) => {
        // Validate the API key by testing it
        console.log('[OpenAIProvider] Validating API key...')

        try {
          const isValid = await testConfig.mutateAsync({ apiKey: value.apiKey })

          if (!isValid) {
            return {
              form: 'Invalid OpenAI API key. Please check your key and try again.',
              fields: {},
            }
          }

          return undefined
        } catch (e) {
          console.error('[OpenAIProvider] Validation failed:', e)
          return {
            form: 'Failed to validate API key. Please try again.',
            fields: {},
          }
        }
      },
    },
    onSubmit: async ({ value }) => {
      console.log('[OpenAIProvider] Saving config...')
      setSaveSuccess(false)

      try {
        await saveConfig.mutateAsync({ apiKey: value.apiKey })
        console.log('[OpenAIProvider] Config saved successfully')
        setSaveSuccess(true)
        form.reset()
      } catch (e) {
        console.error('[OpenAIProvider] Failed to save config:', e)
      }
    },
  })

  const handleDelete = async () => {
    console.log('[OpenAIProvider] Deleting config...')
    try {
      await deleteConfig.mutateAsync()
      console.log('[OpenAIProvider] Config deleted successfully')
      setSaveSuccess(false)
      form.reset()
    } catch (e) {
      console.error('[OpenAIProvider] Failed to delete config:', e)
    }
  }

  // Derive error message from mutations
  const errorMessage =
    saveConfig.error?.message || deleteConfig.error?.message

  if (isLoading) {
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
                <div className="flex gap-2">
                  <Input
                    id="openai-api-key"
                    type="password"
                    placeholder={existingConfig ? MASKED_API_KEY_PLACEHOLDER : 'sk-...'}
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
                  <p className="text-sm text-destructive">
                    {field.state.meta.errors.join(', ')}
                  </p>
                )}
              </div>
            )}
          </form.Field>
        </div>

        {/* Feedback messages */}
        <form.Subscribe selector={(state) => state.errorMap}>
          {(errorMap) => (
            <>
              {errorMap.onSubmit && (
                <p className="text-sm text-destructive">{errorMap.onSubmit}</p>
              )}
            </>
          )}
        </form.Subscribe>
        {errorMessage && <p className="text-sm text-destructive">{errorMessage}</p>}
        {saveSuccess && (
          <p className="text-sm text-green-600">Configuration saved successfully!</p>
        )}

        {/* Action buttons */}
        <div className="flex gap-2">
          <form.Subscribe
            selector={(formState) => ({
              canSubmit: formState.canSubmit,
              isSubmitting: formState.isSubmitting,
            })}
          >
            {({ canSubmit, isSubmitting }) => (
              <Button
                type="submit"
                disabled={!canSubmit || isSubmitting}
              >
                {isSubmitting || saveConfig.isPending ? 'Saving...' : 'Save'}
              </Button>
            )}
          </form.Subscribe>
        </div>
      </form>
    </ProviderSection>
  )
}
