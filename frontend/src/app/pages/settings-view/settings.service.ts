import { HttpClient } from '@angular/common/http';
import { inject, Injectable, NgZone } from '@angular/core';
import { NotificationService } from '@app/services/notification.service';
import { catchError, Observable, tap, throwError } from 'rxjs';
import { UserSettingsRdo } from './models/user-settings.rdo';
import { UserSettingsDto } from './models/user-settings.dto';
import { ProxyTestResultRdo } from './models/proxy-test.rdo';
import { parseErrorMessage } from '@shared/error-utils';
import { WebDavSettings } from './models/settings-form.model';
import { StorageInfo } from './models/adb-storage.model';
import { SyncAction, SyncProviderPayload, SyncSnapshotResponse } from './models/sync.model';

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
        return this._http.post(`/devices/${deviceId}/sync`, {}).pipe(
            tap(() => this._notification.success(`Device ${deviceId} synced successfully`)),
            catchError((error) => {
                this._notification.error(parseErrorMessage(error, `Failed to sync device ${deviceId}`));
                return throwError(() => error);
            })
        );
    }

    public syncBackup(action: SyncAction, provider: SyncProviderPayload) {
        const actionLabel = action === SyncAction.Export ? 'exported' : 'imported';
        const failActionLabel = action === SyncAction.Export ? 'export' : 'import';

        return this._http.post('/settings/sync', {
            action,
            provider,
        }).pipe(
            tap(() => this._notification.success(`Database snapshot ${actionLabel} successfully`)),
            catchError((error) => {
                this._notification.error(parseErrorMessage(error, `Failed to ${failActionLabel} backup`));
                return throwError(() => error);
            })
        );
    }

    public syncWebDav(action: SyncAction, webdav: WebDavSettings) {
        return this.syncBackup(action, {
            webDav: {
                baseUrl: webdav.baseUrl,
                remoteDir: webdav.remoteDir,
                username: webdav.username,
                password: webdav.password,
            },
        });
    }

    public getSyncSnapshot() {
        return this._http.get<SyncSnapshotResponse>('/settings/sync/snapshot');
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

    public testSingleProxy(proxyUrl: string): Observable<ProxyTestResultRdo> {
        return this.testProxy([proxyUrl]).pipe(
            tap((res) => {
                if (res.valid) {
                    this._notification.success('Proxy connection successful');
                } else {
                    const err = res.error || 'Proxy is not able to reach SoundCloud API';
                    this._notification.error(err);
                }
            }),
            catchError((error) => {
                this._notification.error(parseErrorMessage(error, 'Proxy verification failed'));
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