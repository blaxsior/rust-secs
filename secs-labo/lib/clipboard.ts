
export async function copyToClipboard(text: string) {
    await navigator.clipboard.writeText(text);
}

export async function copyFromClipboard() {
    return await navigator.clipboard.readText();
}