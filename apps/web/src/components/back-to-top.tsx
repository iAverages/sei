import { makeEventListener } from "@solid-primitives/event-listener";
import { createSignal, onCleanup, onMount } from "solid-js";
import { Portal } from "solid-js/web";
import { ArrowUp } from "~/icons";
import { cn } from "~/lib/utils";

export const BackToTop = () => {
    const [visible, setVisible] = createSignal(typeof window === "undefined" ? false : window.scrollY > 240);
    onMount(() => {
        let raf = 0;
        const onScroll = () => {
            if (raf) return;
            raf = requestAnimationFrame(() => {
                setVisible(window.scrollY > 240);
                raf = 0;
            });
        };
        onScroll();
        makeEventListener(document, "scroll", onScroll);
        onCleanup(() => {
            if (raf) cancelAnimationFrame(raf);
        });
    });

    const handleClick = () => {
        window.scrollTo({ top: 0, behavior: "smooth" });
    };

    return (
        <Portal>
            <button
                type="button"
                aria-label="Go to top"
                onClick={handleClick}
                class={cn(
                    "fixed bottom-6 right-6 z-50 grid h-12 w-12 place-items-center rounded-xl cursor-pointer text-white shadow-brand",
                    "bg-gradient-to-br from-indigo-600 to-violet-600 hover:from-indigo-500 hover:to-violet-500",
                    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 ring-offset-background",
                    "transition-all duration-300",
                    visible() ? "opacity-100 translate-y-0 animate-pop" : "opacity-0 translate-y-4 pointer-events-none",
                )}
            >
                <ArrowUp />
                <span class="sr-only">Back to top</span>
            </button>
        </Portal>
    );
};
