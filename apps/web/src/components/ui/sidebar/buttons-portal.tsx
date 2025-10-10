import { type Accessor, createContext, type JSXElement, useContext } from "solid-js";
import { Portal } from "solid-js/web";

export type HeaderButtonsArea = HTMLDivElement | null;
export type HeaderButtonsAreaSetter = (element: HTMLElement) => void;
export type HeaderButtonsContext = {
    ref: Accessor<HeaderButtonsArea>;
    setRef: HeaderButtonsAreaSetter;
};

export const HeaderButtonsContext = createContext<HeaderButtonsContext>({
    ref: () => null,
    setRef: () => {},
});

export const HeaderButtonsProvider = (props: {
    children: JSXElement;
    buttonsAreaRef: Accessor<HeaderButtonsArea>;
    setButtonsAreaRef: (element: HTMLElement) => void;
}) => {
    return (
        <HeaderButtonsContext.Provider value={{ ref: props.buttonsAreaRef, setRef: props.setButtonsAreaRef }}>
            {props.children}
        </HeaderButtonsContext.Provider>
    );
};

export const HeaderButtonsPortal = (props: { children: JSXElement }) => {
    const context = useContext(HeaderButtonsContext);
    return <Portal mount={context.ref()!}>{props.children}</Portal>;
};
