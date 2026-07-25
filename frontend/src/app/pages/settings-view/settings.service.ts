import { HttpClient } from '@angular/common/http';
import { inject, Injectable, NgZone } from '@angular/core';
import { NotificationService } from '@app/services/notification.service';
import { catchError, Observable, tap, throwError } from 'rxjs';
import { UserSettingsRdo } from './models/user-settings.rdo';
import { UserSettingsDto } from './models/user-settings.dto';
import { ProxyTestResultRdo } from './models/proxy-test.rdo';
import { parseErrorMessage } from '@shared/error-utils';
import { StorageInfo } from './models/settings-form.model';

@Injectable({ providedIn: 'root' })
export class SettingsService {
    private readonly _http = inject(HttpClient);
    private readonly _notification = inject(NotificationService);

    public getSettings() {
        return this._http.get<UserSettingsRdo>('/settings');
    }

    public updateSettings(userSettings: UserSettingsDto) {
        return this._http.post('/settings', userSettings).pipe(
            tap(() => this._notification.success('Settings updated')),
            catchError((error) => {
                this._notification.error(parseErrorMessage(error, 'Failed to update settings'));
                return throwError(() => error);
            })
        );
    }

    public getAdbDevices() {
        return this._http.get<string[]>('/devices');
    }

    public getDeviceStorages(deviceId: string) {
        return this._http.get<StorageInfo[]>(`/devices/${deviceId}/storage`);
    }

    public syncDevice(deviceId: string) {
        return this._http.post(`/devices/${deviceId}/sync`, {});
    }
}

@Injectable({ providedIn: 'root' })
export class SettingsTestingService {
    private readonly _http = inject(HttpClient);
    private readonly _notification = inject(NotificationService);
    private readonly _zone = inject(NgZone);

    public testSoundCloud(url: string) {
        return this._http.post<string>('/settings/test/soundcloud', { url }).pipe(
            tap(() => this._notification.success('SoundCloud URL is valid')),
            catchError((error) => {
                this._notification.error(parseErrorMessage(error, 'Invalid configuration'));
                return throwError(() => error);
            })
        );
    }

    public testProxy(proxyUrls: string[]): Observable<ProxyTestResultRdo> {
        return new Observable<ProxyTestResultRdo>(sub => {
            const ws = new WebSocket(`ws://${window.location.host}/api/settings/test/proxy/ws`);
            let receivedCount = 0;

            ws.onopen = () => {
                ws.send(JSON.stringify(proxyUrls));
            };

            ws.onmessage = (event) => {
                this._zone.run(() => {
                    try {
                        const result = JSON.parse(event.data) as ProxyTestResultRdo;
                        sub.next(result);
                        receivedCount++;
                        if (receivedCount === proxyUrls.length) {
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
                    if (receivedCount < proxyUrls.length) {
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