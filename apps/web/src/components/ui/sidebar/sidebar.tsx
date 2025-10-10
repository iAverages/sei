import { Link } from "@tanstack/solid-router";
import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
} from "~/components/ui/sidebar";
import type { User } from "~/lib/user";
import { NavUser } from "./user-nav";

export function AppSidebar(props: { user: User }) {
    return (
        <Sidebar collapsible="offcanvas" variant="inset">
            <SidebarHeader>
                <SidebarMenu>
                    <SidebarMenuItem>
                        <SidebarMenuButton class="data-[slot=sidebar-menu-button]:!p-1.5">
                            <Link to="/">
                                <span class="text-base font-semibold">Sei</span>
                            </Link>
                        </SidebarMenuButton>
                    </SidebarMenuItem>
                </SidebarMenu>
            </SidebarHeader>
            <SidebarContent>{/* <NavMain items={data.navMain} /> */}</SidebarContent>
            <SidebarFooter>
                <NavUser user={props.user} />
            </SidebarFooter>
        </Sidebar>
    );
}
