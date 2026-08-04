export const validateInt = (value: string) => {
    return /^-?\d+$/.test(value);
}

export const validateFloat = (value: string) => {
    return /^-?\d+(\.\d*)?$/.test(value);
}