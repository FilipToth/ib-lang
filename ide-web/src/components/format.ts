/// Numbers as a person reads them. Kept apart from the views that show them so
/// that importing a formatter does not drag a server call in with it.

/// Bytes as a size, rather than a count of bytes.
export const formatBytes = (bytes: number): string => {
    if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    if (bytes >= 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${bytes} B`;
};

/// Groups digits, since six zeroes in a row do not read as a quantity.
export const formatCount = (count: number): string =>
    count.toLocaleString("en-US");
