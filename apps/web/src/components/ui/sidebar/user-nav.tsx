import { useNavigate, useRouter } from "@tanstack/solid-router";
import { createSignal } from "solid-js";
import { toast } from "solid-sonner";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu";
import { SidebarMenu, SidebarMenuButton, SidebarMenuItem } from "~/components/ui/sidebar";
import { logout, refreshMalAnime, type User } from "~/lib/user";

export function NavUser({ user }: { user: User }) {
    const navigate = useNavigate();
    const router = useRouter();
    const [refreshing, setRefreshing] = createSignal(false);

    return (
        <SidebarMenu>
            <SidebarMenuItem>
                <DropdownMenu>
                    <DropdownMenuTrigger
                        as={SidebarMenuButton}
                        size="lg"
                        class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground cursor-pointer"
                    >
                        <Avatar class="h-8 w-8 rounded-lg grayscale">
                            <AvatarImage src={user.picture} alt={user.name} />
                            <AvatarFallback class="rounded-lg">CN</AvatarFallback>
                        </Avatar>
                        <div class="grid flex-1 text-left text-sm leading-tight">
                            <span class="truncate font-medium">{user.name}</span>
                        </div>
                        ...
                    </DropdownMenuTrigger>

                    <DropdownMenuContent class="w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg">
                        <DropdownMenuLabel class="p-0 font-normal">
                            <div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                                <Avatar class="h-8 w-8 rounded-lg">
                                    <AvatarImage src={user.picture} alt={user.name} />
                                    <AvatarFallback class="rounded-lg">CN</AvatarFallback>
                                </Avatar>
                                <div class="grid flex-1 text-left text-sm leading-tight">
                                    <span class="truncate font-medium">{user.name}</span>
                                </div>
                            </div>
                        </DropdownMenuLabel>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                            disabled={refreshing()}
                            onClick={async () => {
                                setRefreshing(true);
                                try {
                                    await refreshMalAnime();
                                    await router.invalidate({ sync: true });
                                    toast.success("MAL anime list refreshed");
                                } catch (error) {
                                    toast.error(
                                        error instanceof Error ? error.message : "Failed to refresh MAL anime list",
                                    );
                                } finally {
                                    setRefreshing(false);
                                }
                            }}
                        >
                            Refresh MAL anime
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem
                            onClick={async () => {
                                try {
                                    await logout();
                                    await navigate({ to: "/login" });
                                    await router.invalidate({ sync: true });
                                } catch (error) {
                                    toast.error(error instanceof Error ? error.message : "Failed to log out");
                                }
                            }}
                        >
                            Log out
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </SidebarMenuItem>
        </SidebarMenu>
    );
}
