import type { Component } from "solid-js";
import { useRoutes } from "@solidjs/router";
import { ColorModeProvider, ColorModeScript } from "@kobalte/core";

import { routes } from "./routes";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import AuthProvider from "~/components/auth";

const queryClient = new QueryClient({});

const App: Component = () => {
    const Route = useRoutes(routes);

    return (
        <>
            <QueryClientProvider client={queryClient}>
                <ColorModeScript />
                <ColorModeProvider>
                    <AuthProvider>
                        <main>
                            <Route />
                        </main>
                    </AuthProvider>
                </ColorModeProvider>
            </QueryClientProvider>
        </>
    );
};

export default App;
