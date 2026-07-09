import {Component, inject, signal, computed} from '@angular/core';
import {CommonModule} from '@angular/common';
import {MatIcon} from '@angular/material/icon';
import {MatButton} from '@angular/material/button';
import {
    MatDialogClose,
    MatDialogContent,
    MatDialogTitle,
    MatDialogActions
} from '@angular/material/dialog';
import {MatTooltip} from '@angular/material/tooltip';
import {SettingsTestingService} from '@app/pages/settings-view/settings.service';
import { FixedSizeVirtualScrollStrategy, RxVirtualFor, RxVirtualScrollViewportComponent } from '@rx-angular/template/virtual-scrolling';
import {finalize} from 'rxjs';

export interface ProxyStatus {
    url: string;
    loading: boolean;
    valid?: boolean;
    error?: string;
}

@Component({
    selector: 'app-proxy-checker-dialog',
    imports: [
        CommonModule, MatIcon, MatButton,
        MatDialogTitle, MatDialogContent, MatDialogActions, MatDialogClose,
        MatTooltip, RxVirtualScrollViewportComponent, RxVirtualFor, FixedSizeVirtualScrollStrategy
    ],
    templateUrl: './proxy-checker-dialog.component.html',
    styleUrl: './proxy-checker-dialog.component.scss',
})
export class ProxyCheckerDialogComponent {
    private readonly _testing = inject(SettingsTestingService);

    protected readonly importStatuses = signal<ProxyStatus[]>([]);
    protected readonly workingProxies = signal<string[]>([]);
    protected readonly isTesting = signal<boolean>(false);
    protected readonly copied = signal<boolean>(false);

    public readonly pastedText = signal<string>('');

    public readonly parsedProxies = computed(() => parseProxies(this.pastedText()));

    public readonly checkingCount = computed(() => this.importStatuses().filter(s => s.loading).length);

    protected onTextChange(event: Event): void {
        if (event.target instanceof HTMLTextAreaElement) this.pastedText.set(event.target.value);
    }

    protected testAll(): void {
        const proxies = this.parsedProxies();
        if (proxies.length === 0) return;

        this.isTesting.set(true);
        this.workingProxies.set([]);
        this.importStatuses.set(proxies.map(url => ({ url, loading: true })));

        this._testing.testProxy(proxies)
            .pipe(finalize(() => this.isTesting.set(false)))
            .subscribe({
                next: (result) => {
                    this.importStatuses.update(statuses =>
                        statuses.map(s => s.url === result.url ? {
                            ...s,
                            loading: false,
                            valid: result.valid,
                            error: result.error || undefined
                        } : s)
                    );
                    if (result.valid) this.workingProxies.update(prev => [...prev, result.url]);
                }
            });
    }

    protected copyToClipboard(): void {
        navigator.clipboard.writeText(this.workingProxies().join('\n'));
        this.copied.set(true);
        setTimeout(() => this.copied.set(false), 2000);
    }
}

const parseProxies = (text: string): string[] => {
    if (!text) return [];
    const lines = text.split(/[\n,;]+/).map(l => l.trim()).filter(l => l.length > 0);
    const valid = lines.filter(l => /^(http|https|socks4|socks4a|socks5|socks5h):\/\//i.test(l));
    return Array.from(new Set(valid));
}