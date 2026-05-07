import {HttpInterceptorFn} from "@angular/common/http";

export const apiInterceptor: HttpInterceptorFn = (req, next) => {
    const url = `/api/${req.url}`.replace(/\/+/g, '/').replace(':/', '://');

    return next(req.clone({url}));
};