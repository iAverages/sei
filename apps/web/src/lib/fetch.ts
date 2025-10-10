import { createIsomorphicFn } from "@tanstack/solid-start";
import { Cookies } from "./cookies";

const getApiUrl = createIsomorphicFn()
    .server(() => "http://localhost:3001")
    .client(() => "");

export const api = (input: RequestInfo | URL, init?: RequestInit) => {
    const headers = new Headers(init?.headers);
    headers.append("Cookie", Cookies.getRaw());

    console.log({
        headers,
    });

    return fetch(getApiUrl() + input, {
        ...init,
        headers,
    });
};
