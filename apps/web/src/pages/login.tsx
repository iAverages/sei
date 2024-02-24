import { Button } from "~/components/ui/button";
import { env } from "~/env.mjs";

const Login = () => {
    return (
        <div class={"flex w-screen h-screen items-center justify-center"}>
            <h1 class={"text-3xl font-bold"}>Login</h1>
            <a href={`${import.meta.env.VITE_API_URL ?? ""}/oauth/mal/redirect`}>
                <Button>Login</Button>
            </a>
        </div>
    );
};

export default Login;
