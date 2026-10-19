import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { NotificationService } from '@app/services/notification.service';
import { WebSocketService } from '@app/services/websocket.service';
import { Observable, tap } from 'rxjs';
import { UserSettingsRdo } from './models/user-settings.rdo';
import { UserSettingsDto } from './models/user-settings.dto';
import { ProxyTestResultRdo } from './models/proxy-test.rdo';
import { withNotification } from '@shared/utils/notification.utils';
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
            withNotification(this._notification, 'Settings updated', 'Failed to update settings')
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
            withNotification(
                this._notification,
                `Device ${deviceId} synced successfully`,
                `Failed to sync device ${deviceId}`
            )
        );
    }

    public scanDevice(deviceId: string) {
        return this._http.post(`/devices/${deviceId}/scan`, {}).pipe(
            withNotification(
                this._notification,
                `Device ${deviceId} music indexing started`,
                `Failed to trigger music indexing on ${deviceId}`
            )
        );
    }

    public syncBackup(action: SyncAction, provider: SyncProviderPayload) {
        const actionLabel = action === SyncAction.Export ? 'exported' : 'imported';
        const failActionLabel = action === SyncAction.Export ? 'export' : 'import';

        return this._http.post('/settings/sync', {
            action,
            provider,
        }).pipe(
            withNotification(
                this._notification,
                `Database snapshot ${actionLabel} successfully`,
                `Failed to ${failActionLabel} backup`
            )
        );
    }

    public getSyncSnapshot() {
        return this._http.get<SyncSnapshotResponse>('/settings/sync/snapshot');
    }
}

@Injectable({ providedIn: 'root' })
export class SettingsTestingService {
    private readonly _http = inject(HttpClient);
    private readonly _notification = inject(NotificationService);
    private readonly _ws = inject(WebSocketService);

    public testSoundCloud(url: string) {
        return this._http.post<string>('/settings/test/soundcloud', { url }).pipe(
            withNotification(this._notification, 'SoundCloud URL is valid', 'Invalid configuration')
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
            withNotification(this._notification, null, 'Proxy verification failed')
        );
    }

    public testProxy(proxyUrls: string[]): Observable<ProxyTestResultRdo> {
        return this._ws.requestStream('/api/settings/test/proxy/ws', proxyUrls, proxyUrls.length);
    }
}