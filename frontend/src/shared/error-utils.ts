import { HttpErrorResponse } from '@angular/common/http';

/**
 * Strips long query parameters (e.g. AWS signed tokens, CloudFront parameters) from URLs inside error strings.
 */
const sanitizeUrls = (message: string): string =>
    message.replace(/https?:\/\/[^\s)"'`]+(?:\?[^\s)"'`]+)?/g, (url) => {
        try {
            const parsed = new URL(url);
            if (parsed.search && parsed.search.length > 20) {
                return `${parsed.origin}${parsed.pathname}`;
            }
            return url;
        } catch {
            const queryIdx = url.indexOf('?');
            return queryIdx !== -1 ? url.substring(0, queryIdx) : url;
        }
    });

/**
 * Cleans technical Rust/HTTP error boilerplate into human-readable messages.
 */
const cleanTechnicalBoilerplate = (message: string): string => {
    let clean = message;

    // HTTP status server/client error (403 Forbidden) for url (...) -> 403 Forbidden
    clean = clean.replace(/HTTP status (?:server|client) error \((\d{3}\s+[^)]+)\)(?:\s+for url\s*\([^)]*\))?/gi, '$1');

    // reqwest::Error { kind: ..., ... } -> Network error ($1)
    clean = clean.replace(/reqwest::Error\s*\{\s*kind:\s*([^,}]+)[^}]*\}/gi, 'Network error ($1)');

    // "Download failed: Download failed: ..." deduplication
    clean = clean.replace(/^(?:Download failed:\s*)+/gi, 'Download failed: ');

    // Anyhow / stack trace prefixes
    clean = clean.replace(/\n\s*caused by:\s*/gi, ' → ');

    return clean.trim();
};

export const parseErrorMessage = (error: unknown, fallback = 'Operation failed'): string => {
    if (!error) {
        return fallback;
    }

    let rawMessage = '';

    if (typeof error === 'string') {
        rawMessage = error;
    } else if (error instanceof HttpErrorResponse) {
        const errObj = error.error;
        if (typeof errObj === 'string' && errObj.trim().length > 0) {
            rawMessage = errObj;
        } else if (errObj && typeof errObj === 'object') {
            const message = ('error' in errObj && typeof errObj.error === 'string' ? errObj.error : undefined)
                || ('message' in errObj && typeof errObj.message === 'string' ? errObj.message : undefined);
            const code = 'code' in errObj && typeof errObj.code === 'string' ? errObj.code : undefined;

            if (message && code) {
                rawMessage = `${message} [${code}]`;
            } else if (message) {
                rawMessage = message;
            } else if (code) {
                rawMessage = code;
            }
        }
        if (!rawMessage) {
            rawMessage = error.message || (error.status ? `HTTP ${error.status} ${error.statusText}` : fallback);
        }
    } else if (error instanceof Error) {
        rawMessage = error.message;
    } else if (typeof error === 'object' && error !== null) {
        if ('message' in error && typeof (error as { message: unknown }).message === 'string') {
            rawMessage = (error as { message: string }).message;
        } else if ('error' in error && typeof (error as { error: unknown }).error === 'string') {
            rawMessage = (error as { error: string }).error;
        } else {
            rawMessage = String(error);
        }
    } else {
        rawMessage = String(error);
    }

    if (!rawMessage || rawMessage === '[object Object]') {
        return fallback;
    }

    const sanitized = sanitizeUrls(rawMessage);
    return cleanTechnicalBoilerplate(sanitized) || fallback;
};
