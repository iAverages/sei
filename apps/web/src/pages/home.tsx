import { createSignal } from "solid-js";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "~/components/ui/button";

export default function Home() {
    const [count, setCount] = createSignal(0);

    return (
        <div class={"p-6"}>
            <Card class="w-[380px]">
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
            </Card>
        </div>
    );
}
