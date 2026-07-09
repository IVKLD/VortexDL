import { Component, input, computed } from '@angular/core';

@Component({
    selector: 'app-list-view-shell',
    templateUrl: './list-view-shell.html',
    styleUrl: './list-view-shell.scss',
})
export class ListViewShellComponent {
    protected readonly showSkeleton = computed(() => this.loading() && !this.hasResults());
    protected readonly showResults = computed(() => this.hasResults());
    protected readonly showResultsDimmed = computed(() => this.hasResults() && this.loading());
    protected readonly showEmpty = computed(() =>
        this.empty() || (!this.hasResults() && !this.loading() && !this.initial())
    );
    protected readonly showInitial = computed(() => this.initial() && !this.loading());

    public loading = input(false);
    public hasResults = input(false);
    public empty = input(false);
    public initial = input(false);
}
