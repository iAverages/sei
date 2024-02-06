import { Button } from "~/components/ui/button";

const Login = () => {
    return (
        <div class={"flex w-screen h-screen items-center justify-center"}>
            <h1 class={"text-3xl font-bold"}>Login</h1>
            <a href="http://localhost:3001/oauth/mal/redirect">
                <Button>Login</Button>
            </a>
        </div>
    );
};

export default Login;
