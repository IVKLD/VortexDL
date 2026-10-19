import { Directive, inject, input, output } from '@angular/core';
import { DragSelectService } from './drag-select.service';

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
        '(mousemove)': 'onMouseMove($event)',
        '(click)': 'onClick($event)',
        '(window:mouseup)': 'onWindowMouseUp()',
    },
})
export class DragSelectDirective<T extends SelectableItem = SelectableItem> {
    private readonly _dragService = inject(DragSelectService);

    public readonly item = input.required<T>({ alias: 'appDragSelect' });
    public readonly selection = input.required<SelectionHandler>();
    public readonly itemClick = output<T>();

    private applyAction(): void {
        if (this._dragService.dragAction === 'deselect') {
            this.selection().deselect(this.item().id);
        } else {
            this.selection().select(this.item().id);
        }
    }

    protected onMouseDown(event: MouseEvent): void {
        if (event.button !== 0) return;

        const isSelected = this.selection().has(this.item().id);
        this._dragService.startPress(event.clientX, event.clientY, isSelected ? 'deselect' : 'select');

        if (this.selection().hasSelection()) {
            this._dragService.startDrag();
            this.applyAction();
        } else {
            this._dragService.longPressTimer = setTimeout(() => {
                if (this._dragService.isMouseDown()) {
                    this._dragService.startDrag();
                    this.applyAction();
                }
            }, 250);
        }
    }

    protected onMouseEnter(): void {
        if (this._dragService.isMouseDown()) {
            this._dragService.startDrag();
            this.applyAction();
        }
    }

    protected onMouseMove(event: MouseEvent): void {
        if (
            this._dragService.isMouseDown() &&
            !this._dragService.isDragActive() &&
            !this.selection().hasSelection()
        ) {
            const dx = Math.abs(event.clientX - this._dragService.pressStartX);
            const dy = Math.abs(event.clientY - this._dragService.pressStartY);
            if (dx > 8 || dy > 8) {
                this._dragService.clearTimer();
            }
        }
    }

    protected onWindowMouseUp(): void {
        this._dragService.endDrag();
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
