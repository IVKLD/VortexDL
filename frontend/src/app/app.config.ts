import {
    ApplicationConfig,
    ErrorHandler,
    provideBrowserGlobalErrorListeners,
    provideZonelessChangeDetection
} from '@angular/core';
import {GlobalErrorHandler} from '@shared/error-handler';
import {provideRouter} from '@angular/router';
import {routes} from './app.routes';
import {provideHttpClient, withFetch, withInterceptors} from '@angular/common/http';
import {MAT_ICON_DEFAULT_OPTIONS} from '@angular/material/icon';
import {MAT_FORM_FIELD_DEFAULT_OPTIONS} from '@angular/material/form-field';
import {MAT_SNACK_BAR_DEFAULT_OPTIONS} from '@angular/material/snack-bar';
import {apiInterceptor} from '@shared/interceptors/api.interceptor';
import {retryInterceptor} from '@shared/interceptors/retry.interceptor';

export const appConfig: ApplicationConfig = {
    providers: [
        {
            provide: MAT_ICON_DEFAULT_OPTIONS,
            useValue: {fontSet: 'material-symbols-rounded'},
        },
        {
            provide: MAT_FORM_FIELD_DEFAULT_OPTIONS,
            useValue: {appearance: 'outline'},
        },
        {
            provide: MAT_SNACK_BAR_DEFAULT_OPTIONS,
            useValue: {duration: 4000},
        },
        {
            provide: ErrorHandler,
            useClass: GlobalErrorHandler,
        },
        provideBrowserGlobalErrorListeners(),
        provideRouter(routes),
        provideHttpClient(withInterceptors([retryInterceptor, apiInterceptor]), withFetch()),
        provideZonelessChangeDetection(),
    ],
};
