import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar"
import { MonitorSpeaker, Home, Activity, Clapperboard } from "lucide-react"
import { Link } from "react-router-dom";

export function AppSidebar() {
  const { state } = useSidebar();
  const routes = [
    {
      title: "Ana Sayfa",
      path: "/",
      icon: <Home />
    },
    {
      title: "Sistem İzleyici",
      path: "/system",
      icon: <Activity />
    },
     {
      title: "Dizi Film",
      path: "/tmdb",
      icon: <Clapperboard />
     }
  ]

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <div className="flex items-center gap-2 p-2">
          <MonitorSpeaker className="size-6 shrink-0" />
          {/* state kontrolünün içine whitespace-nowrap ekledik */}
          {state === "expanded" && (
            <div className="flex flex-col overflow-hidden">
              <span className="font-bold leading-none whitespace-nowrap">
                Personal Dashboard
              </span>
            </div>
          )}
        </div>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarMenu>
            <SidebarMenuItem>
              {routes.map((route, key) => {
                console.log(route)
                return (
                  <SidebarMenuButton key={key} asChild>
                    <Link to={route.path}>
                      {route.icon}
                      <span>{route.title}</span>
                    </Link>
                  </SidebarMenuButton>
                )
              })}
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  )
}