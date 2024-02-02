import { createQuery } from "@tanstack/solid-query";
import { JSX, Show } from "solid-js";

const createUser = () =>
    createQuery(() => ({
        queryKey: ["user"],
        queryFn: async () => {
            return fetch("http://localhost:3001/api/auth/me").then((res) => res.json());
        },
    }));

const AuthProvider = (props: { children: JSX.Element }) => {
    const user = createUser();

    return (
        <Show when={user.data} fallback={<>Loading user...</>}>
            {props.children}
        </Show>
    );
};

export default AuthProvider;
