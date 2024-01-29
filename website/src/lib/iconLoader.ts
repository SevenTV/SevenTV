async function loadIconAsync(name: string) {
    try {
        const icon = (await import('@fortawesome/pro-regular-svg-icons'))[name];
        if (!icon) {
            throw new Error('Icon not found');
        }
        return icon;
    } catch (e) {
        return (await import('@fortawesome/free-solid-svg-icons'))[name];
    }
}

export function loadIcon(name: string, callback: (icon: any) => void) {
    loadIconAsync(name).then(callback);
}
