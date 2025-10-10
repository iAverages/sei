import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export const cn = (...inputs: ClassValue[]) => {
    return twMerge(clsx(inputs));
};

export const trytm = async <T>(promise: Promise<T>): Promise<[T, null] | [null, Error]> => {
    try {
        const data = await promise;
        return [data, null];
    } catch (throwable) {
        if (throwable instanceof Error) return [null, throwable];

        throw throwable;
    }
};

export const prependSlash = (path: string) => {
    if (!path.startsWith("/")) return `/${path}`;
    return path;
};
