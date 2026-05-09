import { HttpInterceptorFn } from '@angular/common/http';

export const apiInterceptor: HttpInterceptorFn = (req, next) => {
    if (req.url.startsWith('http')) {
        return next(req);
    }

    const path = req.url.startsWith('/') ? req.url.substring(1) : req.url;
    const url = `/api/${path}`;

    return next(req.clone({ url }));
};
