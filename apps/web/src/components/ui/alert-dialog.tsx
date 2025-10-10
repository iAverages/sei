import type { JSX, ValidComponent } from "solid-js";
import { splitProps } from "solid-js";

import * as AlertDialogPrimitive from "@kobalte/core/alert-dialog";
import type { PolymorphicProps } from "@kobalte/core/polymorphic";

import { cn } from "~/lib/utils";

const AlertDialog = AlertDialogPrimitive.Root;
const AlertDialogTrigger = AlertDialogPrimitive.Trigger;
const AlertDialogPortal = AlertDialogPrimitive.Portal;

type AlertDialogOverlayProps<T extends ValidComponent = "div"> = AlertDialogPrimitive.AlertDialogOverlayProps<T> & {
    class?: string | undefined;
};

const AlertDialogOverlay = <T extends ValidComponent = "div">(
    props: PolymorphicProps<T, AlertDialogOverlayProps<T>>,
) => {
    const [local, others] = splitProps(props as AlertDialogOverlayProps, ["class"]);
    return (
        <AlertDialogPrimitive.Overlay
            class={cn(
                "data-[expanded]:animate-in animate-out fade-out-0 data-[expanded]:fade-in-0 fixed inset-0 z-50 bg-black/50",
                local.class,
            )}
            {...others}
        />
    );
};

type AlertDialogContentProps<T extends ValidComponent = "div"> = AlertDialogPrimitive.AlertDialogContentProps<T> & {
    class?: string | undefined;
    children?: JSX.Element;
};

const AlertDialogContent = <T extends ValidComponent = "div">(
    props: PolymorphicProps<T, AlertDialogContentProps<T>>,
) => {
    const [local, others] = splitProps(props as AlertDialogContentProps, ["class", "children"]);
    return (
        <AlertDialogPortal>
            <AlertDialogOverlay />
            <AlertDialogPrimitive.Content
                class={cn(
                    "bg-background data-[expanded]:animate-in animate-out fade-out-0 data-[expanded]:fade-in-0 zoom-out-95 data-[expanded]:zoom-in-95 fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-4 rounded-lg border p-6 shadow-lg duration-200 sm:max-w-lg",
                    local.class,
                )}
                {...others}
            >
                {local.children}
                <AlertDialogPrimitive.CloseButton class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[expanded]:bg-accent data-[expanded]:text-muted-foreground">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="size-4"
                    >
                        <path d="M18 6l-12 12" />
                        <path d="M6 6l12 12" />
                    </svg>
                    <span class="sr-only">Close</span>
                </AlertDialogPrimitive.CloseButton>
            </AlertDialogPrimitive.Content>
        </AlertDialogPortal>
    );
};

type AlertDialogTitleProps<T extends ValidComponent = "h2"> = AlertDialogPrimitive.AlertDialogTitleProps<T> & {
    class?: string | undefined;
};

const AlertDialogTitle = <T extends ValidComponent = "h2">(props: PolymorphicProps<T, AlertDialogTitleProps<T>>) => {
    const [local, others] = splitProps(props as AlertDialogTitleProps, ["class"]);
    return <AlertDialogPrimitive.Title class={cn("text-lg font-semibold", local.class)} {...others} />;
};

type AlertDialogDescriptionProps<T extends ValidComponent = "p"> =
    AlertDialogPrimitive.AlertDialogDescriptionProps<T> & {
        class?: string | undefined;
    };

const AlertDialogDescription = <T extends ValidComponent = "p">(
    props: PolymorphicProps<T, AlertDialogDescriptionProps<T>>,
) => {
    const [local, others] = splitProps(props as AlertDialogDescriptionProps, ["class"]);
    return <AlertDialogPrimitive.Description class={cn("text-sm text-muted-foreground", local.class)} {...others} />;
};

const AlertDialogHeader = <T extends ValidComponent = "div">(
    props: PolymorphicProps<T, AlertDialogDescriptionProps<T>>,
) => {
    const [local, others] = splitProps(props as AlertDialogDescriptionProps, ["class"]);
    return (
        <div
            data-slot="alert-dialog-header"
            class={cn("flex flex-col gap-2 text-center sm:text-left", local.class)}
            {...others}
        />
    );
};
const AlertDialogFooter = <T extends ValidComponent = "div">(
    props: PolymorphicProps<T, AlertDialogDescriptionProps<T>>,
) => {
    const [local, others] = splitProps(props as AlertDialogDescriptionProps, ["class"]);
    return (
        <div
            data-slot="alert-dialog-footer"
            class={cn("flex flex-col-reverse gap-2 sm:flex-row sm:justify-end", local.class)}
            {...others}
        />
    );
};

export {
    AlertDialog,
    AlertDialogPortal,
    AlertDialogOverlay,
    AlertDialogTrigger,
    AlertDialogContent,
    AlertDialogTitle,
    AlertDialogDescription,
    AlertDialogHeader,
    AlertDialogFooter,
};
