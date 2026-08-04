import React, { useState } from "react";

type UseFocusOptions = {
    onBlur?: () => void;
    onClick?: () => void;
    onFocus?: () => void;
};

function useFocus(target: React.RefObject<HTMLElement>, options?: UseFocusOptions) {
    const [isFocused, setFocus] = useState(false);

    const clickSelfHandler = () => {
        target.current?.focus();
        options?.onClick?.();
    }

    const focusSelfHandler = () => {
        setFocus(true);
        target.current?.focus();
        options?.onFocus?.();
    }

    const blurSelfHandler = () => {
        setFocus(false);
        options?.onBlur?.();

    }

    return {
        isFocused,
        click: clickSelfHandler,
        focus: focusSelfHandler,
        blur: blurSelfHandler,
    }
}

export {
    useFocus
}