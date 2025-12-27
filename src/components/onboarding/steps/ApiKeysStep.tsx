import { useEffect, useRef, useState } from 'react'
import { StepContainer } from '../StepContainer'
import { useOnboardingNavigation } from '@/hooks/useOnboardingNavigation'
import { useAppConfig } from '@/hooks/useAppConfig'
import { useSaveAppConfig } from '@/hooks/useSaveAppConfig'
import { OpenAIProvider } from '@/components/preferences/api-keys/OpenAiProvider'
import { AzureOpenAIProvider } from '@/components/preferences/api-keys/AzureProvider'
import type { Provider } from '@/components/preferences/api-keys/types'

export function ApiKeysStep() {
  const { goNext, goBack, skipOnboarding, isNavigating } = useOnboardingNavigation()
  const { data: config, isLoading } = useAppConfig()
  const saveConfig = useSaveAppConfig()

  // Local state - initialized from config on first load
  const [activeProvider, setActiveProvider] = useState<Provider>(null)
  const [expandedSection, setExpandedSection] = useState<Provider>(null)
  const isInitialized = useRef(false)

  // Sync state from config on initial load only
  useEffect(() => {
    if (config && !isInitialized.current) {
      setActiveProvider(config.active_provider)
      setExpandedSection(config.active_provider)
      isInitialized.current = true
    }
  }, [config])

  // Toggle section expand/collapse (visual only)
  const handleToggleExpand = (provider: Provider) => {
    setExpandedSection(expandedSection === provider ? null : provider)
  }

  // Toggle provider activation (functional)
  const handleToggleProvider = (provider: Provider) => {
    const newProvider = activeProvider === provider ? null : provider
    const previousProvider = activeProvider
    setActiveProvider(newProvider)

    saveConfig.mutate(
      { activeProvider: newProvider },
      {
        onError: () => {
          setActiveProvider(previousProvider)
        },
      }
    )
  }

  // Check if any provider is configured and active
  const hasActiveProvider = activeProvider != null

  const handleNext = () => {
    if (hasActiveProvider) {
      goNext('api_keys')
    }
  }

  if (isLoading) {
    return (
      <StepContainer
        title="Configure API Keys"
        description="Loading..."
        showBack={true}
        showSkip={true}
        onBack={() => goBack('api_keys')}
        onSkip={() => skipOnboarding.mutate()}
      >
        <div className="text-muted-foreground">Loading configuration...</div>
      </StepContainer>
    )
  }

  return (
    <StepContainer
      title="Configure API Keys"
      description="Choose your speech recognition provider and enter your API credentials."
      onNext={handleNext}
      nextDisabled={!hasActiveProvider}
      onBack={() => goBack('api_keys')}
      onSkip={() => skipOnboarding.mutate()}
      isLoading={isNavigating || skipOnboarding.isPending}
    >
      <div className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Dictara uses OpenAI's Whisper model for speech recognition. You can use either OpenAI
          directly or Azure OpenAI. Only one provider can be active at a time.
        </p>

        <div className="space-y-3">
          <OpenAIProvider
            isExpanded={expandedSection === 'open_ai'}
            isActive={activeProvider === 'open_ai'}
            onToggleExpand={handleToggleExpand}
            onToggleActive={handleToggleProvider}
          />

          <AzureOpenAIProvider
            isExpanded={expandedSection === 'azure_open_ai'}
            isActive={activeProvider === 'azure_open_ai'}
            onToggleExpand={handleToggleExpand}
            onToggleActive={handleToggleProvider}
          />
        </div>

        {hasActiveProvider && (
          <p className="text-sm text-green-600 dark:text-green-400">
            Provider configured! Click Next to continue.
          </p>
        )}
      </div>
    </StepContainer>
  )
}
