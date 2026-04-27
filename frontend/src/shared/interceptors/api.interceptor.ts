import {HttpInterceptorFn} from "@angular/common/http";

export const apiInterceptor: HttpInterceptorFn = (req, next) => {
    const url = `${process.env.NG_APP_API_URL}/${req.url}`.replace(/\/+/g, '/').replace(':/', '://');

    return next(req.clone({url}));
};