import {AudioFormat} from '@shared/models/music-track.model';

export interface DashboardStat {
    icon: string;
    label: string;
    value: number;
    iconClass: string;
    isSize?: boolean;
}

export interface ActivityDay {
    label: string;
    count: number;
    heightPercent: number;
}

export interface FormatItem {
    format: AudioFormat;
    count: number;
    percentage: number;
    color: string;
}

