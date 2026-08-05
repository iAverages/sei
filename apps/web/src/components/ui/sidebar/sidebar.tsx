import { Link, useNavigate, useRouter } from "@tanstack/solid-router";
import { createSignal, For, Show } from "solid-js";
import { ListForm } from "~/components/list-form";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "~/components/ui/dialog";
import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarGroupAction,
    SidebarGroupContent,
    SidebarGroupLabel,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    useSidebar,
} from "~/components/ui/sidebar";
import { createList, type ListSummary } from "~/lib/list";
import type { User } from "~/lib/user";
import { NavUser } from "./user-nav";

export function AppSidebar(props: { user: User; lists: ListSummary[] }) {
    const [createOpen, setCreateOpen] = createSignal(false);
    const router = useRouter();
    const navigate = useNavigate();
    const { setOpenMobile } = useSidebar();

    return (
        <>
            <Sidebar collapsible="offcanvas" variant="inset">
                <SidebarHeader>
                    <SidebarMenu>
                        <SidebarMenuItem>
                            <SidebarMenuButton as={Link} to="/" class="data-[slot=sidebar-menu-button]:!p-1.5">
                                <span class="text-base font-semibold">Sei</span>
                            </SidebarMenuButton>
                        </SidebarMenuItem>
                    </SidebarMenu>
                </SidebarHeader>
                <SidebarContent>
                    <SidebarGroup>
                        <SidebarGroupLabel>Lists</SidebarGroupLabel>
                        <SidebarGroupAction
                            onClick={() => setCreateOpen(true)}
                            aria-label="Create list"
                            title="Create list"
                        >
                            <span aria-hidden="true" class="text-lg leading-none">
                                +
                            </span>
                        </SidebarGroupAction>
                        <SidebarGroupContent>
                            <SidebarMenu>
                                <For each={props.lists}>
                                    {(list) => (
                                        <SidebarMenuItem>
                                            <Link
                                                to="/$listId"
                                                params={{ listId: list.id }}
                                                onClick={() => setOpenMobile(false)}
                                                class="flex h-8 w-full items-center overflow-hidden rounded-md p-2 text-sm outline-hidden ring-sidebar-ring hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 data-[status=active]:bg-sidebar-accent data-[status=active]:font-medium data-[status=active]:text-sidebar-accent-foreground"
                                            >
                                                <span>{list.name}</span>
                                            </Link>
                                        </SidebarMenuItem>
                                    )}
                                </For>
                            </SidebarMenu>
                        </SidebarGroupContent>
                    </SidebarGroup>
                </SidebarContent>
                <SidebarFooter>
                    <NavUser user={props.user} />
                </SidebarFooter>
            </Sidebar>

            <Dialog open={createOpen()} onOpenChange={setCreateOpen}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Create a list</DialogTitle>
                        <DialogDescription>
                            Choose a name, privacy level, and optional starting titles.
                        </DialogDescription>
                    </DialogHeader>
                    <Show when={createOpen()}>
                        <ListForm
                            submitLabel="Create list"
                            onSubmit={async (value) => {
                                const detail = await createList(value);
                                setCreateOpen(false);
                                await router.invalidate();
                                await navigate({ to: "/$listId", params: { listId: detail.list.id } });
                            }}
                        />
                    </Show>
                </DialogContent>
            </Dialog>
        </>
    );
}
