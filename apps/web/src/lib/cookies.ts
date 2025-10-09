import { createIsomorphicFn } from "@tanstack/solid-start";
import { getRequestHeader, setCookie as setServerCookie } from "@tanstack/solid-start/server";
import serverCookie from "cookie"; // used for parsing cookie on the server
import clientCookie from "js-cookie"; // used for setting cookie on the client

export const Cookies = {
    get: (key: string, defaultValue?: string) => {
        const cookies = Cookies.getAll();
        if (!cookies) return defaultValue;
        return cookies[key] ?? defaultValue;
    },

    getAll: createIsomorphicFn()
        .server(() => {
            const cookieHeader = getRequestHeader("Cookie");
            if (!cookieHeader) return null;
            return serverCookie.parse(cookieHeader);
        })
        .client(() => {
            return clientCookie.get();
        }),

    set: createIsomorphicFn()
        .server((name: string, value: string) => {
            return setServerCookie(name, value);
        })
        .client((name: string, value: string) => {
            return clientCookie.set(name, value);
        }),

    getRaw: createIsomorphicFn()
        .server(() => {
            const cookieHeader = getRequestHeader("Cookie");
            if (!cookieHeader) return "";
            return cookieHeader;
        })
        .client(() => {
            return document.cookie;
        }),
};
