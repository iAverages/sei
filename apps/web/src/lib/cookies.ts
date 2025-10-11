import { createIsomorphicFn } from "@tanstack/solid-start";
import { getRequestHeader, setCookie as setServerCookie } from "@tanstack/solid-start/server";
import serverCookie, { type SerializeOptions } from "cookie"; // used for parsing cookie on the server
import clientCookie from "js-cookie"; // used for setting cookie on the client
import { createSignal } from "solid-js";
import { z } from "zod/v4";

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
        .server((name: string, value: string, attrs?: SerializeOptions) => {
            return setServerCookie(name, value, attrs);
        })
        .client((name: string, value: string, attrs?: SerializeOptions) => {
            return clientCookie.set(
                name,
                value,
                attrs
                    ? {
                          ...attrs,
                          sameSite:
                              typeof attrs.sameSite === "boolean"
                                  ? attrs.sameSite
                                      ? "strict"
                                      : "none"
                                  : attrs.sameSite,
                      }
                    : undefined,
            );
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

export const useCookie = (cookieName: string, defaultState?: string) => {
    const allCookies = Cookies.getAll();
    const [value, _setValue] = createSignal(allCookies?.[cookieName] ?? defaultState);

    const setValue = (value: string | (() => string)) => {
        const newValue = typeof value === "string" ? value : value();
        Cookies.set(cookieName, newValue);
        _setValue(newValue);
    };

    return [value, setValue] as const;
};
