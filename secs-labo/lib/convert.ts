export function numToHexStr(num: number) {
    return num.toString(16).toUpperCase().padStart(2, "0");
}

export function hexStrToNum(str: string) {
    return parseInt(str, 16);
}

export function numToBinaryStr(num: number) {
    return num.toString(2).padStart(8, "0");
}

export function binaryStrToNum(str: string) {
    return parseInt(str, 2);
}

 