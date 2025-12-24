import { ChevronDown, ChevronRight } from 'lucide-react'
import type { ReactNode } from 'react'
import { Collapsible, CollapsibleContent } from '../../ui/collapsible'
import { Switch } from '../../ui/switch'
import type { Provider } from './types'

interface ProviderSectionProps {
  provider: NonNullable<Provider>
  title: string
  isExpanded: boolean
  isActive: boolean
  canEnable: boolean
  onToggleExpand: (provider: Provider) => void
  onToggleActive: (provider: Provider) => void
  children: ReactNode
}

export function ProviderSection({
  provider,
  title,
  isExpanded,
  isActive,
  canEnable,
  onToggleExpand,
  onToggleActive,
  children,
}: ProviderSectionProps) {
  return (
    <Collapsible open={isExpanded}>
      <div className="border rounded-lg p-4">
        <div className="flex items-center justify-between w-full">
          <div className="flex items-center gap-2">
            {isExpanded ? (
              <ChevronDown
                className="h-4 w-4 text-muted-foreground cursor-pointer hover:text-foreground"
                onClick={() => onToggleExpand(provider)}
              />
            ) : (
              <ChevronRight
                className="h-4 w-4 text-muted-foreground cursor-pointer hover:text-foreground"
                onClick={() => onToggleExpand(provider)}
              />
            )}
            <h3 className="text-lg font-semibold">{title}</h3>
          </div>
          <Switch
            checked={isActive}
            disabled={!canEnable}
            onCheckedChange={() => onToggleActive(provider)}
          />
        </div>

        <CollapsibleContent className="mt-3">{children}</CollapsibleContent>
      </div>
    </Collapsible>
  )
}
