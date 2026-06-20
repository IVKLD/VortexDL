import {ChangeDetectionStrategy, Component, inject, signal, computed} from '@angular/core';
import {HttpErrorResponse} from '@angular/common/http';
import {CommonModule} from '@angular/common';
import {MatIcon} from '@angular/material/icon';
import {MatButton} from '@angular/material/button';
import {
    MatDialogClose,
    MatDialogContent,
    MatDialogTitle,
    MatDialogActions
} from '@angular/material/dialog';
import {MatFormField, MatHint} from '@angular/material/form-field';
import {MatInput} from '@angular/material/input';
import {MatTooltip} from '@angular/material/tooltip';
import {SettingsTestingService} from "@app/pages/settings-view/settings.service";
import {parseErrorMessage} from '@shared/error-utils';

@Component({
    selector: 'app-proxy-checker-dialog',
    imports: [
        CommonModule, MatIcon, MatButton, MatInput, MatFormField,
        MatHint, MatDialogTitle, MatDialogContent, MatDialogActions, MatDialogClose,
        MatTooltip
    ],
    templateUrl: './proxy-checker-dialog.component.html',
    styleUrl: './proxy-checker-dialog.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ProxyCheckerDialogComponent {
    private readonly _testing = inject(SettingsTestingService);

    public readonly pastedText = signal<string>('');

    public readonly parsedProxies = computed(() => {
        return parseProxies(this.pastedText());
    });

    protected readonly importStatuses = signal<Record<string, {
        loading: boolean;
        valid?: boolean;
        error?: string
    }>>({});
    protected readonly workingProxies = signal<string[]>([]);
    protected readonly isTesting = signal<boolean>(false);

    protected onTextChange(event: Event): void {
        const target = event.target;
        if (target instanceof HTMLTextAreaElement) {
            this.pastedText.set(target.value);
        }
    }

    protected testAll(): void {
        const proxies = this.parsedProxies();
        if (proxies.length === 0) return;

        this.isTesting.set(true);
        this.importStatuses.set({});
        this.workingProxies.set([]);

        for (const proxy of proxies) {
            this.importStatuses.update(prev => ({
                ...prev,
                [proxy]: {loading: true}
            }));
        }

        this._testing.testProxy(proxies).subscribe({
            next: (response) => {
                for (const result of response.results) {
                    this.importStatuses.update(prev => ({
                        ...prev,
                        [result.url]: {
                            loading: false,
                            valid: result.valid,
                            error: result.error || undefined
                        }
                    }));
                    if (result.valid) {
                        this.workingProxies.update(prev => [...prev, result.url]);
                    }
                }
                this.isTesting.set(false);
            },
            error: (err: HttpErrorResponse | Error) => {
                const errorDetail = parseErrorMessage(err, 'Verification failed');
                for (const proxy of proxies) {
                    this.importStatuses.update(prev => ({
                        ...prev,
                        [proxy]: {loading: false, valid: false, error: errorDetail}
                    }));
                }
                this.isTesting.set(false);
            }
        });
    }

    protected copyToClipboard(): void {
        const text = this.workingProxies().join('\n');
        navigator.clipboard.writeText(text);
    }
}

const parseProxies = (text: string): string[] => {
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