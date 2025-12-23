import * as React from "react"

// Mobile mode disabled - always use desktop sidebar layout
export function useIsMobile() {
  // Keep the hook structure to satisfy React's rules of hooks
  const [isMobile] = React.useState(false)
  return isMobile
}
