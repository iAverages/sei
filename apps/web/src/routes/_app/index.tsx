import { createFileRoute } from "@tanstack/solid-router";
import { For } from "solid-js";
import {
    Sidebar,
    SidebarContent,
    SidebarGroup,
    SidebarGroupContent,
    SidebarGroupLabel,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
} from "~/components/ui/sidebar";

export const Route = createFileRoute("/_app/")({
    component: RouteComponent,
});

const items = [
    {
        title: "Home",
        url: "#",
    },
    {
        title: "Inbox",
        url: "#",
    },
    {
        title: "Calendar",
        url: "#",
    },
    {
        title: "Search",
        url: "#",
    },
    {
        title: "Settings",
        url: "#",
    },
];

function RouteComponent() {
    return (
        <main>
            <h1>Hello world!</h1>
        </main>
    );
}
