import {
    ApplicationConfig,
    ErrorHandler,
    provideBrowserGlobalErrorListeners,
    provideZonelessChangeDetection
} from '@angular/core';
import {GlobalErrorHandler} from '@shared/error-handler';
import {provideRouter} from '@angular/router';
import {routes} from './app.routes';
import {provideHttpClient, withInterceptors} from '@angular/common/http';
import {MAT_ICON_DEFAULT_OPTIONS} from '@angular/material/icon';
import {MAT_FORM_FIELD_DEFAULT_OPTIONS} from '@angular/material/form-field';
import {MAT_SNACK_BAR_DEFAULT_OPTIONS} from '@angular/material/snack-bar';
import {apiInterceptor} from '@shared/interceptors/api.interceptor';
import {retryInterceptor} from '@shared/interceptors/retry.interceptor';
import {loadingInterceptor} from '@shared/interceptors/loading.interceptor';
import {MAT_DIALOG_SCROLL_STRATEGY} from '@angular/material/dialog';
import {StickyAwareScrollStrategy} from '@shared/services/scroll-strategy';
import {MAT_SELECT_SCROLL_STRATEGY} from "@angular/material/select";
import {MAT_MENU_SCROLL_STRATEGY} from "@angular/material/menu";
import {MAT_AUTOCOMPLETE_SCROLL_STRATEGY} from "@angular/material/autocomplete";
import {MAT_BOTTOM_SHEET_DEFAULT_OPTIONS} from "@angular/material/bottom-sheet";

const provideInertScroll = (strategy: StickyAwareScrollStrategy) => () =>
    strategy;

export const scrollStrategyProviders = [
    {
        provide: MAT_DIALOG_SCROLL_STRATEGY,
        useFactory: provideInertScroll,
        deps: [StickyAwareScrollStrategy],
    },
    {
        provide: MAT_SELECT_SCROLL_STRATEGY,
        useFactory: provideInertScroll,
        deps: [StickyAwareScrollStrategy],
    },
    {
        provide: MAT_MENU_SCROLL_STRATEGY,
        useFactory: provideInertScroll,
        deps: [StickyAwareScrollStrategy],
    },
    {
        provide: MAT_AUTOCOMPLETE_SCROLL_STRATEGY,
        useFactory: provideInertScroll,
        deps: [StickyAwareScrollStrategy],
    },
    {
        provide: MAT_BOTTOM_SHEET_DEFAULT_OPTIONS,
        useFactory: (s: StickyAwareScrollStrategy) => ({
            scrollStrategy: s,
        }),
        deps: [StickyAwareScrollStrategy],
    },
];

export const appConfig: ApplicationConfig = {
    providers: [
        scrollStrategyProviders,
        {
            provide: MAT_ICON_DEFAULT_OPTIONS,
            useValue: { fontSet: 'material-symbols-rounded' },
        },
        {
            provide: MAT_FORM_FIELD_DEFAULT_OPTIONS,
            useValue: { appearance: 'outline', subscriptSizing: 'dynamic' },
        },
        {
            provide: MAT_SNACK_BAR_DEFAULT_OPTIONS,
            useValue: { duration: 4000 },
        },
        {
            provide: ErrorHandler,
            useClass: GlobalErrorHandler,
        },
        provideBrowserGlobalErrorListeners(),
        provideRouter(routes),
        provideHttpClient(withInterceptors([retryInterceptor, apiInterceptor, loadingInterceptor])),
        provideZonelessChangeDetection(),
    ],
};
