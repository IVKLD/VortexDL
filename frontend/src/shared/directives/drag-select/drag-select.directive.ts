import { Directive, inject, input, output } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { EMPTY, filter, fromEvent, Subject, switchMap, takeUntil, tap, timer } from 'rxjs';
import { DragAction, DragSelectService } from './drag-select.service';

export interface SelectableItem {
    id: number;
}

export interface SelectionHandler {
    has(id: number): boolean;
    hasSelection(): boolean;
    select(id: number): void;
    deselect(id: number): void;
    toggle(id: number): void;
}

@Directive({
    selector: '[appDragSelect]',
    host: {
        '(mousedown)': 'onMouseDown($event)',
        '(mouseenter)': 'onMouseEnter()',
        '(click)': 'onClick($event)',
    },
})
export class DragSelectDirective<T extends SelectableItem> {
    private readonly _dragService = inject(DragSelectService);

    public readonly item = input.required<T>({ alias: 'appDragSelect' });
    public readonly selection = input.required<SelectionHandler>();
    private readonly _press$ = new Subject<MouseEvent>();
    public readonly itemClick = output<T>();

    constructor() {
        this._press$
            .pipe(
                switchMap((event) => {
                    const isSelected = this.selection().has(this.item().id);
                    const action: DragAction = isSelected ? 'deselect' : 'select';

                    if (this.selection().hasSelection()) {
                        this._dragService.startDrag(action);
                        this.applyAction(action);
                        return EMPTY;
                    }

                    const { clientX: startX, clientY: startY } = event;

                    return timer(250).pipe(
                        takeUntil(
                            fromEvent<MouseEvent>(window, 'mousemove').pipe(
                                filter((m) => Math.hypot(m.clientX - startX, m.clientY - startY) > 8),
                            ),
                        ),
                        takeUntil(fromEvent(window, 'mouseup')),
                        tap(() => {
                            this._dragService.startDrag(action);
                            this.applyAction(action);
                        }),
                    );
                }),
                takeUntilDestroyed(),
            )
            .subscribe();
    }

    private applyAction(action: DragAction): void {
        if (action === 'deselect') {
            this.selection().deselect(this.item().id);
        } else {
            this.selection().select(this.item().id);
        }
    }

    protected onMouseDown(event: MouseEvent): void {
        if (event.button !== 0) return;
        this._press$.next(event);
    }

    protected onMouseEnter(): void {
        if (this._dragService.isDragActive()) {
            this.applyAction(this._dragService.dragAction);
        }
    }

    protected onClick(event: MouseEvent): void {
        if (this._dragService.justFinishedDrag) {
            event.preventDefault();
            event.stopPropagation();
            this._dragService.justFinishedDrag = false;
            return;
        }

        if (this.selection().hasSelection()) {
            this.selection().toggle(this.item().id);
        } else {
            this.itemClick.emit(this.item());
        }
    }
}
