import { Component, input } from '@angular/core';
import { MatIcon } from '@angular/material/icon';

export interface ActiveProxyOperation {
    proxy: string | null;
    url: string;
    status: string;
}

export interface ProxyAttempt {
    proxy: string;
    status: string;
    success?: boolean;
    error?: string;
}

@Component({
    selector: 'app-proxy-resolver',
    imports: [MatIcon],
    templateUrl: './proxy-resolver.html',
    styleUrl: './proxy-resolver.scss',
})
export class ProxyResolverComponent {
    public readonly activeOperation = input.required<ActiveProxyOperation>();
    public readonly attempts = input.required<ProxyAttempt[]>();
}
