import * as DialogPrimitive from "@kobalte/core/dialog";
import type { PolymorphicProps } from "@kobalte/core/polymorphic";
import type { ComponentProps, JSX, ValidComponent } from "solid-js";
import { splitProps } from "solid-js";
import { cn } from "~/lib/utils";

const Dialog = DialogPrimitive.Root;
const DialogTrigger = DialogPrimitive.Trigger;
const DialogPortal = DialogPrimitive.Portal;

type DialogOverlayProps<T extends ValidComponent = "div"> = DialogPrimitive.DialogOverlayProps<T> & {
    class?: string;
};

const DialogOverlay = <T extends ValidComponent = "div">(props: PolymorphicProps<T, DialogOverlayProps<T>>) => {
    const [local, others] = splitProps(props as DialogOverlayProps, ["class"]);
    return (
        <DialogPrimitive.Overlay
            class={cn(
                "fixed inset-0 z-50 bg-black/50 animate-out fade-out-0 data-[expanded]:animate-in data-[expanded]:fade-in-0",
                local.class,
            )}
            {...others}
        />
    );
};

type DialogContentProps<T extends ValidComponent = "div"> = DialogPrimitive.DialogContentProps<T> & {
    class?: string;
    children?: JSX.Element;
};

const DialogContent = <T extends ValidComponent = "div">(props: PolymorphicProps<T, DialogContentProps<T>>) => {
    const [local, others] = splitProps(props as DialogContentProps, ["class", "children"]);
    return (
        <DialogPortal>
            <DialogOverlay />
            <DialogPrimitive.Content
                class={cn(
                    "fixed left-1/2 top-1/2 z-50 grid max-h-[calc(100vh-2rem)] w-full max-w-[calc(100%-2rem)] -translate-x-1/2 -translate-y-1/2 gap-4 overflow-y-auto rounded-lg border bg-background p-6 shadow-lg duration-200 animate-out fade-out-0 zoom-out-95 data-[expanded]:animate-in data-[expanded]:fade-in-0 data-[expanded]:zoom-in-95 sm:max-w-lg",
                    local.class,
                )}
                {...others}
            >
                {local.children}
                <DialogPrimitive.CloseButton class="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-hidden focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="size-4"
                        aria-hidden="true"
                    >
                        <path d="M18 6l-12 12" />
                        <path d="M6 6l12 12" />
                    </svg>
                    <span class="sr-only">Close</span>
                </DialogPrimitive.CloseButton>
            </DialogPrimitive.Content>
        </DialogPortal>
    );
};

const DialogHeader = (props: ComponentProps<"div">) => {
    const [local, others] = splitProps(props, ["class"]);
    return <div class={cn("flex flex-col gap-2 text-center sm:text-left", local.class)} {...others} />;
};

type DialogTitleProps<T extends ValidComponent = "h2"> = DialogPrimitive.DialogTitleProps<T> & {
    class?: string;
};

const DialogTitle = <T extends ValidComponent = "h2">(props: PolymorphicProps<T, DialogTitleProps<T>>) => {
    const [local, others] = splitProps(props as DialogTitleProps, ["class"]);
    return <DialogPrimitive.Title class={cn("text-lg font-semibold", local.class)} {...others} />;
};

type DialogDescriptionProps<T extends ValidComponent = "p"> = DialogPrimitive.DialogDescriptionProps<T> & {
    class?: string;
};

const DialogDescription = <T extends ValidComponent = "p">(props: PolymorphicProps<T, DialogDescriptionProps<T>>) => {
    const [local, others] = splitProps(props as DialogDescriptionProps, ["class"]);
    return <DialogPrimitive.Description class={cn("text-sm text-muted-foreground", local.class)} {...others} />;
};

export { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger };
