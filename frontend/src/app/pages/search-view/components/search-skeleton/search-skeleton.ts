import { Component, OnInit, signal } from '@angular/core';

@Component({
    selector: 'app-search-skeleton',
    imports: [],
    templateUrl: './search-skeleton.html',
    styleUrl: './search-skeleton.scss',
    host: {
        '(window:resize)': 'onResize()'
    }
})
export class SearchSkeletonComponent implements OnInit {
    protected readonly skeletons = signal<number[]>([]);

    private _updateCount(): void {
        const height = window.innerHeight;
        const rowHeight = 80;
        const count = Math.max(5, Math.ceil(height / rowHeight) + 1);

        if (this.skeletons().length !== count) {
            this.skeletons.set(Array.from({ length: count }, (_, i) => i));
        }
    }

    protected onResize(): void {
        this._updateCount();
    }

    public ngOnInit(): void {
        this._updateCount();
    }
}

