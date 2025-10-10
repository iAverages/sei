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
import { NavUser } from "./user-nav";
import { User } from "~/lib/user";

const data = {
    user: {
        name: "shadcn",
        email: "m@example.com",
        avatar: "/avatars/shadcn.jpg",
    },
    navMain: [
        {
            title: "Dashboard",
            url: "#",
        },
        {
            title: "Lifecycle",
            url: "#",
        },
        {
            title: "Analytics",
            url: "#",
        },
        {
            title: "Projects",
            url: "#",
        },
        {
            title: "Team",
            url: "#",
        },
    ],
};

export function AppSidebar(props: { user: User }) {
    return (
        <Sidebar collapsible="offcanvas" variant="inset">
            <SidebarHeader>
                <SidebarMenu>
                    <SidebarMenuItem>
                        <SidebarMenuButton class="data-[slot=sidebar-menu-button]:!p-1.5">
                            <Link to="/">
                                {/* <IconInnerShadowTop className="!size-5" /> */}
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
