import { lazy } from "solid-js";
import type { RouteDefinition } from "@solidjs/router";

import Login from "~/pages/login";
import Root from "~/pages/__root";

export const routes: RouteDefinition[] = [
  {
    path: "/",
    component: Root,
    children: [
      {
        path: "/",
        component: lazy(() => import("~/pages/home")),
      },
    ],
  },

  {
    path: "/anime/:id",
    component: lazy(() => import("~/pages/view-anime")),
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
