import { HttpErrorResponse } from '@angular/common/http';

export function parseErrorMessage(error: HttpErrorResponse | Error | string | null | undefined, fallback = 'Operation failed'): string {
    if (!error) {
        return fallback;
    }

    if (typeof error === 'string') {
        return error;
    }

    if (error instanceof HttpErrorResponse) {
        const errObj = error.error;
        if (errObj && typeof errObj === 'object') {
            const message = 'error' in errObj && typeof errObj.error === 'string' ? errObj.error : undefined;
            const code = 'code' in errObj && typeof errObj.code === 'string' ? errObj.code : undefined;
            
            if (message && code) {
                return `${message} [${code}]`;
            }
            if (message) return message;
            if (code) return code;
        }
        return error.message || fallback;
    }

    if (error instanceof Error) {
        return error.message;
    }

    return fallback;
}



