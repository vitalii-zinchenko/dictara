import { Check } from 'lucide-react'
import type { OpenAITranscriptionModel } from '@/bindings'
import { cn } from '@/lib/utils'
import { Label } from '../../ui/label'
import { OPENAI_MODEL_OPTIONS, PRICING_AS_OF_LABEL } from './openaiModels'

interface OpenAIModelSelectorProps {
  value: OpenAITranscriptionModel
  onChange: (model: OpenAITranscriptionModel) => void
  disabled?: boolean
}

/**
 * Radio-group selector for the OpenAI file-transcription model.
 *
 * Built from native radio inputs so arrow keys, Tab and Space work without
 * extra key handling.
 */
export function OpenAIModelSelector({ value, onChange, disabled }: OpenAIModelSelectorProps) {
  return (
    <div className="space-y-2">
      <div className="flex items-baseline justify-between gap-2">
        <Label id="openai-model-label">Transcription Model</Label>
        <span className="text-xs text-muted-foreground">Pricing as of {PRICING_AS_OF_LABEL}</span>
      </div>

      <div role="radiogroup" aria-labelledby="openai-model-label" className="space-y-2">
        {OPENAI_MODEL_OPTIONS.map((model) => {
          const isSelected = model.id === value

          return (
            <label
              key={model.id}
              className={cn(
                'flex items-start gap-3 rounded-lg border p-3 transition-colors',
                disabled ? 'opacity-60' : 'cursor-pointer hover:bg-accent/50',
                isSelected && 'border-primary bg-accent/30',
                'focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2'
              )}
            >
              <input
                type="radio"
                name="openai-model"
                value={model.id}
                checked={isSelected}
                disabled={disabled}
                onChange={() => onChange(model.id)}
                className="sr-only"
              />

              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between gap-2">
                  <span className="flex items-center gap-2 font-medium text-sm">
                    {model.label}
                    {isSelected && <Check className="h-4 w-4 text-primary" />}
                  </span>
                  <span className="text-xs text-muted-foreground whitespace-nowrap">
                    {model.pricePerMinute}
                  </span>
                </div>

                <p className="text-xs text-muted-foreground mt-0.5">{model.description}</p>

                {model.tokenPricing && (
                  <ul className="mt-1 space-y-0.5 text-xs text-muted-foreground">
                    {model.tokenPricing.map((line) => (
                      <li key={line}>{line}</li>
                    ))}
                  </ul>
                )}
              </div>
            </label>
          )
        })}
      </div>
    </div>
  )
}
