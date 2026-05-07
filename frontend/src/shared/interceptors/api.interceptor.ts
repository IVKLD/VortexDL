import {HttpInterceptorFn} from "@angular/common/http";

export const apiInterceptor: HttpInterceptorFn = (req, next) => {
    // Если запрос уже содержит http:// или https://, не трогаем его
    if (req.url.startsWith('http')) {
        return next(req);
    }

    // Убеждаемся, что относительный путь не начинается со слеша перед склейкой
    const path = req.url.startsWith('/') ? req.url.substring(1) : req.url;
    const url = `/api/${path}`;

    return next(req.clone({ url }));
};