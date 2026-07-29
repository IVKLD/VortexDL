import { Signal } from '@angular/core';
import { FieldTree } from '@angular/forms/signals';

export enum HeaderFeature {
    Search = 'search',
    Sort = 'sort',
    Stats = 'stats'
}

export interface HeaderConfig {
    title: string;
    features: HeaderFeature[];
    searchPlaceholder?: string;
    sortOptions?: HeaderSortOption<unknown>[];
}

export interface HeaderSearchBind {
    formField: FieldTree<string>;
    onFocus?: () => void;
    onBlur?: () => void;
    onSubmit?: (query: string) => void;
}

export interface HeaderSortOption<T = unknown> {
    label: string;
    shortLabel?: string;
    value: T;
    icon: string;
}

export interface HeaderSortBind<T = unknown> {
    value: Signal<T>;
    onSortChange: (value: T) => void;
}
