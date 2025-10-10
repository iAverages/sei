import { Link } from "@tanstack/solid-router";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuGroup,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu";
import { SidebarMenu, SidebarMenuButton, SidebarMenuItem } from "~/components/ui/sidebar";
import type { User } from "~/lib/user";

export function NavUser({ user }: { user: User }) {
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
                        {/* TODO */}
                        {/* <DropdownMenuSeparator /> */}
                        {/* <DropdownMenuGroup> */}
                        {/*     <DropdownMenuItem>Account</DropdownMenuItem> */}
                        {/*     <DropdownMenuItem>Billing</DropdownMenuItem> */}
                        {/*     <DropdownMenuItem>Notifications</DropdownMenuItem> */}
                        {/* </DropdownMenuGroup> */}
                        <DropdownMenuSeparator />
                        <DropdownMenuItem as={Link} to="/logout">
                            Log out
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </SidebarMenuItem>
        </SidebarMenu>
    );
}
