import type { OpenAITranscriptionModel } from '@/bindings'

/**
 * Single source of truth for OpenAI transcription model metadata.
 *
 * Pricing is intentionally static — there is no runtime pricing fetch. When
 * OpenAI changes prices, update the entries below together with
 * PRICING_AS_OF_LABEL.
 *
 * Verified against https://developers.openai.com/api/docs/pricing
 */
export const PRICING_AS_OF_LABEL = 'Aug 22, 2026'

export interface OpenAIModelOption {
  /** API model id, also the persisted value */
  id: OpenAITranscriptionModel
  /** Friendly name shown in the selector */
  label: string
  /** Headline per-minute audio price */
  pricePerMinute: string
  /** Token pricing, shown where per-minute alone is not the whole story */
  tokenPricing?: string[]
  /** Short capability note */
  description: string
}

export const OPENAI_MODEL_OPTIONS: OpenAIModelOption[] = [
  {
    id: 'gpt-transcribe',
    label: 'GPT Transcribe',
    pricePerMinute: '~$0.0045/min',
    description: 'Recommended for recorded speech in its original language.',
  },
  {
    id: 'gpt-4o-mini-transcribe',
    label: 'GPT-4o Mini Transcribe',
    pricePerMinute: '~$0.003/min',
    description: 'Fastest and cheapest option.',
  },
  {
    id: 'gpt-4o-transcribe',
    label: 'GPT-4o Transcribe',
    pricePerMinute: '~$0.006/min',
    description: 'Higher accuracy than the mini model.',
  },
  {
    id: 'gpt-4o-transcribe-diarize',
    label: 'GPT-4o Transcribe + Diarization',
    pricePerMinute: '~$0.006/min',
    tokenPricing: ['$2.50 / 1M audio input tokens', '$10.00 / 1M output tokens'],
    description: 'Adds speaker labels. Dictara still inserts the plain transcript.',
  },
  {
    id: 'whisper-1',
    label: 'Whisper',
    pricePerMinute: '~$0.006/min',
    description: 'The original Whisper V2 model.',
  },
]

/** Model used when no selection has been persisted yet. */
export const DEFAULT_OPENAI_MODEL: OpenAITranscriptionModel = 'whisper-1'

export function getOpenAIModelOption(id: OpenAITranscriptionModel): OpenAIModelOption {
  return OPENAI_MODEL_OPTIONS.find((m) => m.id === id) ?? OPENAI_MODEL_OPTIONS[0]
}
