import { createFileRoute } from "@tanstack/solid-router";
import LoginButton from "~/components/login-button";

export const Route = createFileRoute("/login")({
    component: RouteComponent,
});

function RouteComponent() {
    return (
        <main class="min-h-screen w-full text-white relative overflow-hidden">
            <section class="relative z-10 flex min-h-screen items-center justify-center px-6">
                <div class="w-full max-w-md">
                    <div class="mb-8 text-center">
                        <h1 class="text-3xl sm:text-4xl font-extrabold tracking-tight leading-tight">Welcome back</h1>
                        <p class="mt-3 text-sm sm:text-base text-white/70">
                            Sign in to continue using your anime tracker
                        </p>
                    </div>

                    <div class="rounded-2xl bg-white/5 backdrop-blur-md border border-white/10 p-6 sm:p-8 shadow-2xl">
                        <LoginButton
                            provider="mal"
                            aria-label="Continue with MyAnimeList"
                            label="Continue with MyAnimeList"
                            class="w-full"
                        />
                    </div>
                </div>
            </section>
        </main>
    );
}
