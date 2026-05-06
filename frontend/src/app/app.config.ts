import { ApplicationConfig, provideBrowserGlobalErrorListeners, provideZonelessChangeDetection } from '@angular/core';
import { provideRouter } from '@angular/router';
import { routes } from './app.routes';
import { provideHttpClient, withFetch, withInterceptors } from "@angular/common/http";
import {MAT_ICON_DEFAULT_OPTIONS} from "@angular/material/icon";
import {apiInterceptor} from "@shared/interceptors/api.interceptor";

export const appConfig: ApplicationConfig = {
    providers: [
        {
            provide: MAT_ICON_DEFAULT_OPTIONS,
            useValue: { fontSet: 'material-symbols-rounded' }
        },
        provideBrowserGlobalErrorListeners(),
        provideRouter(routes),
        provideHttpClient(withInterceptors([apiInterceptor]), withFetch()),
        provideZonelessChangeDetection()
    ]
};
