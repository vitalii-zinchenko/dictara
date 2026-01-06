import { OpenAIProvider } from './OpenAiProvider'
import { AzureOpenAIProvider } from './AzureProvider'
import { LocalProvider } from './LocalProvider'
import type { Provider } from './types'

interface ProviderListProps {
  activeProvider: Provider
  expandedSection: Provider
  onToggleExpand: (provider: Provider) => void
  onToggleActive: (provider: Provider) => void
}

/**
 * Renders the list of all available transcription providers.
 * Used in both Preferences and Onboarding flows.
 */
export function ProviderList({
  activeProvider,
  expandedSection,
  onToggleExpand,
  onToggleActive,
}: ProviderListProps) {
  return (
    <div className="space-y-3">
      <OpenAIProvider
        isExpanded={expandedSection === 'open_ai'}
        isActive={activeProvider === 'open_ai'}
        onToggleExpand={onToggleExpand}
        onToggleActive={onToggleActive}
      />

      <AzureOpenAIProvider
        isExpanded={expandedSection === 'azure_open_ai'}
        isActive={activeProvider === 'azure_open_ai'}
        onToggleExpand={onToggleExpand}
        onToggleActive={onToggleActive}
      />

      <LocalProvider
        isExpanded={expandedSection === 'local'}
        isActive={activeProvider === 'local'}
        onToggleExpand={onToggleExpand}
        onToggleActive={onToggleActive}
      />
    </div>
  )
}
