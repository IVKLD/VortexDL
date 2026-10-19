import { Injectable, NgZone, inject } from '@angular/core';
import { Observable } from 'rxjs';

@Injectable({ providedIn: 'root' })
export class WebSocketService {
    private readonly _zone = inject(NgZone);

    public connect<T>(path: string): Observable<T> {
        return new Observable<T>(sub => {
            let ws: WebSocket;
            let timer: ReturnType<typeof setTimeout>;
            const run = () => {
                ws = new WebSocket(`ws://${window.location.host}${path}`);
                ws.onmessage = e => this._zone.run(() => sub.next(JSON.parse(e.data)));
                ws.onclose = () => timer = setTimeout(run, 3000);
                ws.onerror = () => ws.close();
            };
            run();
            return () => {
                clearTimeout(timer);
                ws.onclose = ws.onerror = ws.onmessage = null;
                ws.close();
            };
        });
    }

    public requestStream<TReq, TRes>(path: string, payload: TReq, expectedCount?: number): Observable<TRes> {
        return new Observable<TRes>(sub => {
            const ws = new WebSocket(`ws://${window.location.host}${path}`);
            let received = 0;

            ws.onopen = () => {
                ws.send(JSON.stringify(payload));
            };

            ws.onmessage = (event) => {
                this._zone.run(() => {
                    try {
                        const result = JSON.parse(event.data) as TRes;
                        sub.next(result);
                        received++;
                        if (expectedCount !== undefined && received >= expectedCount) {
                            ws.close();
                            sub.complete();
                        }
                    } catch (e) {
                        sub.error(e);
                        ws.close();
                    }
                });
            };

            ws.onerror = (err) => {
                this._zone.run(() => sub.error(err));
            };

            ws.onclose = () => {
                this._zone.run(() => {
                    if (expectedCount !== undefined && received < expectedCount) {
                        sub.error(new Error('Connection closed prematurely'));
                    } else {
                        sub.complete();
                    }
                });
            };

            return () => {
                ws.close();
            };
        });
    }
}
