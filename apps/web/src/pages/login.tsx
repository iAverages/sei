import { Button } from "~/components/ui/button";
import { useMalLogin } from "~/hooks/useMalLogin";

const Login = () => {
    const { isLoading, loginError, openDiscordLogin } = useMalLogin({
        redirect: "/",
    });

    return (
        <div class={"flex w-screen h-screen items-center justify-center"}>
            <h1 class={"text-3xl font-bold"}>Login</h1>
            <form onSubmit={openDiscordLogin}>
                <Button>Login</Button>
            </form>
        </div>
    );
};

export default Login;
