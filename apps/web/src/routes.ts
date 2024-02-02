import { lazy } from "solid-js";
import type { RouteDefinition } from "@solidjs/router";

import Home from "./pages/home";
import Login from "~/pages/login";

export const routes: RouteDefinition[] = [
    {
        path: "/",
        component: Home,
    },
    {
        path: "/login",
        component: Login,
    },
    {
        path: "**",
        component: lazy(() => import("./errors/404")),
    },
];
