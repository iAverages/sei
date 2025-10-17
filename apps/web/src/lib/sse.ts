import { createEffect, onCleanup } from "solid-js";

export type UseSseStreamProps<T> = {
    url: string;
    onMessage?: (message: T) => void;
};
export const useSseStream = <T>({ url, onMessage }: UseSseStreamProps<T>) => {
    createEffect(() => {
        const eventSource = new EventSource(url);
        eventSource.onopen = (event) => {
            console.log("connected to sse", event);
        };
        eventSource.onmessage = (event) => {
            try {
                const parsedData: T = JSON.parse(event.data);
                // TODO: add validation
                // setTimeout(() => {
                onMessage?.(parsedData as T);
                // }, 1);
            } catch (error) {
                console.error("Failed to parse SSE data:", error);
            }
        };

        eventSource.onerror = (error) => {
            console.error("SSE Error:", error);
            eventSource.close();
        };

        onCleanup(() => eventSource.close());
    });
};
