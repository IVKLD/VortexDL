import {Component, inject, input, signal} from '@angular/core';
import {CommonModule} from '@angular/common';
import {MatIcon} from '@angular/material/icon';
import {MatIconButton} from '@angular/material/button';
import {MatTooltip} from '@angular/material/tooltip';
import {MatDialog} from '@angular/material/dialog';
import {FieldTree} from "@angular/forms/signals";
import {MatDivider} from "@angular/material/list";
import {filter} from "rxjs";
import { HttpErrorResponse } from '@angular/common/http';
import {
    ProxyCheckerDialogComponent
} from "@app/pages/settings-view/sections/network-section/components/fallback-checker-dialog/proxy-checker-dialog.component";
import {NetworkSettings} from "@app/pages/settings-view/models/settings-form.model";
import {SettingsTestingService} from "@app/pages/settings-view/settings.service";
import {parseErrorMessage} from '@shared/error-utils';

@Component({
    selector: 'app-fallback-proxies',
    imports: [
        CommonModule, MatIcon, MatIconButton, MatTooltip, MatDivider
    ],
    templateUrl: './fallback-proxies.component.html',
    styleUrl: './fallback-proxies.component.scss',
    })
export class FallbackProxiesComponent {

    private readonly _testingService = inject(SettingsTestingService);
    private readonly _dialog = inject(MatDialog);
    public readonly form = input.required<FieldTree<NetworkSettings>>();

    public readonly proxyStatuses = signal<Record<string, { loading: boolean; valid?: boolean; error?: string }>>({});
    public readonly isTestingAll = signal<boolean>(false);

    get hasFailedProxies(): boolean {
        const current = this.form().fallbackProxies().value() || [];
        return current.some((p: string) => this.proxyStatuses()[p]?.valid === false);
    }

    remove(proxy: string): void {
        const current = this.form().fallbackProxies().value() || [];
        this.form().fallbackProxies().value.set(current.filter((p: string) => p !== proxy));
        this.form().fallbackProxies().markAsDirty();

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

        dialogRef.afterClosed().pipe(filter((u): u is string[] => u !== undefined)).subscribe((workingProxies: string[]) => {
            if (workingProxies.length > 0) {
                const current = this.form().fallbackProxies().value() || [];
                const next = Array.from(new Set([...current, ...workingProxies]));
                this.form().fallbackProxies().value.set(next);
                this.form().fallbackProxies().markAsDirty();

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

        this._testingService.testProxy([proxy]).subscribe({
            next: (result) => {
                this.proxyStatuses.update(prev => ({
                    ...prev,
                    [proxy]: {loading: false, valid: result.valid, error: result.error || undefined}
                }));
            },
            error: (err: HttpErrorResponse | Error) => {
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
        for (const proxy of proxies) {
            this.proxyStatuses.update(prev => ({
                ...prev,
                [proxy]: {loading: true}
            }));
        }

        this._testingService.testProxy(proxies).subscribe({
            next: (result) => {
                this.proxyStatuses.update(prev => ({
                    ...prev,
                    [result.url]: {
                        loading: false,
                        valid: result.valid,
                        error: result.error || undefined
                    }
                }));
            },
            error: (err: HttpErrorResponse | Error) => {
                const errorDetail = parseErrorMessage(err, 'Verification failed');
                for (const proxy of proxies) {
                    if (this.proxyStatuses()[proxy]?.loading) {
                        this.proxyStatuses.update(prev => ({
                            ...prev,
                            [proxy]: {loading: false, valid: false, error: errorDetail}
                        }));
                    }
                }
                this.isTestingAll.set(false);
            },
            complete: () => {
                this.isTestingAll.set(false);
            }
        });
    }

    pruneFailedActive(): void {
        const current = this.form().fallbackProxies().value() || [];
        const next = current.filter((p: string) => {
            const status = this.proxyStatuses()[p];
            return !status || status.valid !== false;
        });
        this.form().fallbackProxies().value.set(next);
        this.form().fallbackProxies().markAsDirty();
    }

    clearAllActive(): void {
        this.form().fallbackProxies().value.set([]);
        this.form().fallbackProxies().markAsDirty();
        this.proxyStatuses.set({});
    }
}
