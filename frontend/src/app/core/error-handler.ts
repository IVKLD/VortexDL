import { ErrorHandler, Injectable } from '@angular/core';

@Injectable()
export class GlobalErrorHandler implements ErrorHandler {
    handleError(error: any): void {
        const chunkErrorRegex = /Loading chunk [\d]+ failed/;
        if (chunkErrorRegex.test(error.message)) {
            console.error('Network error: Failed to load a code chunk. Please check your connection or reload the page.');
        }

        // Log the full error to the console. 
        // Modern browsers will use the source maps we just enabled to show the real file/line.
        console.error('------- RUNTIME ERROR -------');
        console.error('Message:', error.message || error);
        if (error.stack) {
            console.error('Stack Trace:', error.stack);
        }
        console.error('Original Error Object:', error);
        console.error('-----------------------------');
    }
}
