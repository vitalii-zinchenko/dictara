import { createFileRoute } from '@tanstack/react-router'
import { About } from '@/components/preferences/About'

export const Route = createFileRoute('/preferences/about')({
  component: AboutRoute,
})

function AboutRoute() {
  return <About />
}
