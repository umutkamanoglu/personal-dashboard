import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarHeader,
} from "@/components/ui/sidebar"

import { Gauge } from "lucide-react"

export function AppSidebar() {
    return (
        <Sidebar>
            <SidebarHeader>
                <h2 className="flex items-center justify-center font-medium text-xl"><Gauge className="mr-2" /> Personal Dashboard</h2>
            </SidebarHeader>
            <SidebarContent>
                <SidebarGroup />
                <SidebarGroup />
            </SidebarContent>
            <SidebarFooter />
        </Sidebar>
    )
}