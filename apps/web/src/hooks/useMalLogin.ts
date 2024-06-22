import { useNavigate } from "@solidjs/router";
import { createSignal } from "solid-js";
// import { DISCORD_LOGIN_URL } from "~/api";
// import { useUser } from "~/hooks/useUser";

type MalLoginProps = {
    redirect?: string;
    errorRedirect?: string;
};

export const useMalLogin = (props?: MalLoginProps) => {
    const [loginError, setLoginError] = createSignal<string>();
    const [isLoggingIn, setIsLoggingIn] = createSignal(false);
    // const user = useUser();
    const nav = useNavigate();

    // Pass event so it will auto cancell if aviable
    const openDiscordLogin = (e?: Event) => {
        if (isLoggingIn()) return;
        e?.preventDefault();
        setIsLoggingIn(true);

        // if (user.data) {
        //     nav(props.redirect);
        //     return;
        // }

        const width = 600;
        const height = 800;
        const left = window.screen.width / 2 - width / 2;
        const top = window.screen.height / 2 - height / 2;

        const loginWindow = window.open(
            `${import.meta.env.PUBLIC_API_URL ?? ""}/oauth/mal/redirect`,
            "_blank",
            `popup, width=${width}, height=${height}, top=${top}, left=${left}`
        );

        const interval = setInterval(async () => {
            if (loginWindow.closed) {
                clearInterval(interval);
                try {
                    // const { data } = await user.refetch();
                    // if (!data) {
                    //     setLoginError("Failed to login, please try again.");
                    //     if (props.errorRedirect) nav(props.errorRedirect);
                    // } else {
                    //     if (props.redirect) nav(props.redirect);
                    // }
                } catch (e) {
                    console.log("Faield to refetch user login after window closed", e);
                    if (props.errorRedirect) nav(props.errorRedirect);
                }
                setIsLoggingIn(false);
            }
        }, 500);
    };

    return {
        loginError,
        openDiscordLogin,
        isLoading: isLoggingIn,
        isError: () => !!loginError(),
    };
};
