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
}
