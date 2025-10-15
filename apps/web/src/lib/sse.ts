import { createEffect, createSignal, onCleanup } from "solid-js";

export type UseSseStreamProps<T> = {
    url: string;
    onMessage?: (message: T) => void;
};
export const useSseStream = <T>({ url, onMessage }: UseSseStreamProps<T>) => {
    const [data, setData] = createSignal<T[]>([]);

    createEffect(() => {
        const eventSource = new EventSource(url);
        eventSource.onopen = (event) => {
            console.log("connected to sse", event);
        };
        eventSource.onmessage = (event) => {
            try {
                const parsedData: T = JSON.parse(event.data);
                // TODO: add validation
                setData((prev) => [...prev, parsedData as T]);
                onMessage?.(parsedData as T);
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

    return { data };
};
