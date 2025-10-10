import type { JSX } from "solid-js";
import { Button } from "~/components/ui/button";
import { cn } from "~/lib/utils";

type Provider = "mal"; //| "anilist";

export type LoginButtonProps = JSX.ButtonHTMLAttributes<HTMLButtonElement> & {
    provider: Provider;
    fullWidth?: boolean;
    size?: "sm" | "default" | "lg";
    label?: string;
};

const providerStyles: Record<Provider, string> = {
    mal: "bg-[#2E51A2] hover:bg-[#2E51A2]/90 text-white focus-visible:ring-[#2E51A2]",
    // anilist: "bg-[#02A9FF] hover:bg-[#02A9FF]/90 text-white focus-visible:ring-[#02A9FF]",
};

export const LoginButton = (props: LoginButtonProps) => {
    return (
        <Button
            {...props}
            size={props.size}
            as={"a"}
            href={`/oauth/${props.provider}/redirect`}
            class={cn(
                "font-semibold tracking-tight shadow-lg transition-transform active:scale-[0.98] cursor-pointer",
                "rounded-xl px-6 py-6",
                providerStyles[props.provider],
                props.class,
            )}
        >
            {props.label}
        </Button>
    );
};

export default LoginButton;
