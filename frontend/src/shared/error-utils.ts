import { HttpErrorResponse } from '@angular/common/http';

export function parseErrorMessage(error: any, fallback = 'Operation failed'): string {
    if (!error) {
        return fallback;
    }

    // Extract the raw server error body
    const errObj = error instanceof HttpErrorResponse ? error.error : (error.error || error);

    if (errObj && typeof errObj === 'object') {
        const message = errObj.error;
        const code = errObj.code;
        if (message && code) {
            return `${message} [${code}]`;
        }
        if (message) return message;
        if (code) return code;
    }

    if (typeof errObj === 'string') {
        return errObj;
    }

    return error.message || fallback;
}
