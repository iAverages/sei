import type { Component } from "solid-js";
import { Router } from "@solidjs/router";
import { ColorModeProvider, ColorModeScript } from "@kobalte/core";

import { routes } from "./routes";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";

const queryClient = new QueryClient({});

const App: Component = () => {
    return (
        <QueryClientProvider client={queryClient}>
            <ColorModeScript />
            <ColorModeProvider>
                <Router>{routes}</Router>
            </ColorModeProvider>
        </QueryClientProvider>
    );
};

export default App;
