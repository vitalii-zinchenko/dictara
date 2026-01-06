import { Link, useRouterState } from '@tanstack/react-router'
import { Download, Info, Key, Keyboard } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import { Separator } from '../ui/separator'
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarTrigger,
} from '../ui/sidebar'

interface PreferencesLayoutProps {
  children: ReactNode
}

const menuItems = [
  {
    title: 'API Keys',
    url: '/preferences/api-keys',
    icon: Key,
  },
  {
    title: 'Hotkeys',
    url: '/preferences/hotkeys',
    icon: Keyboard,
  },
  {
    title: 'Updates',
    url: '/preferences/updates',
    icon: Download,
  },
  {
    title: 'About',
    url: '/preferences/about',
    icon: Info,
  },
]

export function PreferencesLayout({ children }: PreferencesLayoutProps) {
  const routerState = useRouterState()
  const currentPath = routerState.location.pathname
  const [sidebarOpen, setSidebarOpen] = useState(true)

  // Get current page title
  const currentPage = menuItems.find((item) => item.url === currentPath)
  const pageTitle = currentPage?.title ?? 'Preferences'

  return (
    <SidebarProvider
      open={sidebarOpen}
      onOpenChange={setSidebarOpen}
      style={{ '--sidebar-width': '10rem' } as React.CSSProperties}
    >
      <Sidebar collapsible="icon">
        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupContent>
              <SidebarMenu>
                {menuItems.map((item) => (
                  <SidebarMenuItem key={item.title}>
                    <SidebarMenuButton asChild isActive={currentPath === item.url}>
                      <Link to={item.url}>
                        <item.icon />
                        <span>{item.title}</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>
      </Sidebar>
      <SidebarInset>
        <header className="flex h-14 shrink-0 items-center gap-2 border-b px-4">
          <SidebarTrigger className="-ml-1" />
          <Separator orientation="vertical" className="mr-2 h-4" />
          <span className="font-medium">{pageTitle}</span>
        </header>
        <main className="flex-1 overflow-y-auto overflow-x-hidden p-6">{children}</main>
      </SidebarInset>
    </SidebarProvider>
  )
}
