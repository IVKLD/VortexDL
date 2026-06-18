import { ChangeDetectionStrategy, Component, inject, signal, computed } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';
import { CommonModule } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatButton, MatIconButton } from '@angular/material/button';
import { MatDialogRef, MatDialogClose, MatDialogContent, MatDialogTitle, MatDialogActions } from '@angular/material/dialog';
import { MatFormField, MatLabel, MatHint } from '@angular/material/form-field';
import { MatInput } from '@angular/material/input';
import { MatTooltip } from '@angular/material/tooltip';
import {SettingsService} from "@app/pages/settings-view/settings.service";
import {parseErrorMessage} from '@shared/error-utils';

@Component({
    selector: 'app-proxy-checker-dialog',
    imports: [
        CommonModule, MatIcon, MatButton, MatIconButton, MatInput, MatFormField,
        MatLabel, MatHint, MatDialogTitle, MatDialogContent, MatDialogActions, MatDialogClose,
        MatTooltip
    ],
    templateUrl: './proxy-checker-dialog.component.html',
    styleUrl: './proxy-checker-dialog.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ProxyCheckerDialogComponent {
    private readonly _api = inject(SettingsService);
    private readonly _dialogRef = inject(MatDialogRef<ProxyCheckerDialogComponent>);

    public readonly pastedText = signal<string>('');
    
    public readonly parsedProxies = computed(() => {
        return this.parseProxies(this.pastedText());
    });

    public readonly importStatuses = signal<Record<string, { loading: boolean; valid?: boolean; error?: string }>>({});
    public readonly workingProxies = signal<string[]>([]);
    public readonly isTesting = signal<boolean>(false);

    onTextChange(event: Event): void {
        const target = event.target;
        if (target instanceof HTMLTextAreaElement) {
            this.pastedText.set(target.value);
        }
    }

    private parseProxies(text: string): string[] {
        if (!text) return [];
        const lines = text.split(/[\n,;]+/).map(l => l.trim()).filter(l => l.length > 0);
        const results: string[] = [];

        for (const line of lines) {
            if (/^(http|https|socks4|socks4a|socks5|socks5h):\/\//i.test(line)) {
                results.push(line);
            }
        }
        return Array.from(new Set(results));
    }

    testAll(): void {
        const proxies = this.parsedProxies();
        if (proxies.length === 0) return;

        this.isTesting.set(true);
        this.importStatuses.set({});
        this.workingProxies.set([]);
        let completed = 0;

        for (const proxy of proxies) {
            this.importStatuses.update(prev => ({
                ...prev,
                [proxy]: { loading: true }
            }));

            this._api.testProxy(proxy).subscribe({
                next: () => {
                    this.importStatuses.update(prev => ({
                        ...prev,
                        [proxy]: { loading: false, valid: true }
                    }));
                    this.workingProxies.update(prev => [...prev, proxy]);
                    completed++;
                    if (completed === proxies.length) this.isTesting.set(false);
                },
                error: (err: HttpErrorResponse | Error) => {
                    const errorDetail = parseErrorMessage(err, 'Verification failed');
                    this.importStatuses.update(prev => ({
                        ...prev,
                        [proxy]: { loading: false, valid: false, error: errorDetail }
                    }));
                    completed++;
                    if (completed === proxies.length) this.isTesting.set(false);
                }
            });
        }
    }

    copyToClipboard(): void {
        const text = this.workingProxies().join('\n');
        navigator.clipboard.writeText(text);
    }

    importWorking(): void {
        this._dialogRef.close(this.workingProxies());
    }
}
