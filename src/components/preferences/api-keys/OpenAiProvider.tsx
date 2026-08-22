import {
  useDeleteOpenAIConfig,
  useOpenAIConfig,
  useSaveOpenAIConfig,
  useTestOpenAIConfig,
} from '@/hooks/useOpenAIConfig'
import { waitForPaint } from '@/utils/waitForPaint'
import { useForm } from '@tanstack/react-form'
import type { OpenAITranscriptionModel } from '@/bindings'
import { error as logError } from '@tauri-apps/plugin-log'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'
import { Button } from '../../ui/button'
import { Input } from '../../ui/input'
import { Label } from '../../ui/label'
import { OpenAIModelSelector } from './OpenAIModelSelector'
import { DEFAULT_OPENAI_MODEL } from './openaiModels'
import { ProviderSection } from './ProviderSection'
import type { Provider } from './types'
import { MASKED_API_KEY_PLACEHOLDER } from './utils'

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

  // Pending model selection, only set while the user is changing it. Falls back
  // to the persisted model so the selector always shows what is saved.
  const [pendingModel, setPendingModel] = useState<OpenAITranscriptionModel | null>(null)
  const savedModel = existingConfig?.model ?? DEFAULT_OPENAI_MODEL
  const selectedModel = pendingModel ?? savedModel

  const form = useForm({
    defaultValues: {
      apiKey: '',
    },
    validators: {
      onSubmitAsync: async ({ value }) => {
        // Workaround for TanStack Form not yielding to browser paint cycle
        // See: https://github.com/TanStack/form/issues/1967
        await waitForPaint()

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
          logError(`[OpenAIProvider] onSubmitAsync caught error: ${e}`)
          return {
            form: 'Failed to validate API key. Please try again.',
            fields: {},
          }
        }
      },
    },
    onSubmit: async ({ value }) => {
      setSaveSuccess(false)

      try {
        await saveConfig.mutateAsync({ apiKey: value.apiKey, model: selectedModel })
        setSaveSuccess(true)
        setPendingModel(null)
        form.reset()
        // Auto-enable the provider after successful save
        if (!isActive) {
          onToggleActive('open_ai')
        }
      } catch (e) {
        logError(`[OpenAIProvider] Failed to save config: ${e}`)
      }
    },
  })

  const handleDelete = async () => {
    try {
      await deleteConfig.mutateAsync()
      setSaveSuccess(false)
      setPendingModel(null)
      form.reset()
      // Deactivate the provider if it was active
      if (isActive) {
        onToggleActive('open_ai')
      }
    } catch (e) {
      logError(`[OpenAIProvider] Failed to delete config: ${e}`)
    }
  }

  // With a key already stored, the model can be changed on its own - the
  // backend keeps the stored key, which is never sent to the frontend.
  const handleSaveModelOnly = async () => {
    setSaveSuccess(false)

    try {
      await saveConfig.mutateAsync({ model: selectedModel })
      setSaveSuccess(true)
      setPendingModel(null)
    } catch (e) {
      logError(`[OpenAIProvider] Failed to save model: ${e}`)
    }
  }

  const handleSelectModel = (model: OpenAITranscriptionModel) => {
    setPendingModel(model)
    setSaveSuccess(false)
  }

  const hasUnsavedModelChange = !!existingConfig && selectedModel !== savedModel

  // Derive error message from mutations
  const errorMessage = saveConfig.error?.message || deleteConfig.error?.message

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
                  <p className="text-sm text-destructive">{field.state.meta.errors.join(', ')}</p>
                )}
              </div>
            )}
          </form.Field>
        </div>

        {/* Model selection - saved together with the key, or on its own */}
        <form.Subscribe selector={(state) => state.isSubmitting}>
          {(isSubmitting) => (
            <OpenAIModelSelector
              value={selectedModel}
              onChange={handleSelectModel}
              disabled={isSubmitting || saveConfig.isPending}
            />
          )}
        </form.Subscribe>

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
              <Button type="submit" disabled={!canSubmit || saveConfig.isPending}>
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

          {hasUnsavedModelChange && (
            <form.Subscribe selector={(state) => state.isSubmitting}>
              {(isSubmitting) => (
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleSaveModelOnly}
                  disabled={isSubmitting || saveConfig.isPending}
                >
                  {saveConfig.isPending ? (
                    <>
                      <Loader2 className="h-4 w-4 animate-spin" />
                      Saving...
                    </>
                  ) : (
                    'Save Model'
                  )}
                </Button>
              )}
            </form.Subscribe>
          )}
        </div>
      </form>
    </ProviderSection>
  )
}
