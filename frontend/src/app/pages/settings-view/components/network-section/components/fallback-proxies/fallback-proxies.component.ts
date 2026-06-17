import {ChangeDetectionStrategy, Component, inject, input, signal} from '@angular/core';
import {CommonModule} from '@angular/common';
import {MatIcon} from '@angular/material/icon';
import {MatIconButton, MatButton} from '@angular/material/button';
import {MatTooltip} from '@angular/material/tooltip';
import {MatDialog} from '@angular/material/dialog';
import {FieldTree} from "@angular/forms/signals";
import {MatDivider} from "@angular/material/list";
import {filter} from "rxjs";
import {
    ProxyCheckerDialogComponent
} from "@app/pages/settings-view/components/network-section/components/fallback-checker-dialog/proxy-checker-dialog.component";
import {NetworkSettings} from "@app/pages/settings-view/models/settings-form.model";
import {SettingsService} from "@app/pages/settings-view/settings.service";
import {parseErrorMessage} from '@shared/error-utils';

@Component({
    selector: 'app-fallback-proxies',
    imports: [
        CommonModule, MatIcon, MatIconButton, MatTooltip, MatButton, MatDivider
    ],
    templateUrl: './fallback-proxies.component.html',
    styleUrl: './fallback-proxies.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class FallbackProxiesComponent {
    public readonly form = input.required<FieldTree<NetworkSettings>>();

    private readonly _api = inject(SettingsService);
    private readonly _dialog = inject(MatDialog);

    public readonly proxyStatuses = signal<Record<string, { loading: boolean; valid?: boolean; error?: string }>>({});
    public readonly isTestingAll = signal<boolean>(false);

    remove(proxy: string): void {
        const current = this.form().fallbackProxies().value() || [];
        this.form().fallbackProxies().value.set(current.filter((p: string) => p !== proxy));

        this.proxyStatuses.update(prev => {
            const next = {...prev};
            delete next[proxy];
            return next;
        });
    }

    openProxyChecker(): void {
        const dialogRef = this._dialog.open(ProxyCheckerDialogComponent, {
            width: '900px',
            maxWidth: '100%',
            autoFocus: 'textarea',
        });

        dialogRef.afterClosed().pipe(filter(u => u === undefined)).subscribe((workingProxies: string[]) => {
            if (workingProxies.length > 0) {
                const current = this.form().fallbackProxies().value() || [];
                const next = Array.from(new Set([...current, ...workingProxies]));
                this.form().fallbackProxies().value.set(next);

                this.proxyStatuses.update(prev => {
                    const updated = {...prev};
                    for (const p of workingProxies) {
                        updated[p] = {loading: false, valid: true};
                    }
                    return updated;
                });
            }
        });
    }

    testIndividualProxy(proxy: string): void {
        this.proxyStatuses.update(prev => ({
            ...prev,
            [proxy]: {loading: true}
        }));

        this._api.testProxy(proxy).subscribe({
            next: () => {
                this.proxyStatuses.update(prev => ({
                    ...prev,
                    [proxy]: {loading: false, valid: true}
                }));
            },
            error: (err: any) => {
                const errorDetail = parseErrorMessage(err, 'Verification failed');
                this.proxyStatuses.update(prev => ({
                    ...prev,
                    [proxy]: {loading: false, valid: false, error: errorDetail}
                }));
            }
        });
    }

    testAllProxies(): void {
        const proxies = this.form().fallbackProxies().value() || [];
        if (proxies.length === 0) return;

        this.isTestingAll.set(true);
        let completed = 0;

        for (const proxy of proxies) {
            this.proxyStatuses.update(prev => ({
                ...prev,
                [proxy]: {loading: true}
            }));

            this._api.testProxy(proxy).subscribe({
                next: () => {
                    this.proxyStatuses.update(prev => ({
                        ...prev,
                        [proxy]: {loading: false, valid: true}
                    }));
                    completed++;
                    if (completed === proxies.length) this.isTestingAll.set(false);
                },
                error: (err: any) => {
                    const errorDetail = parseErrorMessage(err, 'Verification failed');
                    this.proxyStatuses.update(prev => ({
                        ...prev,
                        [proxy]: {loading: false, valid: false, error: errorDetail}
                    }));
                    completed++;
                    if (completed === proxies.length) this.isTestingAll.set(false);
                }
            });
        }
    }

    pruneFailedActive(): void {
        const current = this.form().fallbackProxies().value() || [];
        const next = current.filter((p: string) => {
            const status = this.proxyStatuses()[p];
            return !status || status.valid !== false;
        });
        this.form().fallbackProxies().value.set(next);
    }

    clearAllActive(): void {
        this.form().fallbackProxies().value.set([]);
        this.proxyStatuses.set({});
    }
}
