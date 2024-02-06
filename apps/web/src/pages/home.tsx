import { Show, createSignal } from "solid-js";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "~/components/ui/button";
import { RouteDefinition } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
const createUser = () =>
    createQuery(() => ({
        queryKey: ["user"],
        queryFn: async () => {
            return fetch("http://localhost:3001/api/v1/auth/me", {
                credentials: "include",
            }).then((res) => res.json());
        },
    }));
export default function Home(props: RouteDefinition) {
    const user = createUser();

    return (
        <div class={"p-6"}>
            <a href="http://localhost:3001/oauth/mal/redirect">
                <Button>Login</Button>
            </a>
            <Show when={user.data} fallback={<>Loading user...</>}>
                <p>{JSON.stringify(user.data)}</p>
            </Show>
            {/* <Card class="w-[380px]">
                <CardHeader>
                    <CardTitle>Notifications</CardTitle>
                    <CardDescription>You have 3 unread messages.</CardDescription>
                </CardHeader>
                <CardContent class="grid gap-4">
                    <div class=" flex items-center space-x-4 rounded-md border p-4">
                        <div class="flex-1 space-y-1">
                            <p class="text-sm font-medium leading-none">Push Notifications</p>
                            <p class="text-muted-foreground text-sm">Send notifications to device.</p>
                        </div>
                    </div>
                    <div>
                        <div class="mb-4 grid grid-cols-[25px_1fr] items-start pb-4 last:mb-0 last:pb-0">
                            <span class="flex h-2 w-2 translate-y-1 rounded-full bg-sky-500" />
                            <div class="space-y-1">
                                <p class="text-sm font-medium leading-none">cum</p>
                                <p class="text-muted-foreground text-sm">ballz</p>
                            </div>
                        </div>
                    </div>
                </CardContent>
                <CardFooter>
                    <Button class="w-full">Mark all as read</Button>
                </CardFooter>
            </Card> */}
        </div>
    );
}
