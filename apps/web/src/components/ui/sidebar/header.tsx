import { Link, useMatches } from "@tanstack/solid-router";
import { For, Show } from "solid-js";
import {
    Breadcrumb,
    BreadcrumbItem,
    BreadcrumbLink,
    BreadcrumbList,
    BreadcrumbSeparator,
} from "~/components/ui/breadcrumb";
import { Separator } from "~/components/ui/separator";
import { SidebarTrigger } from "~/components/ui/sidebar";
import type { HeaderButtonsAreaSetter } from "./buttons-portal";

export const Header = (props: { setButtonsAreaRef: HeaderButtonsAreaSetter }) => {
    const matches = useMatches();
    const breadcrumbItems = () =>
        matches()
            .filter((match) => match.loaderData?.crumb)
            .flatMap(({ pathname, loaderData }) => ({
                href: pathname,
                label: loaderData!.crumb,
            }));

    matches().filter((match) => match.context);

    return (
        <header class="flex h-(--header-height) shrink-0 items-center gap-2 border-b transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)">
            <div class="flex w-full items-center gap-1 px-4 lg:gap-2 lg:px-6">
                <SidebarTrigger class="-ml-1" />
                <Separator orientation="vertical" class="mx-2 data-[orientation=vertical]:h-4" />

                <Breadcrumb>
                    <BreadcrumbList>
                        <For each={breadcrumbItems()}>
                            {(crumb, index) => (
                                <Show
                                    when={breadcrumbItems().length > 1 && index() === 0}
                                    fallback={
                                        <>
                                            {index() !== 0 && <BreadcrumbSeparator class="hidden md:block" />}
                                            <BreadcrumbItem>{crumb.label}</BreadcrumbItem>
                                        </>
                                    }
                                >
                                    <BreadcrumbItem class="hidden md:block">
                                        <BreadcrumbLink as={Link} to={crumb.href}>
                                            {crumb.label}
                                        </BreadcrumbLink>
                                    </BreadcrumbItem>
                                </Show>
                            )}
                        </For>
                    </BreadcrumbList>
                </Breadcrumb>
                <div ref={props.setButtonsAreaRef} />
            </div>
        </header>
    );
};
