export interface ProxyTestResultRdo {
    url: string;
    valid: boolean;
    error?: string;
}

export interface ProxyTestResponseRdo {
    results: ProxyTestResultRdo[];
}
