import {ErrorHandler, Injectable} from '@angular/core';

@Injectable()
export class GlobalErrorHandler implements ErrorHandler {
    public handleError(error: Error | string | null | undefined): void {
        const message = error instanceof Error ? error.message : String(error);
        const stack = error instanceof Error ? error.stack : undefined;

        const chunkErrorRegex = /Loading chunk [\d]+ failed/;
        if (chunkErrorRegex.test(message)) {
            console.error('Network error: Failed to load a code chunk. Please check your connection or reload the page.');
        }

        console.error('------- RUNTIME ERROR -------');
        console.error('Message:', message);
        if (stack) {
            console.error('Stack Trace:', stack);
        }
        console.error('Original Error Object:', error);
        console.error('-----------------------------');
    }
}


