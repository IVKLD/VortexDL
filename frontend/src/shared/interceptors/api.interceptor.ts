import {HttpInterceptorFn} from "@angular/common/http";

export const apiInterceptor: HttpInterceptorFn = (req, next) => {
    const baseUrl = typeof process !== 'undefined' && process.env['NG_APP_API_URL'] ? process.env['NG_APP_API_URL'] : '';
    const url = `${baseUrl}/${req.url}`.replace(/\/+/g, '/').replace(':/', '://');

    return next(req.clone({url}));
};