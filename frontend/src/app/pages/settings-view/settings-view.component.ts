import { ChangeDetectionStrategy, Component, inject, OnInit, signal } from '@angular/core';
import { FormRoot } from '@angular/forms/signals';
import { SettingsService, SettingsTestingService } from './settings.service';
import { SettingsState } from './settings.state';
import { NotificationService } from '@app/services/notification.service';
import { finalize } from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { parseErrorMessage } from '@shared/error-utils';
import { WebSocketService } from '@app/services/websocket.service';
import { SoundcloudSectionComponent } from './sections/soundcloud-section/soundcloud-section.component';
import { DownloadsSectionComponent } from './sections/downloads-section/downloads-section.component';
import { NetworkSectionComponent } from './sections/network-section/network-section.component';
import { AdbSectionComponent } from './sections/adb-section/adb-section.component';

@Component({
    selector: 'app-settings-view',
    imports: [
        FormRoot,
        SoundcloudSectionComponent,
        DownloadsSectionComponent,
        NetworkSectionComponent,
        AdbSectionComponent
    ],
    templateUrl: './settings-view.component.html',
    styleUrl: './settings-view.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SettingsView implements OnInit {
    private readonly _api = inject(SettingsService);
    private readonly _testing = inject(SettingsTestingService);
    private readonly _notification = inject(NotificationService);
    private readonly _ws = inject(WebSocketService);

    protected readonly state = inject(SettingsState);

    protected readonly isTesting = signal(false);
    protected readonly isNetworkTesting = signal(false);
    protected readonly connectedDevices = signal<string[]>([]);
    protected readonly isRefreshing = signal(false);

    protected readonly soundcloudUrlTestError = signal<string | null>(null);
    protected readonly proxyUrlTestError = signal<string | null>(null);

    constructor() {
        this.refreshDevices();

        this._ws
            .connect<string[]>('/api/devices/ws')
            .pipe(takeUntilDestroyed())
            .subscribe({
                next: (devices) => this.updateConnectedDevices(devices),
                error: (err) => console.error('ADB devices WebSocket error:', err),
            });
    }

    private updateConnectedDevices(devices: string[]): void {
        this.connectedDevices.set(devices);

        const current = [...this.state.settingsForm.adb.devices().value()];
        let changed = false;

        for (const id of devices) {
            if (!current.some((d) => d.deviceId === id)) {
                current.push({
                    deviceId: id,
                    remoteMusicDir: '/storage/Music',
                    enabled: true,
                });
                changed = true;
            }
        }

        if (changed) {
            this.state.settingsForm.adb.devices().value.set(current);
            this.state.settingsForm.adb.devices().markAsDirty();
        }
    }

    protected testSoundcloud(): void {
        this.isTesting.set(true);

        this._testing
            .testSoundCloud(this.state.settingsForm.soundcloud.profileUrl().value())
            .pipe(
                finalize(() => {
                    this.isTesting.set(false);
                    this.state.settingsForm.soundcloud.profileUrl().reset();
                    this.state.settingsForm.soundcloud.profileUrl().reloadValidation();
                })
            )
            .subscribe({
                next: () => this.soundcloudUrlTestError.set(null),
                error: (err) => {
                    const errMsg = parseErrorMessage(err, 'Invalid SoundCloud configuration');
                    this.soundcloudUrlTestError.set(errMsg);
                },
            });
    }

    protected testProxy(): void {
        this.isNetworkTesting.set(true);
        const proxy = this.state.settingsForm.network.proxyUrl().value();

        this._testing
            .testProxy([proxy])
            .pipe(
                finalize(() => {
                    this.isNetworkTesting.set(false);
                    this.state.settingsForm.network.proxyUrl().reset();
                    this.state.settingsForm.network.proxyUrl().reloadValidation();
                })
            )
            .subscribe({
                next: (res) => {
                    if (res.valid) {
                        this.proxyUrlTestError.set(null);
                        this._notification.success('Proxy connection successful');
                    } else {
                        const err = res.error || 'Proxy is not able to reach SoundCloud API';
                        this.proxyUrlTestError.set(err);
                        this._notification.error(err);
                    }
                },
                error: (err) => {
                    const errMsg = parseErrorMessage(err, 'Proxy verification failed');
                    this.proxyUrlTestError.set(errMsg);
                    this._notification.error(errMsg);
                },
            });
    }

    protected refreshDevices(): void {
        this.isRefreshing.set(true);
        this._api
            .getAdbDevices()
            .pipe(finalize(() => this.isRefreshing.set(false)))
            .subscribe({
                next: (devices) => this.updateConnectedDevices(devices),
                error: (err) => console.error('Failed to get connected ADB devices:', err),
            });
    }

    protected removeDevice(index: number): void {
        this.state.settingsForm.adb.devices().value.update((current) => current.filter((_, i) => i !== index));
        this.state.settingsForm.adb.devices().markAsDirty();
    }

    public ngOnInit(): void {
        this.state.loadSettings();
    }
}
