import { useNavigate } from "@solidjs/router";
import { createQuery } from "@tanstack/solid-query";
import { TbLoader2 } from "solid-icons/tb";
import { JSX, Show, createEffect } from "solid-js";
import { env } from "~/env.mjs";

export const createUser = () => {
    return createQuery(() => ({
        staleTime: 1000 * 60,
        queryKey: ["user", "me"],
        queryFn: async () => {
            const res = await fetch(`${import.meta.env.VITE_API_URL ?? ""}/api/v1/auth/me`, {
                credentials: "include",
            });

            console.log("res", res);
            if (!res.ok) {
                throw res;
            }
            return res.json();
        },
        retry(failureCount, error) {
            if (!(error instanceof Response)) return true;

            // Don't retry on unauthorized
            if (error.status === 401) return false;

            return failureCount < 3;
        },
    }));
};

const AuthProvider = (props: { children: JSX.Element }) => {
    const user = createUser();
    const nav = useNavigate();

    createEffect(() => {
        if (user.error) {
            nav("/login");
        }
    });

    return (
        <Show
            when={user.data}
            fallback={
                <div class={"w-screen h-screen flex items-center justify-center"}>
                    <TbLoader2 class={"animate-spin w-8  h-8"} />
                </div>
            }>
            <>{props.children}</>
        </Show>
    );
};

export default AuthProvider;
