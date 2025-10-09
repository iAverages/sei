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

export function NavUser({
    user,
}: {
    user: {
        name: string;
        email: string;
        avatar: string;
    };
}) {
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
                            <AvatarImage src={user.avatar} alt={user.name} />
                            <AvatarFallback class="rounded-lg">CN</AvatarFallback>
                        </Avatar>
                        <div class="grid flex-1 text-left text-sm leading-tight">
                            <span class="truncate font-medium">{user.name}</span>
                            <span class="text-muted-foreground truncate text-xs">{user.email}</span>
                        </div>
                        ...
                    </DropdownMenuTrigger>

                    <DropdownMenuContent class="w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg">
                        <DropdownMenuLabel class="p-0 font-normal">
                            <div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                                <Avatar class="h-8 w-8 rounded-lg">
                                    <AvatarImage src={user.avatar} alt={user.name} />
                                    <AvatarFallback class="rounded-lg">CN</AvatarFallback>
                                </Avatar>
                                <div class="grid flex-1 text-left text-sm leading-tight">
                                    <span class="truncate font-medium">{user.name}</span>
                                    <span class="text-muted-foreground truncate text-xs">{user.email}</span>
                                </div>
                            </div>
                        </DropdownMenuLabel>
                        <DropdownMenuSeparator />
                        <DropdownMenuGroup>
                            <DropdownMenuItem>Account</DropdownMenuItem>
                            <DropdownMenuItem>Billing</DropdownMenuItem>
                            <DropdownMenuItem>Notifications</DropdownMenuItem>
                        </DropdownMenuGroup>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem>Log out</DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </SidebarMenuItem>
        </SidebarMenu>
    );
}
