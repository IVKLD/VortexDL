import {HttpInterceptorFn} from '@angular/common/http';
import {retry, timer} from 'rxjs';

export const retryInterceptor: HttpInterceptorFn = (req, next) => {
    return next(req).pipe(
        retry({
            count: 2,
            delay: (error, retryCount) => {
                if (error.status >= 500 || error.status === 0) {
                    const delayMs = retryCount * 1000;
                    console.warn(`HTTP ${error.status} on ${req.url}. Retrying attempt ${retryCount} in ${delayMs}ms...`);
                    return timer(delayMs);
                }

                throw error;
            }
        })
    );
};
