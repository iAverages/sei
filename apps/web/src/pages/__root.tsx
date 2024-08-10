import { RouteSectionProps } from "@solidjs/router";
import AuthProvider from "~/components/auth";
import { SolidQueryDevtools } from "@tanstack/solid-query-devtools";

export default function Root(props: RouteSectionProps) {
  return (
    <>
      <AuthProvider>
        <>{props.children}</>
      </AuthProvider>
      <SolidQueryDevtools />
    </>
  );
}
