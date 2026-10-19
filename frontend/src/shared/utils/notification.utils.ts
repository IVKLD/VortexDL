import { MonoTypeOperatorFunction, catchError, pipe, tap, throwError } from 'rxjs';
import { NotificationService } from '@app/services/notification.service';
import { parseErrorMessage } from '@shared/error-utils';

export function withNotification<T>(
    notification: NotificationService,
    successMsg?: string | (() => string) | null,
    errorFallback: string | (() => string) = 'Operation failed'
): MonoTypeOperatorFunction<T> {
    return pipe(
        tap(() => {
            if (!successMsg) return;
            const msg = typeof successMsg === 'function' ? successMsg() : successMsg;
            if (msg) {
                notification.success(msg);
            }
        }),
        catchError((error: unknown) => {
            const fallback = typeof errorFallback === 'function' ? errorFallback() : errorFallback;
            notification.error(parseErrorMessage(error, fallback));
            return throwError(() => error);
        })
    );
}
